use tauri::State;

use crate::db::Database;
use crate::error::AppError;
use crate::models::DatabaseStatus;

/// Returns the current status of the local SQLite database, including
/// whether it is connected, its filesystem path, and the latest
/// migration version that has been applied.
#[tauri::command]
pub fn get_database_status(db: State<'_, Database>) -> Result<DatabaseStatus, AppError> {
    let migration_version = db.migration_version()?;
    Ok(DatabaseStatus {
        connected: true,
        database_path: db.path().to_string(),
        migration_version,
    })
}
