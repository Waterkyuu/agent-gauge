use crate::domain::comparison::{
    ComparisonCursor, ComparisonDetail, ComparisonPage, NewComparisonOutcome, NewComparisonResult,
    NewComparisonRun, NewTokenUsage, NewToolCall,
};
use crate::dto::comparison::{
    ComparisonOutcomeRequest, ComparisonResultRequest, ListComparisonsRequest,
    SaveComparisonRequest,
};
use crate::error::AppError;
use crate::repositories::comparison::ComparisonRepository;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_PAGE_SIZE: u64 = 30;
const MAX_PAGE_SIZE: u64 = 100;
const MAX_RESPONSE_LENGTH: usize = 2_000_000;
const MAX_ERROR_LENGTH: usize = 4096;
const MAX_TOOL_CALLS: usize = 1000;

/// Validates comparison contracts and coordinates persistence use cases.
#[derive(Clone)]
pub(crate) struct ComparisonService {
    /// Persistence boundary for comparison history.
    repository: ComparisonRepository,
}

impl ComparisonService {
    /// Creates a comparison history service over its repository.
    pub(crate) fn new(repository: ComparisonRepository) -> Self {
        Self { repository }
    }

    /// Validates and atomically persists one completed comparison.
    pub(crate) async fn save(&self, request: SaveComparisonRequest) -> Result<i64, AppError> {
        let query = request.query.trim().to_string();
        if query.is_empty() || query.len() > 16_000 || request.results.is_empty() {
            return Err(AppError::InvalidComparison);
        }
        let mut agents = HashSet::with_capacity(request.results.len());
        let mut results = Vec::with_capacity(request.results.len());
        for result in request.results {
            if !agents.insert(result.agent) {
                return Err(AppError::InvalidComparison);
            }
            results.push(map_result(result)?);
        }
        let created_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AppError::ComparisonDatabaseFailed)?
            .as_millis();
        let created_at_ms =
            i64::try_from(created_at_ms).map_err(|_| AppError::ComparisonDatabaseFailed)?;

        self.repository
            .insert(&NewComparisonRun {
                query,
                metric_version: 1,
                created_at_ms,
                results,
            })
            .await
            .map_err(|_| AppError::ComparisonDatabaseFailed)
    }

    /// Loads one validated bounded history page.
    pub(crate) async fn list(
        &self,
        request: ListComparisonsRequest,
    ) -> Result<ComparisonPage, AppError> {
        let limit = request.limit.unwrap_or(DEFAULT_PAGE_SIZE);
        if limit == 0 || limit > MAX_PAGE_SIZE {
            return Err(AppError::InvalidComparison);
        }
        let cursor = request.cursor.map(|cursor| ComparisonCursor {
            created_at_ms: cursor.created_at_ms,
            id: cursor.id,
        });
        if cursor.is_some_and(|cursor| cursor.created_at_ms <= 0 || cursor.id <= 0) {
            return Err(AppError::InvalidComparison);
        }
        self.repository
            .list(cursor, limit)
            .await
            .map_err(|_| AppError::ComparisonDatabaseFailed)
    }

    /// Loads one complete comparison or returns a stable missing-record error.
    pub(crate) async fn find(&self, id: i64) -> Result<ComparisonDetail, AppError> {
        if id <= 0 {
            return Err(AppError::InvalidComparison);
        }
        self.repository
            .find(id)
            .await
            .map_err(|_| AppError::ComparisonDatabaseFailed)?
            .ok_or(AppError::ComparisonNotFound)
    }
}

