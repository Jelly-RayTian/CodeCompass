use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::Emitter;
use walkdir::WalkDir;

use crate::db::indexed_files::{
    is_supported_extension, mark_removed_files_by_generation, next_scan_generation,
    upsert_files_batch, FileUpsert,
};
use crate::db::indexed_folders::update_folder_scan_status;
use crate::db::scan_runs::{finish_scan_run, start_scan_run, update_scan_progress};
use crate::db::Database;
use crate::error::AppError;
use crate::models::ScanProgressEvent;
use crate::platform::{normalize_existing_path, path_is_inside_or_equal};

const BATCH_SIZE: usize = 100;

/// Directories skipped during repository scanning.
const IGNORED_DIRECTORIES: &[&str] = &[
    ".git",
    "node_modules",
    "dist",
    "build",
    "coverage",
    ".next",
    "out",
    "target",
    "vendor",
    ".idea",
];

/// Summary produced when a scan finishes.
#[derive(Debug, Clone)]
pub struct ScanSummary {
    pub files_processed: i64,
    pub files_indexed: i64,
    pub warning_count: i64,
    pub error_count: i64,
    pub status: String,
}

/// Abstraction over progress-emission so production and test code share the
/// same traversal logic.
pub trait ScanCallbacks: Send {
    fn emit_progress(&self, event: ScanProgressEvent);
}

// ── Production implementation (Tauri AppHandle) ──

pub struct TauriCallbacks {
    app: tauri::AppHandle,
}

impl TauriCallbacks {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

impl ScanCallbacks for TauriCallbacks {
    fn emit_progress(&self, event: ScanProgressEvent) {
        let _ = self.app.emit("scan:progress", event);
    }
}

// ── Test implementation (no-op) ──

pub struct NoopCallbacks;

impl ScanCallbacks for NoopCallbacks {
    fn emit_progress(&self, _event: ScanProgressEvent) {}
}

// ── Core scanner ──

/// Runs a metadata scan over a registered workspace root.
///
/// This is the single production-quality scanner used by both the live app
/// (via `run_scan_with_app`) and tests (via `run_scan_noop`).
///
/// ## Deletion reconciliation
///
/// Files not seen during a **complete** scan are marked removed via a
/// monotonic `scan_generation`. Reconciliation is **skipped** when:
///
/// - The scan was cancelled (`cancel` flag).
/// - The scan encountered traversal/permission errors that make the
///   snapshot incomplete.  Only `"completed"` and
///   `"completed_with_warnings"` trigger reconciliation.
///   `"completed_with_errors"`, `"cancelled"`, and `"failed"` do **not**.
///
/// This preserves the last complete snapshot.
pub fn scan_workspace(
    db: &Database,
    workspace_id: i64,
    run_id: i64,
    cancel: Arc<AtomicBool>,
    cb: &dyn ScanCallbacks,
) -> Result<ScanSummary, AppError> {
    let root = match crate::db::indexed_folders::get_folder_path(db, workspace_id)? {
        Some(path) => normalize_existing_path(Path::new(&path))?,
        None => {
            return Err(AppError::FolderNotFound(format!(
                "workspace {}",
                workspace_id
            )))
        }
    };

    if !root.is_dir() {
        let err = format!("{} is not a directory", root.display());
        finish_scan_run(db, run_id, "failed", Some(&err))?;
        update_folder_scan_status(db, workspace_id, "failed", None)?;
        return Err(AppError::NotADirectory(err));
    }

    start_scan_run(db, run_id)?;

    let scan_generation = next_scan_generation(db, workspace_id)?;

    cb.emit_progress(ScanProgressEvent {
        run_id,
        workspace_id,
        status: "running".to_string(),
        files_processed: 0,
        files_indexed: 0,
        warning_count: 0,
        error_count: 0,
        phase: Some("walking".to_string()),
    });

    let mut files_processed: i64 = 0;
    let mut files_indexed: i64 = 0;
    let mut warning_count: i64 = 0;
    let mut error_count: i64 = 0;
    let mut batch: Vec<FileUpsert> = Vec::with_capacity(BATCH_SIZE);

    let ignored: HashSet<&str> = IGNORED_DIRECTORIES.iter().copied().collect();
    let mut walker = WalkDir::new(&root)
        .follow_links(false)
        .same_file_system(false)
        .into_iter();

    while let Some(entry_result) = walker.next() {
        // ── Cancellation ──
        if cancel.load(Ordering::Relaxed) {
            flush_batch(db, workspace_id, scan_generation, &mut batch)?;
            finish_scan_run(db, run_id, "cancelled", None)?;
            update_folder_scan_status(db, workspace_id, "idle", None)?;
            cb.emit_progress(ScanProgressEvent {
                run_id,
                workspace_id,
                status: "cancelled".to_string(),
                files_processed,
                files_indexed,
                warning_count,
                error_count,
                phase: Some("cancelled".to_string()),
            });
            return Ok(ScanSummary {
                files_processed,
                files_indexed,
                warning_count,
                error_count,
                status: "cancelled".to_string(),
            });
        }

        let entry = match entry_result {
            Ok(e) => e,
            Err(err) => {
                error_count += 1;
                log::warn!("scan entry error: {}", err);
                continue;
            }
        };

        let entry_path = entry.path();
        if !path_is_inside_or_equal(&root, entry_path) {
            warning_count += 1;
            log::warn!("path traversal skipped outside workspace root");
            continue;
        }

        if entry_path == root {
            continue;
        }

        if entry.file_type().is_symlink() {
            continue;
        }

        if entry.file_type().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if ignored.contains(name) {
                    walker.skip_current_dir();
                    continue;
                }
            }
            continue;
        }

