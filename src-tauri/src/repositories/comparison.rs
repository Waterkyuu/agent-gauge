use crate::domain::comparison::{
    AgentKind, ComparisonAgentSummary, ComparisonCursor, ComparisonDetail, ComparisonPage,
    ComparisonResultDetail, ComparisonResultStatus, ComparisonStatus, ComparisonSummary,
    NewComparisonOutcome, NewComparisonRun, NewTokenUsage, NewToolCall,
};
use crate::models::comparison::{comparison_result, comparison_run, comparison_tool_call};
use sea_orm::sea_query::{Expr, ExprTrait};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    FromQueryResult, QueryFilter, QueryOrder, QuerySelect, SqliteTransactionMode,
    TransactionOptions, TransactionTrait,
};
use std::collections::HashMap;

/// SeaORM repository for transactional history writes and bounded read queries.
#[derive(Clone)]
pub(crate) struct ComparisonRepository {
    /// Shared asynchronous SQLite connection pool.
    database: DatabaseConnection,
}

#[derive(Debug, FromQueryResult)]
struct AgentSummaryRow {
    /// Parent comparison identifier.
    comparison_run_id: i64,
    /// Stable persisted Agent identifier.
    agent_kind: String,
    /// Stable persisted result status.
    status: String,
}

impl ComparisonRepository {
    /// Creates a repository over the initialized application database.
    pub(crate) fn new(database: DatabaseConnection) -> Self {
        Self { database }
    }

    /// Inserts a complete comparison and all child records in one immediate transaction.
    pub(crate) async fn insert(&self, run: &NewComparisonRun) -> Result<i64, DbErr> {
        let status = aggregate_status(&run.results);
        let transaction = self
            .database
            .begin_with_options(TransactionOptions {
                sqlite_transaction_mode: Some(SqliteTransactionMode::Immediate),
                ..TransactionOptions::default()
            })
            .await?;
        let run_model = comparison_run::ActiveModel {
            query: Set(run.query.clone()),
            status: Set(status.as_str().to_string()),
            metric_version: Set(run.metric_version),
            created_at_ms: Set(run.created_at_ms),
            ..Default::default()
        }
        .insert(&transaction)
        .await?;

        for result in &run.results {
            let (
                result_status,
                response,
                error_message,
                total_duration_ms,
                time_to_first_token_ms,
                thinking_duration_ms,
                token_usage,
                tool_calls,
            ) = match &result.outcome {
                NewComparisonOutcome::Succeeded {
                    response,
                    total_duration_ms,
                    time_to_first_token_ms,
                    thinking_duration_ms,
                    token_usage,
                    tool_calls,
                } => (
                    ComparisonResultStatus::Succeeded,
                    Some(response.clone()),
                    None,
                    Some(*total_duration_ms),
                    *time_to_first_token_ms,
                    Some(*thinking_duration_ms),
                    token_usage.as_ref(),
                    tool_calls.as_slice(),
                ),
                NewComparisonOutcome::Failed { error_message } => (
                    ComparisonResultStatus::Failed,
                    None,
                    Some(error_message.clone()),
                    None,
                    None,
                    None,
                    None,
                    &[][..],
                ),
            };
            let result_model = comparison_result::ActiveModel {
                comparison_run_id: Set(run_model.id),
                agent_kind: Set(result.agent.as_str().to_string()),
                model: Set(result.model.clone()),
                reasoning_effort: Set(result.reasoning_effort.clone()),
                status: Set(result_status.as_str().to_string()),
                response: Set(response),
                error_message: Set(error_message),
                total_duration_ms: Set(total_duration_ms),
                time_to_first_token_ms: Set(time_to_first_token_ms),
                thinking_duration_ms: Set(thinking_duration_ms),
                total_tokens: Set(token_usage.map(|usage| usage.total_tokens)),
                input_tokens: Set(token_usage.map(|usage| usage.input_tokens)),
                cached_input_tokens: Set(token_usage.map(|usage| usage.cached_input_tokens)),
                cache_write_input_tokens: Set(
                    token_usage.map(|usage| usage.cache_write_input_tokens)
                ),
                output_tokens: Set(token_usage.map(|usage| usage.output_tokens)),
                reasoning_output_tokens: Set(
                    token_usage.and_then(|usage| usage.reasoning_output_tokens)
                ),
                ..Default::default()
            }
            .insert(&transaction)
            .await?;

            if !tool_calls.is_empty() {
                comparison_tool_call::Entity::insert_many(tool_calls.iter().map(|tool_call| {
                    comparison_tool_call::ActiveModel {
                        comparison_result_id: Set(result_model.id),
                        sequence: Set(tool_call.sequence),
                        name: Set(tool_call.name.clone()),
                        duration_ms: Set(tool_call.duration_ms),
                        ..Default::default()
                    }
                }))
                .exec(&transaction)
                .await?;
            }
        }

        transaction.commit().await?;
        Ok(run_model.id)
    }

