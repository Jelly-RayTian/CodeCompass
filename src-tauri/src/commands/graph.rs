use tauri::State;

use crate::analysis::graph::build_graph;
use crate::db::indexed_folders::get_folder_path;
use crate::db::Database;
use crate::error::AppError;
use crate::models::DependencyGraph;

#[tauri::command]
pub fn get_dependency_graph(
    db: State<'_, Database>,
    workspace_id: i64,
) -> Result<DependencyGraph, AppError> {
    if get_folder_path(&db, workspace_id)?.is_none() {
        return Err(AppError::FolderNotFound(format!(
            "workspace {}",
            workspace_id
        )));
    }
    build_graph(&db, workspace_id)
}
