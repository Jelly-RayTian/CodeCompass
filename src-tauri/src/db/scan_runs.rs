use rusqlite::{params, OptionalExtension};

use crate::error::AppError;
use crate::models::ScanRun;

use super::Database;

/// Creates a new scan run record in the `queued` state.
pub fn create_scan_run(db: &Database, workspace_id: i64) -> Result<ScanRun, AppError> {
    let now = now_epoch_secs();
    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO scan_runs (workspace_id, status, started_at, phase) \
         VALUES (?1, ?2, ?3, ?4)",
        params![workspace_id, "queued", now, "queued"],
    )?;
    let id = conn.last_insert_rowid();
    Ok(ScanRun {
        id,
        workspace_id,
        status: "queued".to_string(),
        started_at: now,
        completed_at: None,
        files_processed: 0,
        files_indexed: 0,
        warning_count: 0,
        error_count: 0,
        error_message: None,
        phase: Some("queued".to_string()),
    })
}

/// Transitions a scan run from queued to running.
pub fn start_scan_run(db: &Database, run_id: i64) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "UPDATE scan_runs SET status = 'running', phase = 'walking' WHERE id = ?1",
        params![run_id],
    )?;
    Ok(())
}

/// Updates the mutable progress fields of a scan run.
pub fn update_scan_progress(
    db: &Database,
    run_id: i64,
    files_processed: i64,
    files_indexed: i64,
    warning_count: i64,
    error_count: i64,
    phase: &str,
) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "UPDATE scan_runs SET \
         files_processed = ?1, files_indexed = ?2, warning_count = ?3, \
         error_count = ?4, phase = ?5 WHERE id = ?6",
        params![
            files_processed,
            files_indexed,
            warning_count,
            error_count,
            phase,
            run_id
        ],
    )?;
    Ok(())
}

/// Marks a scan run as completed, cancelled, failed, or interrupted.
pub fn finish_scan_run(
    db: &Database,
    run_id: i64,
    status: &str,
    error_message: Option<&str>,
) -> Result<(), AppError> {
    let now = now_epoch_secs();
    let conn = db.lock()?;
    conn.execute(
        "UPDATE scan_runs SET \
         status = ?1, completed_at = ?2, error_message = ?3 WHERE id = ?4",
        params![status, now, error_message, run_id],
    )?;
    Ok(())
}

/// Returns the most recent scan run for a workspace, if any.
pub fn latest_scan_run(db: &Database, workspace_id: i64) -> Result<Option<ScanRun>, AppError> {
    let conn = db.lock()?;
    conn.query_row(
        "SELECT id, workspace_id, status, started_at, completed_at, \
         files_processed, files_indexed, warning_count, error_count, \
         error_message, phase \
         FROM scan_runs \
         WHERE workspace_id = ?1 \
         ORDER BY started_at DESC LIMIT 1",
        params![workspace_id],
        |row| {
            Ok(ScanRun {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                status: row.get(2)?,
                started_at: row.get(3)?,
                completed_at: row.get(4)?,
                files_processed: row.get(5)?,
                files_indexed: row.get(6)?,
                warning_count: row.get(7)?,
                error_count: row.get(8)?,
                error_message: row.get(9)?,
                phase: row.get(10)?,
            })
        },
    )
    .optional()
    .map_err(AppError::Database)
}

/// Returns all scan runs for a workspace, most recent first.
pub fn list_scan_runs(db: &Database, workspace_id: i64) -> Result<Vec<ScanRun>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, status, started_at, completed_at, \
         files_processed, files_indexed, warning_count, error_count, \
         error_message, phase \
         FROM scan_runs \
         WHERE workspace_id = ?1 \
         ORDER BY started_at DESC",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(ScanRun {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            status: row.get(2)?,
            started_at: row.get(3)?,
            completed_at: row.get(4)?,
            files_processed: row.get(5)?,
            files_indexed: row.get(6)?,
            warning_count: row.get(7)?,
            error_count: row.get(8)?,
            error_message: row.get(9)?,
            phase: row.get(10)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)
}

/// On application startup, mark any scan runs that were still running when the
/// app last exited as interrupted.
pub fn mark_interrupted_runs(db: &Database) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "UPDATE scan_runs SET status = 'interrupted', phase = 'interrupted', \
         error_message = 'Application exited while scan was running' \
         WHERE status = 'running' OR status = 'queued'",
        [],
    )?;
    Ok(())
}

/// Returns the count of currently-present files for a workspace.
pub fn file_count(db: &Database, workspace_id: i64) -> Result<i64, AppError> {
    let conn = db.lock()?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM indexed_files WHERE workspace_id = ?1 AND is_present = 1",
        params![workspace_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

fn now_epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::indexed_folders::insert_indexed_folder;
    use crate::db::Database;
    use tempfile::tempdir;

    #[test]
    fn create_start_and_finish_round_trip() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");

        let folder = dir.path().join("scan_root");
        std::fs::create_dir(&folder).expect("create folder");
        let folder_id = insert_indexed_folder(&db, &folder)
            .expect("insert folder")
            .id;

        let run = create_scan_run(&db, folder_id).expect("create");
        assert_eq!(run.status, "queued");

        start_scan_run(&db, run.id).expect("start");
        let latest = latest_scan_run(&db, folder_id)
            .expect("latest")
            .expect("some run");
        assert_eq!(latest.status, "running");

        update_scan_progress(&db, run.id, 10, 9, 1, 0, "walking").expect("update");
        let latest = latest_scan_run(&db, folder_id)
            .expect("latest")
            .expect("some run");
        assert_eq!(latest.files_processed, 10);

        finish_scan_run(&db, run.id, "completed", None).expect("finish");
        let finished = latest_scan_run(&db, folder_id)
            .expect("latest")
            .expect("some run");
        assert_eq!(finished.status, "completed");
        assert!(finished.completed_at.is_some());
    }

    #[test]
    fn mark_interrupted_runs_on_startup() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");

        let folder = dir.path().join("scan_root");
        std::fs::create_dir(&folder).expect("create folder");
        let folder_id = insert_indexed_folder(&db, &folder)
            .expect("insert folder")
            .id;

        let run = create_scan_run(&db, folder_id).expect("create");
        start_scan_run(&db, run.id).expect("start");

        mark_interrupted_runs(&db).expect("mark interrupted");
        let run = latest_scan_run(&db, folder_id)
            .expect("latest")
            .expect("some run");
        assert_eq!(run.status, "interrupted");
    }
}