    /// Loads one bounded newest-first history page without response bodies or tool details.
    pub(crate) async fn list(
        &self,
        cursor: Option<ComparisonCursor>,
        limit: u64,
    ) -> Result<ComparisonPage, DbErr> {
        let mut query = comparison_run::Entity::find();
        if let Some(cursor) = cursor {
            query = query.filter(
                Expr::tuple([
                    Expr::col(comparison_run::Column::CreatedAtMs),
                    Expr::col(comparison_run::Column::Id),
                ])
                .lt(Expr::tuple([
                    Expr::value(cursor.created_at_ms),
                    Expr::value(cursor.id),
                ])),
            );
        }
        let mut runs = query
            .order_by_desc(comparison_run::Column::CreatedAtMs)
            .order_by_desc(comparison_run::Column::Id)
            .limit(limit.saturating_add(1))
            .all(&self.database)
            .await?;
        let has_more = runs.len() > limit as usize;
        if has_more {
            runs.truncate(limit as usize);
        }

        let run_ids = runs.iter().map(|run| run.id).collect::<Vec<_>>();
        let mut agents_by_run = HashMap::<i64, Vec<ComparisonAgentSummary>>::new();
        if !run_ids.is_empty() {
            let agent_rows = comparison_result::Entity::find()
                .select_only()
                .column(comparison_result::Column::ComparisonRunId)
                .column(comparison_result::Column::AgentKind)
                .column(comparison_result::Column::Status)
                .filter(comparison_result::Column::ComparisonRunId.is_in(run_ids))
                .order_by_asc(comparison_result::Column::Id)
                .into_model::<AgentSummaryRow>()
                .all(&self.database)
                .await?;
            for row in agent_rows {
                agents_by_run
                    .entry(row.comparison_run_id)
                    .or_default()
                    .push(ComparisonAgentSummary {
                        agent: parse_agent(&row.agent_kind)?,
                        status: parse_result_status(&row.status)?,
                    });
            }
        }

        let items = runs
            .into_iter()
            .map(|run| {
                Ok(ComparisonSummary {
                    id: run.id,
                    query: run.query,
                    status: parse_status(&run.status)?,
                    metric_version: run.metric_version,
                    created_at_ms: run.created_at_ms,
                    agents: agents_by_run.remove(&run.id).unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>, DbErr>>()?;
        let next_cursor = if has_more {
            items.last().map(|item| ComparisonCursor {
                created_at_ms: item.created_at_ms,
                id: item.id,
            })
        } else {
            None
        };

        Ok(ComparisonPage { items, next_cursor })
    }

    /// Loads one complete comparison by primary key using batched child queries.
    pub(crate) async fn find(&self, id: i64) -> Result<Option<ComparisonDetail>, DbErr> {
        let Some(run) = comparison_run::Entity::find_by_id(id)
            .one(&self.database)
            .await?
        else {
            return Ok(None);
        };
        let result_models = comparison_result::Entity::find()
            .filter(comparison_result::Column::ComparisonRunId.eq(id))
            .order_by_asc(comparison_result::Column::Id)
            .all(&self.database)
            .await?;
        let result_ids = result_models
            .iter()
            .map(|result| result.id)
            .collect::<Vec<_>>();
        let mut tools_by_result = HashMap::<i64, Vec<NewToolCall>>::new();
        if !result_ids.is_empty() {
            let tool_models = comparison_tool_call::Entity::find()
                .filter(comparison_tool_call::Column::ComparisonResultId.is_in(result_ids))
                .order_by_asc(comparison_tool_call::Column::ComparisonResultId)
                .order_by_asc(comparison_tool_call::Column::Sequence)
                .all(&self.database)
                .await?;
            for tool in tool_models {
                tools_by_result
                    .entry(tool.comparison_result_id)
                    .or_default()
                    .push(NewToolCall {
                        sequence: tool.sequence,
                        name: tool.name,
                        duration_ms: tool.duration_ms,
                    });
            }
        }

        let results = result_models
            .into_iter()
            .map(|result| {
                let token_usage = result.total_tokens.map(|total_tokens| NewTokenUsage {
                    total_tokens,
                    input_tokens: result.input_tokens.unwrap_or_default(),
                    cached_input_tokens: result.cached_input_tokens.unwrap_or_default(),
                    cache_write_input_tokens: result.cache_write_input_tokens.unwrap_or_default(),
                    output_tokens: result.output_tokens.unwrap_or_default(),
                    reasoning_output_tokens: result.reasoning_output_tokens,
                });
                Ok(ComparisonResultDetail {
                    id: result.id,
                    agent: parse_agent(&result.agent_kind)?,
                    model: result.model,
                    reasoning_effort: result.reasoning_effort,
                    status: parse_result_status(&result.status)?,
                    response: result.response,
                    error_message: result.error_message,
                    total_duration_ms: result.total_duration_ms,
                    time_to_first_token_ms: result.time_to_first_token_ms,
                    thinking_duration_ms: result.thinking_duration_ms,
                    token_usage,
                    tool_calls: tools_by_result.remove(&result.id).unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>, DbErr>>()?;

        Ok(Some(ComparisonDetail {
            id: run.id,
            query: run.query,
            status: parse_status(&run.status)?,
            metric_version: run.metric_version,
            created_at_ms: run.created_at_ms,
            results,
        }))
    }
}

/// Computes the aggregate state from the final Agent outcomes.
fn aggregate_status(
    results: &[crate::domain::comparison::NewComparisonResult],
) -> ComparisonStatus {
    let succeeded = results
        .iter()
        .filter(|result| matches!(result.outcome, NewComparisonOutcome::Succeeded { .. }))
        .count();
    match succeeded {
        0 => ComparisonStatus::Failed,
        count if count == results.len() => ComparisonStatus::Completed,
        _ => ComparisonStatus::Partial,
    }
}

/// Converts a checked database Agent value into its domain representation.
fn parse_agent(value: &str) -> Result<AgentKind, DbErr> {
    AgentKind::parse(value).ok_or_else(|| DbErr::Type("invalid persisted agent kind".to_string()))
}

/// Converts a checked aggregate status into its domain representation.
fn parse_status(value: &str) -> Result<ComparisonStatus, DbErr> {
    ComparisonStatus::parse(value)
        .ok_or_else(|| DbErr::Type("invalid persisted comparison status".to_string()))
}

/// Converts a checked result status into its domain representation.
fn parse_result_status(value: &str) -> Result<ComparisonResultStatus, DbErr> {
    ComparisonResultStatus::parse(value)
        .ok_or_else(|| DbErr::Type("invalid persisted result status".to_string()))
}

#[cfg(test)]
mod tests {
    use super::ComparisonRepository;
    use crate::db::connection::connect_sqlite;
    use crate::db::migration::Migrator;
    use crate::domain::comparison::{
        AgentKind, ComparisonCursor, NewComparisonOutcome, NewComparisonResult, NewComparisonRun,
        NewTokenUsage, NewToolCall,
    };
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
    use sea_orm_migration::MigratorTrait;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DATABASE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    /// Creates one migrated repository backed by an isolated temporary file.
    async fn migrated_repository() -> (
        ComparisonRepository,
        sea_orm::DatabaseConnection,
        std::path::PathBuf,
    ) {
        let sequence = DATABASE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "agent-gauge-repository-test-{}-{sequence}.sqlite3",
            std::process::id()
        ));
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let database = connect_sqlite(&url).await.expect("database should connect");
        Migrator::up(&database, None)
            .await
            .expect("migration should succeed");
        (ComparisonRepository::new(database.clone()), database, path)
    }

    /// Builds one successful run result with metrics that are easy to verify.
    fn successful_result(agent: AgentKind) -> NewComparisonResult {
        NewComparisonResult {
            agent,
            model: Some("test-model".to_string()),
            reasoning_effort: Some("high".to_string()),
            outcome: NewComparisonOutcome::Succeeded {
                response: "finished".to_string(),
                total_duration_ms: 2_500,
                time_to_first_token_ms: Some(300),
                thinking_duration_ms: 800,
                token_usage: Some(NewTokenUsage {
                    total_tokens: 120,
                    input_tokens: 80,
                    cached_input_tokens: 20,
                    cache_write_input_tokens: 0,
                    output_tokens: 40,
                    reasoning_output_tokens: Some(10),
                }),
                tool_calls: vec![NewToolCall {
                    sequence: 1,
                    name: "Read".to_string(),
                    duration_ms: 75,
                }],
            },
        }
    }

    #[test]
    fn saves_and_reloads_every_result_in_one_comparison() {
        tauri::async_runtime::block_on(async {
            let (repository, database, path) = migrated_repository().await;
            let run_id = repository
                .insert(&NewComparisonRun {
                    query: "Compare this repository".to_string(),
                    metric_version: 1,
                    created_at_ms: 1_700_000_000_000,
                    results: vec![
                        successful_result(AgentKind::Codex),
                        NewComparisonResult {
                            agent: AgentKind::Claude,
                            model: None,
                            reasoning_effort: None,
                            outcome: NewComparisonOutcome::Failed {
                                error_message: "Task failed".to_string(),
                            },
                        },
                    ],
                })
                .await
                .expect("comparison should save");

            let detail = repository
                .find(run_id)
                .await
                .expect("comparison should load")
                .expect("comparison should exist");

            assert_eq!(detail.status.as_str(), "partial");
            assert_eq!(detail.results.len(), 2);
            assert_eq!(detail.results[0].agent, AgentKind::Codex);
            assert_eq!(detail.results[0].tool_calls.len(), 1);
            assert_eq!(detail.results[1].agent, AgentKind::Claude);

            database.close().await.expect("database should close");
            std::fs::remove_file(path).expect("temporary database should be removable");
        });
    }

    #[test]
    fn paginates_history_with_a_stable_compound_cursor() {
        tauri::async_runtime::block_on(async {
            let (repository, database, path) = migrated_repository().await;
            for created_at_ms in [100, 200, 200] {
                repository
                    .insert(&NewComparisonRun {
                        query: format!("query-{created_at_ms}"),
                        metric_version: 1,
                        created_at_ms,
                        results: vec![successful_result(AgentKind::Codex)],
                    })
                    .await
                    .expect("comparison should save");
            }

            let first_page = repository
                .list(None, 2)
                .await
                .expect("first page should load");
            let second_page = repository
                .list(first_page.next_cursor, 2)
                .await
                .expect("second page should load");

            assert_eq!(first_page.items.len(), 2);
            assert_eq!(second_page.items.len(), 1);
            assert!(first_page.items[0].id > first_page.items[1].id);
            assert_eq!(second_page.items[0].created_at_ms, 100);
            assert_eq!(second_page.next_cursor, None::<ComparisonCursor>);

            database.close().await.expect("database should close");
            std::fs::remove_file(path).expect("temporary database should be removable");
        });
    }

    #[test]
    fn cursor_lookup_seeks_through_the_compound_history_index() {
        tauri::async_runtime::block_on(async {
            let (_repository, database, path) = migrated_repository().await;
            let plan = database
                .query_all_raw(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    "EXPLAIN QUERY PLAN
                     SELECT id, query, status, metric_version, created_at_ms
                     FROM comparison_runs
                     WHERE (created_at_ms, id) < (200, 5)
                     ORDER BY created_at_ms DESC, id DESC
                     LIMIT 31"
                        .to_string(),
                ))
                .await
                .expect("query plan should be readable")
                .into_iter()
                .map(|row| {
                    row.try_get::<String>("", "detail")
                        .expect("query plan detail should be text")
                })
                .collect::<Vec<_>>()
                .join("\n");

            assert!(
                plan.contains("SEARCH comparison_runs USING INDEX idx_comparison_runs_history"),
                "unexpected query plan: {plan}"
            );

            database.close().await.expect("database should close");
            std::fs::remove_file(path).expect("temporary database should be removable");
        });
    }
}
