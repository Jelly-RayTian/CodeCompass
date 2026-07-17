use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::AppError;

use super::migrations;

/// A thread-safe wrapper around a SQLite `Connection`.
///
/// `Connection` itself is `Send` but not `Sync`, so we protect it
/// with a `Mutex`.  This allows the `Database` to be shared across
/// Tauri command handlers via `tauri::State`.
pub struct Database {
    conn: Mutex<Connection>,
    path: String,
}

impl Database {
    /// Opens (or creates) a SQLite database file at `path`, enables
    /// WAL journal mode and foreign-key enforcement, then runs all
    /// pending refinery migrations.
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let mut conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; \
             PRAGMA foreign_keys=ON; \
             PRAGMA synchronous=NORMAL; \
             PRAGMA cache_size=-8000;",
        )?;
        migrations::runner().run(&mut conn)?;
        Ok(Self {
            path: path.to_string_lossy().to_string(),
            conn: Mutex::new(conn),
        })
    }

    /// Opens an in-memory database and runs migrations.  Useful for tests.
    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, AppError> {
        let mut conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        migrations::runner().run(&mut conn)?;
        Ok(Self {
            path: ":memory:".to_string(),
            conn: Mutex::new(conn),
        })
    }

    /// Returns the filesystem path (or `:memory:`) of this database.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Acquires the mutex guard protecting the inner `Connection`.
    pub fn lock(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        self.conn
            .lock()
            .map_err(|e| AppError::DatabaseLock(e.to_string()))
    }

    /// Returns the highest migration version recorded by refinery,
    /// or `0` if the history table is empty or missing.
    pub fn migration_version(&self) -> Result<i64, AppError> {
        let conn = self.lock()?;
        let version: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM refinery_schema_history",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn open_runs_migrations() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let version = db.migration_version().expect("get version");
        assert!(
            version > 0,
            "migration version should be positive after open"
        );
    }

    #[test]
    fn open_in_memory_runs_migrations() {
        let db = Database::open_in_memory().expect("open in-memory db");
        let version = db.migration_version().expect("get version");
        assert!(version > 0);
    }

    #[test]
    fn path_returns_provided_path() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        assert!(db.path().ends_with("test.db"));
    }
}