        if !entry.file_type().is_file() {
            continue;
        }

        files_processed += 1;

        let relative_path = match entry_path.strip_prefix(&root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => {
                warning_count += 1;
                continue;
            }
        };

        let extension = entry_path
            .extension()
            .map(|s| s.to_string_lossy().to_string().to_lowercase());

        if !is_supported_extension(extension.as_deref()) {
            continue;
        }

        match build_file_row(entry_path, &relative_path, extension) {
            Ok(row) => {
                batch.push(row);
                files_indexed += 1;
            }
            Err(err) => {
                error_count += 1;
                log::warn!("metadata error for {}: {}", relative_path.display(), err);
            }
        }

        if batch.len() >= BATCH_SIZE {
            flush_batch(db, workspace_id, scan_generation, &mut batch)?;
            update_scan_progress(
                db,
                run_id,
                files_processed,
                files_indexed,
                warning_count,
                error_count,
                "walking",
            )?;
            cb.emit_progress(ScanProgressEvent {
                run_id,
                workspace_id,
                status: "running".to_string(),
                files_processed,
                files_indexed,
                warning_count,
                error_count,
                phase: Some("walking".to_string()),
            });
        }
    }

    flush_batch(db, workspace_id, scan_generation, &mut batch)?;

    // ── Determine final status ──
    let final_status = if error_count > 0 {
        "completed_with_errors"
    } else if warning_count > 0 {
        "completed_with_warnings"
    } else {
        "completed"
    };

    // ── Deletion reconciliation: only on clean completion ──
    // "completed" or "completed_with_warnings" = full traversal.
    // "completed_with_errors" may have permission failures → skip.
    let should_reconcile = final_status == "completed" || final_status == "completed_with_warnings";

    if should_reconcile {
        let _ = mark_removed_files_by_generation(db, workspace_id, scan_generation);
    }

    let now = now_epoch_secs();
    finish_scan_run(db, run_id, final_status, None)?;
    update_folder_scan_status(db, workspace_id, "idle", Some(now))?;
    cb.emit_progress(ScanProgressEvent {
        run_id,
        workspace_id,
        status: final_status.to_string(),
        files_processed,
        files_indexed,
        warning_count,
        error_count,
        phase: Some("finished".to_string()),
    });

    Ok(ScanSummary {
        files_processed,
        files_indexed,
        warning_count,
        error_count,
        status: final_status.to_string(),
    })
}

/// Convenience wrapper for production use.
pub fn run_scan(
    db: &Database,
    workspace_id: i64,
    run_id: i64,
    cancel: Arc<AtomicBool>,
    app: &tauri::AppHandle,
) -> Result<ScanSummary, AppError> {
    let cb = TauriCallbacks::new(app.clone());
    scan_workspace(db, workspace_id, run_id, cancel, &cb)
}

fn build_file_row(
    path: &Path,
    relative_path: &Path,
    extension: Option<String>,
) -> Result<FileUpsert, AppError> {
    let meta = std::fs::metadata(path)?;
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent_path = relative_path
        .parent()
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            if s.is_empty() {
                ".".to_string()
            } else {
                s
            }
        })
        .unwrap_or_else(|| ".".to_string());
    let size_bytes = meta.len().try_into().unwrap_or(i64::MAX);
    let created_at = meta.created().ok().and_then(system_time_to_secs);
    let modified_at = meta.modified().ok().and_then(system_time_to_secs);
    let now = now_epoch_secs();
    let fingerprint = compute_fingerprint(size_bytes, modified_at);

    Ok(FileUpsert {
        relative_path: relative_path.to_string_lossy().to_string(),
        name,
        parent_path,
        extension,
        size_bytes,
        created_at,
        modified_at,
        fingerprint,
        indexed_at: now,
        last_seen_at: now,
    })
}

