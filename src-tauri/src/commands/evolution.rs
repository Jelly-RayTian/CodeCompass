use tauri::State;

use crate::analysis::evolution::{build_evolution_report, RepositoryEvolution};
use crate::db::Database;
use crate::error::AppError;

#[tauri::command]
pub fn get_repository_evolution(
    db: State<'_, Database>,
    workspace_id: i64,
) -> Result<RepositoryEvolution, AppError> {
    build_evolution_report(&db, workspace_id)
}
