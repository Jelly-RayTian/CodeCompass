use tauri::{AppHandle, Manager, State};

use crate::analysis::run_analysis;
use crate::db::analysis::list_diagnostics;
use crate::db::imports::list_imports_for_file;
use crate::db::indexed_files::list_recently_analyzed_files;
use crate::db::indexed_folders::get_folder_path;
use crate::db::Database;
use crate::error::AppError;
use crate::models::{AnalysisDiagnostic, ImportEntry};
use crate::tasks::AnalysisManager;

#[tauri::command]
pub fn start_analysis(
    app: AppHandle,
    db: State<'_, Database>,
    manager: State<'_, AnalysisManager>,
    workspace_id: i64,
) -> Result<(), AppError> {
    let root_path = match get_folder_path(&db, workspace_id)? {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            return Err(AppError::FolderNotFound(format!(
                "workspace {}",
                workspace_id
            )))
        }
    };

    if manager.is_running(workspace_id) {
        return Err(AppError::AnalysisAlreadyRunning(workspace_id));
    }

    let token = manager.register(workspace_id);
    let app_for_thread = app.clone();

    std::thread::spawn(move || {
        let db = app_for_thread.state::<Database>();
        let manager = app_for_thread.state::<AnalysisManager>();
        match run_analysis(&db, workspace_id, &root_path, token, &app_for_thread) {
            Ok(()) => log::info!("analysis for workspace {} finished", workspace_id),
            Err(err) => log::error!("analysis for workspace {} failed: {}", workspace_id, err),
        }
        manager.remove(workspace_id);
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_analysis(
    manager: State<'_, AnalysisManager>,
    workspace_id: i64,
) -> Result<bool, AppError> {
    Ok(manager.cancel(workspace_id))
}

#[tauri::command]
pub fn get_file_imports(
    db: State<'_, Database>,
    file_id: i64,
) -> Result<Vec<ImportEntry>, AppError> {
    list_imports_for_file(&db, file_id)
}

#[tauri::command]
pub fn get_analysis_diagnostics(
    db: State<'_, Database>,
    workspace_id: i64,
    severity: Option<String>,
) -> Result<Vec<AnalysisDiagnostic>, AppError> {
    list_diagnostics(&db, workspace_id, severity.as_deref())
}

#[tauri::command]
pub fn get_analyzed_files(
    db: State<'_, Database>,
    workspace_id: i64,
) -> Result<Vec<crate::models::FileEntry>, AppError> {
    list_recently_analyzed_files(&db, workspace_id)
}