/// Maps and bounds one untrusted IPC result before persistence.
fn map_result(result: ComparisonResultRequest) -> Result<NewComparisonResult, AppError> {
    if result.model.as_ref().is_some_and(|value| value.len() > 256)
        || result
            .reasoning_effort
            .as_ref()
            .is_some_and(|value| value.len() > 64)
    {
        return Err(AppError::InvalidComparison);
    }
    let outcome = match result.outcome {
        ComparisonOutcomeRequest::Succeeded { result } => {
            if result.response.len() > MAX_RESPONSE_LENGTH
                || result.tool_calls.len() > MAX_TOOL_CALLS
                || usize::try_from(result.tool_call_count).ok() != Some(result.tool_calls.len())
            {
                return Err(AppError::InvalidComparison);
            }
            let tool_calls = result
                .tool_calls
                .into_iter()
                .enumerate()
                .map(|(index, tool)| {
                    if tool.sequence != index as u64 + 1
                        || tool.name.is_empty()
                        || tool.name.len() > 256
                    {
                        return Err(AppError::InvalidComparison);
                    }
                    Ok(NewToolCall {
                        sequence: to_i64(tool.sequence)?,
                        name: tool.name,
                        duration_ms: to_i64(tool.duration_ms)?,
                    })
                })
                .collect::<Result<Vec<_>, AppError>>()?;
            NewComparisonOutcome::Succeeded {
                response: result.response,
                total_duration_ms: to_i64(result.total_duration_ms)?,
                time_to_first_token_ms: result.time_to_first_token_ms.map(to_i64).transpose()?,
                thinking_duration_ms: to_i64(result.thinking_duration_ms)?,
                token_usage: result.token_usage.map(map_token_usage).transpose()?,
                tool_calls,
            }
        }
        ComparisonOutcomeRequest::Failed { error_message } => {
            if error_message.is_empty() || error_message.len() > MAX_ERROR_LENGTH {
                return Err(AppError::InvalidComparison);
            }
            NewComparisonOutcome::Failed { error_message }
        }
    };
    Ok(NewComparisonResult {
        agent: result.agent.into(),
        model: result.model,
        reasoning_effort: result.reasoning_effort,
        outcome,
    })
}

/// Converts bounded token counters into SQLite signed integers.
fn map_token_usage(
    usage: crate::dto::comparison::TokenUsageRequest,
) -> Result<NewTokenUsage, AppError> {
    Ok(NewTokenUsage {
        total_tokens: to_i64(usage.total_tokens)?,
        input_tokens: to_i64(usage.input_tokens)?,
        cached_input_tokens: to_i64(usage.cached_input_tokens)?,
        cache_write_input_tokens: to_i64(usage.cache_write_input_tokens)?,
        output_tokens: to_i64(usage.output_tokens)?,
        reasoning_output_tokens: usage.reasoning_output_tokens.map(to_i64).transpose()?,
    })
}

/// Rejects unsigned IPC counters that SQLite cannot represent safely.
fn to_i64(value: u64) -> Result<i64, AppError> {
    i64::try_from(value).map_err(|_| AppError::InvalidComparison)
}

#[cfg(test)]
mod tests {
    use super::ComparisonService;
    use crate::db::connection::connect_sqlite;
    use crate::db::migration::Migrator;
    use crate::dto::comparison::{
        AgentKindRequest, ComparisonOutcomeRequest, ComparisonResultRequest, SaveComparisonRequest,
    };
    use crate::error::AppError;
    use crate::repositories::comparison::ComparisonRepository;
    use sea_orm_migration::MigratorTrait;
    use std::sync::atomic::{AtomicU64, Ordering};

    static DATABASE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    /// Creates a migrated service and its owned temporary database resources.
    async fn migrated_service() -> (
        ComparisonService,
        sea_orm::DatabaseConnection,
        std::path::PathBuf,
    ) {
        let sequence = DATABASE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "agent-gauge-service-test-{}-{sequence}.sqlite3",
            std::process::id()
        ));
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let database = connect_sqlite(&url).await.expect("database should connect");
        Migrator::up(&database, None)
            .await
            .expect("migration should succeed");
        (
            ComparisonService::new(ComparisonRepository::new(database.clone())),
            database,
            path,
        )
    }

    #[test]
    fn rejects_duplicate_agents_before_writing_history() {
        tauri::async_runtime::block_on(async {
            let (service, database, path) = migrated_service().await;
            let duplicate = ComparisonResultRequest {
                agent: AgentKindRequest::Codex,
                model: None,
                reasoning_effort: None,
                outcome: ComparisonOutcomeRequest::Failed {
                    error_message: "failed".to_string(),
                },
            };

            let result = service
                .save(SaveComparisonRequest {
                    query: "Compare".to_string(),
                    results: vec![duplicate.clone(), duplicate],
                })
                .await;

            assert_eq!(result, Err(AppError::InvalidComparison));

            database.close().await.expect("database should close");
            std::fs::remove_file(path).expect("temporary database should be removable");
        });
    }
}
