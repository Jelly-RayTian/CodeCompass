use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder, FilePath};
use tauri_plugin_opener::OpenerExt;

use crate::db::indexed_files::{get_file_details, list_workspace_files};
use crate::db::indexed_folders::{
    find_duplicate, find_nested_parents, get_folder_path, insert_indexed_folder,
    list_indexed_folders, remove_indexed_folder, update_folder_scan_status,
};
use crate::db::scan_runs::{
    create_scan_run, file_count, finish_scan_run, latest_scan_run, list_scan_runs,
};
use crate::db::Database;
use crate::error::AppError;
use crate::models::{AddFolderResult, FileEntry, IndexedFolder, ScanRun, ScanStatus};
use crate::platform::normalize_existing_path;
use crate::scanner::run_scan;
use crate::tasks::ScanManager;

/// Core logic for listing indexed folders, separated from the Tauri
/// `State` wrapper so it can be unit-tested with a plain `&Database`.
pub fn fetch_indexed_folders(db: &Database) -> Result<Vec<IndexedFolder>, AppError> {
    list_indexed_folders(db)
}

/// Opens the native OS folder picker and returns the selected path, or
/// `None` if the user cancelled the dialog.
#[tauri::command]
pub fn pick_folder(app: AppHandle) -> Result<Option<String>, AppError> {
    let dialog = FileDialogBuilder::new(app.dialog().clone());
    Ok(dialog.blocking_pick_folder().and_then(|p| match p {
        FilePath::Path(path) => Some(path.to_string_lossy().to_string()),
        FilePath::Url(url) => url
            .to_file_path()
            .ok()
            .map(|path| path.to_string_lossy().to_string()),
    }))
}

/// Registers a new indexed folder after validating it.
///
/// * Normalizes the path.
/// * Rejects exact duplicates.
/// * Allows nested folders but returns a warning if the new folder is
///   already covered by an existing parent folder.
#[tauri::command]
pub fn add_folder(db: State<'_, Database>, path: String) -> Result<AddFolderResult, AppError> {
    if path.trim().is_empty() {
        return Err(AppError::InvalidInput("folder path is empty".to_string()));
    }
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        return Err(AppError::FolderNotFound(path.to_string_lossy().to_string()));
    }
    let normalized = normalize_existing_path(&path)?;
    if !normalized.is_dir() {
        return Err(AppError::NotADirectory(
            normalized.to_string_lossy().to_string(),
        ));
    }

    if find_duplicate(&db, &normalized)?.is_some() {
        return Err(AppError::DuplicateFolder(
            normalized.to_string_lossy().to_string(),
        ));
    }

    let nested_warnings: Vec<String> = find_nested_parents(&db, &normalized)?
        .into_iter()
        .map(|(_, msg)| msg)
        .collect();

    let folder = insert_indexed_folder(&db, &normalized)?;
    let warning = if nested_warnings.is_empty() {
        None
    } else {
        Some(nested_warnings.join("; "))
    };
    Ok(AddFolderResult { folder, warning })
}

/// Lists all indexed folders, most-recently-added first.
#[tauri::command]
pub fn list_indexed_folders_command(
    db: State<'_, Database>,
) -> Result<Vec<IndexedFolder>, AppError> {
    fetch_indexed_folders(&db)
}

/// Removes an indexed folder and all of CodeCompass's derived index data for it.
/// The original files on disk are never modified.
#[tauri::command]
pub fn remove_indexed_folder_command(db: State<'_, Database>, id: i64) -> Result<(), AppError> {
    remove_indexed_folder(&db, id)
}

/// Starts a metadata scan for an indexed folder and returns the scan run
/// record. The scan runs in the background so the frontend stays responsive.
#[tauri::command]
pub fn start_scan(
    app: AppHandle,
    db: State<'_, Database>,
    manager: State<'_, ScanManager>,
    id: i64,
) -> Result<ScanRun, AppError> {
    if get_folder_path(&db, id)?.is_none() {
        return Err(AppError::FolderNotFound(format!("indexed folder {}", id)));
    }

    if let Some(latest) = latest_scan_run(&db, id)? {
        if latest.status == "running" || latest.status == "queued" {
            return Err(AppError::ScanAlreadyRunning(id));
        }
    }

    let run = create_scan_run(&db, id)?;
    let token = manager.register(run.id);
    let run_id = run.id;
    let app_for_thread = app.clone();

    std::thread::spawn(move || {
        let db = app_for_thread.state::<Database>();
        let manager = app_for_thread.state::<ScanManager>();
        match run_scan(&db, id, run_id, token, &app_for_thread) {
            Ok(summary) => {
                log::info!(
                    "scan {} finished: status={}, processed={}, indexed={}, warnings={}, errors={}",
                    run_id,
                    summary.status,
                    summary.files_processed,
                    summary.files_indexed,
                    summary.warning_count,
                    summary.error_count,
                );
            }
            Err(err) => {
                log::error!("scan {} failed: {}", run_id, err);
                let _ = finish_scan_run(&db, run_id, "failed", Some(&err.to_string()));
                let _ = update_folder_scan_status(&db, id, "failed", None);
            }
        }
        manager.remove(run_id);
    });

    Ok(run)
}

/// Requests cancellation of a running scan.
#[tauri::command]
pub fn cancel_scan(manager: State<'_, ScanManager>, run_id: i64) -> Result<bool, AppError> {
    Ok(manager.cancel(run_id))
}

/// Returns the latest scan run plus the current indexed file count for a
/// folder. The frontend polls this to show progress without receiving the
/// entire file list.
#[tauri::command]
pub fn get_scan_status(db: State<'_, Database>, id: i64) -> Result<Option<ScanStatus>, AppError> {
    match latest_scan_run(&db, id)? {
        Some(run) => {
            let file_count = file_count(&db, id)?;
            Ok(Some(ScanStatus { run, file_count }))
        }
        None => Ok(None),
    }
}

/// Lists all indexed files for a workspace.
#[tauri::command]
pub fn list_workspace_files_command(
    db: State<'_, Database>,
    id: i64,
) -> Result<Vec<FileEntry>, AppError> {
    list_workspace_files(&db, id)
}

/// Returns the details of a single indexed file.
#[tauri::command]
pub fn get_file_details_command(
    db: State<'_, Database>,
    id: i64,
) -> Result<Option<FileEntry>, AppError> {
    get_file_details(&db, id)
}

/// Returns the full scan history for a workspace, most recent first.
#[tauri::command]
pub fn list_scan_runs_command(db: State<'_, Database>, id: i64) -> Result<Vec<ScanRun>, AppError> {
    list_scan_runs(&db, id)
}

/// Reveals the indexed folder in the platform file manager.
#[tauri::command]
pub fn reveal_folder(app: AppHandle, path: String) -> Result<(), AppError> {
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| AppError::Io(std::io::Error::other(e)))?;
    Ok(())
}

/// Deprecated legacy command kept for backwards compatibility. New code should
/// use `list_indexed_folders_command`.
#[tauri::command]
pub fn list_workspaces(db: State<'_, Database>) -> Result<Vec<IndexedFolder>, AppError> {
    fetch_indexed_folders(&db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::tempdir;

    #[test]
    fn fetch_indexed_folders_returns_empty_for_new_db() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folders = fetch_indexed_folders(&db).expect("fetch folders");
        assert!(folders.is_empty());
    }
}
