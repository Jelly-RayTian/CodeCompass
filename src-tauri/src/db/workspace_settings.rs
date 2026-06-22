use rusqlite::params;

use crate::db::Database;
use crate::error::AppError;

/// Per-workspace settings for Git analysis and auto-reanalysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSettings {
    pub workspace_id: i64,
    pub git_analysis_enabled: bool,
    pub auto_reanalyze_enabled: bool,
}

pub fn get_settings(db: &Database, workspace_id: i64) -> Result<WorkspaceSettings, AppError> {
    let conn = db.lock()?;
    let (git_enabled, auto_enabled): (i64, i64) = conn.query_row(
        "SELECT git_analysis_enabled, auto_reanalyze_enabled FROM workspaces WHERE id = ?1",
        params![workspace_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok(WorkspaceSettings {
        workspace_id,
        git_analysis_enabled: git_enabled != 0,
        auto_reanalyze_enabled: auto_enabled != 0,
    })
}

pub fn update_settings(
    db: &Database,
    workspace_id: i64,
    git_analysis_enabled: Option<bool>,
    auto_reanalyze_enabled: Option<bool>,
) -> Result<(), AppError> {
    let conn = db.lock()?;

    if let Some(val) = git_analysis_enabled {
        conn.execute(
            "UPDATE workspaces SET git_analysis_enabled = ?1 WHERE id = ?2",
            params![val as i64, workspace_id],
        )?;
    }
    if let Some(val) = auto_reanalyze_enabled {
        conn.execute(
            "UPDATE workspaces SET auto_reanalyze_enabled = ?1 WHERE id = ?2",
            params![val as i64, workspace_id],
        )?;
    }
    Ok(())
}

/// Batch-insert file changes from Git history.
pub fn replace_git_changes(
    db: &Database,
    workspace_id: i64,
    changes: &[(String, String, i64)],
) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM git_file_changes WHERE workspace_id = ?1",
        params![workspace_id],
    )?;

    if changes.is_empty() {
        return Ok(());
    }

    let mut stmt = conn.prepare(
        "INSERT INTO git_file_changes \
         (workspace_id, commit_hash, relative_path, timestamp) \
         VALUES (?1, ?2, ?3, ?4)",
    )?;
    for (hash, path, ts) in changes {
        stmt.execute(params![workspace_id, hash, path, ts])?;
    }
    Ok(())
}

/// Hotspot: files that frequently changed together (top 10 pairs).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoChangePair {
    pub file_a: String,
    pub file_b: String,
    pub together_count: i64,
}

pub fn co_change_hotspots(db: &Database, workspace_id: i64) -> Result<Vec<CoChangePair>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT a.relative_path, b.relative_path, COUNT(*) as cnt \
         FROM git_file_changes a \
         JOIN git_file_changes b ON a.commit_hash = b.commit_hash AND a.relative_path < b.relative_path \
         WHERE a.workspace_id = ?1 AND b.workspace_id = ?1 \
         GROUP BY 1, 2 \
         ORDER BY cnt DESC \
         LIMIT 10",
    )?;
    let rows = stmt
        .query_map(params![workspace_id], |row| {
            Ok(CoChangePair {
                file_a: row.get(0)?,
                file_b: row.get(1)?,
                together_count: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
