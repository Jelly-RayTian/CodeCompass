use std::path::PathBuf;

use tauri::State;

use crate::db::indexed_folders::get_folder_path;
use crate::db::workspace_settings::{
    co_change_hotspots, get_settings, replace_git_changes, update_settings, CoChangePair,
    WorkspaceSettings,
};
use crate::db::Database;
use crate::error::AppError;
use crate::git;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfo {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub status: Option<String>,
    pub commit_count: Option<i64>,
    pub last_commit_short: Option<String>,
    pub last_commit_timestamp: Option<i64>,
    pub last_commit_message: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitFileInfo {
    pub last_commit: Option<String>,
    pub change_frequency: i64,
}

#[tauri::command]
pub fn get_git_info(db: State<'_, Database>, workspace_id: i64) -> Result<GitInfo, AppError> {
    let root = match get_folder_path(&db, workspace_id)? {
        Some(p) => PathBuf::from(p),
        None => {
            return Ok(GitInfo {
                is_repo: false,
                branch: None,
                status: None,
                commit_count: None,
                last_commit_short: None,
                last_commit_timestamp: None,
                last_commit_message: None,
            })
        }
    };

    if !git::is_git_repo(&root) {
        return Ok(GitInfo {
            is_repo: false,
            branch: None,
            status: None,
            commit_count: None,
            last_commit_short: None,
            last_commit_timestamp: None,
            last_commit_message: None,
        });
    }

    let info = GitInfo {
        is_repo: true,
        branch: git::current_branch(&root),
        status: git::working_tree_status(&root),
        commit_count: git::commit_count(&root),
        last_commit_short: git::last_commit_short(&root),
        last_commit_timestamp: git::last_commit_timestamp(&root),
        last_commit_message: git::last_commit_message(&root),
    };

    // Import file change history if git analysis is enabled.
    let settings = get_settings(&db, workspace_id)?;
    if settings.git_analysis_enabled {
        let raw_changes = git::recent_file_changes(&root);
        let changes: Vec<(String, String, i64)> = raw_changes
            .iter()
            .map(|(hash, ts, path)| (hash.clone(), path.clone(), *ts))
            .collect();
        let _ = replace_git_changes(&db, workspace_id, &changes);
    }

    Ok(info)
}

#[tauri::command]
pub fn get_file_git_info(
    db: State<'_, Database>,
    workspace_id: i64,
    relative_path: String,
) -> Result<GitFileInfo, AppError> {
    let root = match get_folder_path(&db, workspace_id)? {
        Some(p) => PathBuf::from(p),
        None => {
            return Ok(GitFileInfo {
                last_commit: None,
                change_frequency: 0,
            })
        }
    };

    let last_commit = git::last_commit_for_file(&root, &relative_path);

    let conn = db.lock()?;
    let freq: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM git_file_changes WHERE workspace_id = ?1 AND relative_path = ?2",
            rusqlite::params![workspace_id, relative_path],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(GitFileInfo {
        last_commit,
        change_frequency: freq,
    })
}

#[tauri::command]
pub fn get_workspace_settings(
    db: State<'_, Database>,
    workspace_id: i64,
) -> Result<WorkspaceSettings, AppError> {
    get_settings(&db, workspace_id)
}

#[tauri::command]
pub fn update_workspace_settings(
    db: State<'_, Database>,
    workspace_id: i64,
    git_analysis_enabled: Option<bool>,
    auto_reanalyze_enabled: Option<bool>,
) -> Result<(), AppError> {
    update_settings(
        &db,
        workspace_id,
        git_analysis_enabled,
        auto_reanalyze_enabled,
    )
}

#[tauri::command]
pub fn get_co_change_hotspots(
    db: State<'_, Database>,
    workspace_id: i64,
) -> Result<Vec<CoChangePair>, AppError> {
    co_change_hotspots(&db, workspace_id)
}
