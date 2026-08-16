use sea_orm::sqlx::sqlite::{SqliteJournalMode, SqliteSynchronous};
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::path::Path;
use std::time::Duration;

/// Opens the SQLite database used by persistent application repositories.
#[cfg(test)]
pub(crate) async fn connect_sqlite(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let mut options = ConnectOptions::new(database_url);
    options
        .min_connections(1)
        .max_connections(4)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(5))
        .test_before_acquire(false)
        .sqlx_logging(false)
        .record_stmt_in_spans(false)
        .map_sqlx_sqlite_opts(|options| {
            options
                .create_if_missing(true)
                .foreign_keys(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .busy_timeout(Duration::from_secs(5))
                .statement_cache_capacity(128)
        });

    Database::connect(options).await
}

/// Opens a cross-platform SQLite file without converting the native path into a URL.
pub(crate) async fn connect_sqlite_path(path: &Path) -> Result<DatabaseConnection, DbErr> {
    let database_path = path.to_path_buf();
    let mut options = ConnectOptions::new("sqlite::memory:");
    options
        .min_connections(1)
        .max_connections(4)
        .connect_timeout(Duration::from_secs(5))
        .acquire_timeout(Duration::from_secs(5))
        .test_before_acquire(false)
        .sqlx_logging(false)
        .record_stmt_in_spans(false)
        .map_sqlx_sqlite_opts(move |options| {
            options
                .filename(&database_path)
                .in_memory(false)
                .create_if_missing(true)
                .foreign_keys(true)
                .journal_mode(SqliteJournalMode::Wal)
                .synchronous(SqliteSynchronous::Normal)
                .busy_timeout(Duration::from_secs(5))
                .statement_cache_capacity(128)
        });

    Database::connect(options).await
}

#[cfg(test)]
mod tests {
    use super::{connect_sqlite, connect_sqlite_path};
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
    use std::sync::atomic::{AtomicU64, Ordering};

    static DATABASE_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    /// Creates a unique temporary SQLite URL owned by the current test process.
    fn temporary_database_url() -> (std::path::PathBuf, String) {
        let sequence = DATABASE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "agent-gauge-db-test-{}-{sequence}.sqlite3",
            std::process::id()
        ));
        let url = format!("sqlite://{}?mode=rwc", path.display());
        (path, url)
    }

    #[test]
    fn configures_every_sqlite_connection_for_concurrent_reads_and_safe_writes() {
        tauri::async_runtime::block_on(async {
            let (path, url) = temporary_database_url();
            let database = connect_sqlite(&url).await.expect("database should connect");

            let journal_mode = database
                .query_one_raw(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    "PRAGMA journal_mode".to_string(),
                ))
                .await
                .expect("journal mode should be readable")
                .expect("journal mode row should exist")
                .try_get::<String>("", "journal_mode")
                .expect("journal mode should be text");
            let foreign_keys = database
                .query_one_raw(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    "PRAGMA foreign_keys".to_string(),
                ))
                .await
                .expect("foreign key mode should be readable")
                .expect("foreign key row should exist")
                .try_get::<i32>("", "foreign_keys")
                .expect("foreign key mode should be numeric");

            assert_eq!(journal_mode, "wal");
            assert_eq!(foreign_keys, 1);

            database.close().await.expect("database should close");
            std::fs::remove_file(path).expect("temporary database should be removable");
        });
    }

    #[test]
    fn persists_the_database_at_the_requested_path() {
        tauri::async_runtime::block_on(async {
            let (path, _) = temporary_database_url();
            let database = connect_sqlite_path(&path)
                .await
                .expect("database file should connect");
            database
                .execute_unprepared("CREATE TABLE persistence_probe (id INTEGER PRIMARY KEY)")
                .await
                .expect("probe table should be created");
            database.close().await.expect("database should close");

            assert!(path.is_file(), "the requested SQLite file should exist");

            let reopened = connect_sqlite_path(&path)
                .await
                .expect("database file should reopen");
            let table = reopened
                .query_one_raw(Statement::from_string(
                    DatabaseBackend::Sqlite,
                    "SELECT name FROM sqlite_master WHERE name = 'persistence_probe'".to_string(),
                ))
                .await
                .expect("persisted schema should be readable");

            assert!(table.is_some());

            reopened.close().await.expect("database should close");
            std::fs::remove_file(path).expect("temporary database should be removable");
        });
    }
}