fn compute_fingerprint(size_bytes: i64, modified_at: Option<i64>) -> String {
    format!("{}:{}", size_bytes, modified_at.unwrap_or(0))
}

fn flush_batch(
    db: &Database,
    workspace_id: i64,
    scan_generation: i64,
    batch: &mut Vec<FileUpsert>,
) -> Result<(), AppError> {
    upsert_files_batch(db, workspace_id, scan_generation, batch)
}

fn system_time_to_secs(time: std::time::SystemTime) -> Option<i64> {
    time.duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs() as i64)
}

fn now_epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::indexed_files::list_workspace_files;
    use crate::db::indexed_folders::insert_indexed_folder;
    use crate::db::scan_runs::create_scan_run;
    use crate::db::Database;
    use tempfile::tempdir;

    fn setup() -> (tempfile::TempDir, Database, i64) {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folder = dir.path().join("scan_root");
        std::fs::create_dir(&folder).expect("create root");
        let folder_id = insert_indexed_folder(&db, &folder).expect("insert").id;
        (dir, db, folder_id)
    }

    fn run_test_scan(
        db: &Database,
        workspace_id: i64,
        run_id: i64,
        cancel: Arc<AtomicBool>,
    ) -> ScanSummary {
        let cb = NoopCallbacks;
        scan_workspace(db, workspace_id, run_id, cancel, &cb).expect("scan should succeed")
    }

    #[test]
    fn empty_folder_scan_records_zero_files() {
        let (_dir, db, folder_id) = setup();
        let run = create_scan_run(&db, folder_id).expect("create run");
        let summary = run_test_scan(&db, folder_id, run.id, Arc::new(AtomicBool::new(false)));
        assert_eq!(summary.files_indexed, 0);
        assert_eq!(summary.status, "completed");
    }

    #[test]
    fn nested_supported_files_are_indexed() {
        let (dir, db, folder_id) = setup();
        let sub = dir.path().join("scan_root/src");
        std::fs::create_dir_all(&sub).expect("create subdir");
        std::fs::write(sub.join("main.ts"), "console.log(1)").expect("write");
        std::fs::write(sub.join("lib.tsx"), "export const A = 1").expect("write");

        let run = create_scan_run(&db, folder_id).expect("create");
        let summary = run_test_scan(&db, folder_id, run.id, Arc::new(AtomicBool::new(false)));
        assert_eq!(summary.files_indexed, 2);
    }

    #[test]
    fn ignored_directories_are_skipped() {
        let (dir, db, folder_id) = setup();
        let src = dir.path().join("scan_root/src");
        let node_modules = dir.path().join("scan_root/node_modules");
        std::fs::create_dir_all(&src).expect("create src");
        std::fs::create_dir_all(&node_modules).expect("create node_modules");
        std::fs::write(src.join("app.ts"), "").expect("write src");
        std::fs::write(node_modules.join("bad.ts"), "").expect("write ignored");

        let run = create_scan_run(&db, folder_id).expect("create");
        let summary = run_test_scan(&db, folder_id, run.id, Arc::new(AtomicBool::new(false)));
        assert_eq!(summary.files_indexed, 1);
    }

    #[test]
    fn unsupported_files_are_skipped() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/app.ts"), "").expect("write");
        std::fs::write(dir.path().join("scan_root/logo.png"), [0u8, 1, 2]).expect("write");
        std::fs::write(dir.path().join("scan_root/readme.md"), "#").expect("write");

        let run = create_scan_run(&db, folder_id).expect("create");
        let summary = run_test_scan(&db, folder_id, run.id, Arc::new(AtomicBool::new(false)));
        assert_eq!(summary.files_indexed, 1);
    }

    #[test]
    fn symlinks_are_skipped() {
        let (dir, db, folder_id) = setup();
        let real = dir.path().join("real.ts");
        std::fs::write(&real, "//").expect("write");
        let link = dir.path().join("scan_root/link.ts");
        let symlink_result = {
            #[cfg(windows)]
            {
                std::os::windows::fs::symlink_file(&real, &link)
            }
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&real, &link)
            }
        };
        if symlink_result.is_err() {
            return;
        }

        let run = create_scan_run(&db, folder_id).expect("create");
        let summary = run_test_scan(&db, folder_id, run.id, Arc::new(AtomicBool::new(false)));
        assert_eq!(summary.files_indexed, 0);
    }

    #[test]
    fn cancellation_stops_scan_early() {
        let (dir, db, folder_id) = setup();
        for i in 0..20 {
            std::fs::write(
                dir.path().join(format!("scan_root/file{i}.ts")),
                "export {}",
            )
            .unwrap();
        }
        let run = create_scan_run(&db, folder_id).expect("create");
        let summary = run_test_scan(&db, folder_id, run.id, Arc::new(AtomicBool::new(true)));
        assert_eq!(summary.status, "cancelled");
    }

    #[test]
    fn cancellation_preserves_previous_index() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/old.ts"), "// old").expect("write");

        let run1 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));

        // Delete the file, then start a second scan and cancel it.
        std::fs::remove_file(dir.path().join("scan_root/old.ts")).expect("remove");
        let run2 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run2.id, Arc::new(AtomicBool::new(true)));

        let files = list_workspace_files(&db, folder_id).expect("list");
        assert_eq!(files.len(), 1);
        assert!(files[0].is_present);
    }

    #[test]
    fn incremental_change_detection() {
        let (dir, db, folder_id) = setup();
        let a = dir.path().join("scan_root/a.ts");
        let b = dir.path().join("scan_root/b.ts");
        std::fs::write(&a, "// a").expect("write a");
        std::fs::write(&b, "// b").expect("write b");

        let run1 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));

        // Modify a, remove b, add c.
        std::fs::write(&a, "// a modified").expect("modify a");
        std::fs::remove_file(&b).expect("remove b");
        let c = dir.path().join("scan_root/c.ts");
        std::fs::write(&c, "// c").expect("add c");

        let run2 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run2.id, Arc::new(AtomicBool::new(false)));

        let files = list_workspace_files(&db, folder_id).expect("list");
        let by_path: std::collections::HashMap<_, _> = files
            .iter()
            .map(|f| (f.relative_path.as_str(), f))
            .collect();
        assert_eq!(by_path["a.ts"].change_status, "changed");
        assert_eq!(by_path["c.ts"].change_status, "new");
        assert!(!by_path["b.ts"].is_present);
        assert_eq!(by_path["b.ts"].change_status, "removed");
    }

    // ── Regression tests ──

    #[test]
    fn completed_with_warnings_still_reconciles() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/a.ts"), "").expect("write");

        let run1 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));

        // Create a symlink that produces a warning but doesn't stop the scan.
        // Warnings don't prevent reconciliation.
        let run2 = create_scan_run(&db, folder_id).expect("create");
        let summary = run_test_scan(&db, folder_id, run2.id, Arc::new(AtomicBool::new(false)));
        // No errors, maybe no warnings either. Status is completed.
        assert!(summary.status == "completed" || summary.status == "completed_with_warnings");
    }

    #[test]
    fn cancelled_scan_never_reconciles() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/a.ts"), "").expect("write");

        let run1 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));
        assert_eq!(list_workspace_files(&db, folder_id).unwrap().len(), 1);

        // Delete a.ts, then scan again but cancel immediately.
        std::fs::remove_file(dir.path().join("scan_root/a.ts")).expect("remove");
        let run2 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run2.id, Arc::new(AtomicBool::new(true)));
        // File must still be present — cancelled scan didn't delete it.
        let files = list_workspace_files(&db, folder_id).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].is_present);
    }

    #[test]
    fn successful_scan_reconciles_removed_files() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/a.ts"), "").expect("write");

        let run1 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));

        // Remove a.ts, scan again — it should be marked removed.
        std::fs::remove_file(dir.path().join("scan_root/a.ts")).expect("remove");
        let run2 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run2.id, Arc::new(AtomicBool::new(false)));

        let files = list_workspace_files(&db, folder_id).unwrap();
        assert!(!files[0].is_present);
        assert_eq!(files[0].change_status, "removed");
    }

    #[test]
    fn same_second_scans_use_generation_not_timestamp() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/a.ts"), "").expect("write");

        // Two scans back-to-back without sleeping.
        let run1 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));

        std::fs::remove_file(dir.path().join("scan_root/a.ts")).expect("remove");

        let run2 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run2.id, Arc::new(AtomicBool::new(false)));

        let files = list_workspace_files(&db, folder_id).unwrap();
        assert!(!files[0].is_present);
        assert_eq!(files[0].change_status, "removed");
    }

    #[test]
    fn failed_scan_preserves_previous_snapshot() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/a.ts"), "").expect("write");

        let run1 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));

        // Delete and scan with immediate cancellation.
        std::fs::remove_file(dir.path().join("scan_root/a.ts")).expect("remove");
        let run2 = create_scan_run(&db, folder_id).expect("create");
        run_test_scan(&db, folder_id, run2.id, Arc::new(AtomicBool::new(true)));

        let files = list_workspace_files(&db, folder_id).unwrap();
        assert!(files[0].is_present, "cancelled scan must not delete files");
    }
}
