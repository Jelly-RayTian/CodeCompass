use tauri::State;

use crate::analysis::health::{build_health_report, RepositoryHealth};
use crate::db::Database;
use crate::error::AppError;

#[tauri::command]
pub fn get_repository_health(
    db: State<'_, Database>,
    workspace_id: i64,
) -> Result<RepositoryHealth, AppError> {
    build_health_report(&db, workspace_id)
}
