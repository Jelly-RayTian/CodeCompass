use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

use crate::db::indexed_files::{
    is_supported_extension, mark_removed_files, upsert_files_batch, FileUpsert,
};
use crate::db::indexed_folders::update_folder_scan_status;
use crate::db::scan_runs::{finish_scan_run, start_scan_run, update_scan_progress};
use crate::db::Database;
use crate::error::AppError;
use crate::models::ScanProgressEvent;
use crate::platform::{normalize_existing_path, path_is_inside_or_equal};

const BATCH_SIZE: usize = 100;

/// Directories that Milestone 2 skips entirely during repository scanning.
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

/// Runs a metadata scan over a registered workspace root.
///
/// * `db` — the SQLite database.
/// * `workspace_id` — the workspace to scan.
/// * `run_id` — the `scan_runs` row that tracks this scan.
/// * `cancel` — set to `true` to stop the scan early.
/// * `app` — used to emit progress events to the frontend.
///
/// The scanner only traverses paths inside the canonical root, skips symbolic
/// links, ignores configured directories, only indexes supported extensions,
/// reads metadata only (never file contents), writes results to SQLite in
/// batches, and continues after recoverable per-file errors.
pub fn run_scan(
    db: &Database,
    workspace_id: i64,
    run_id: i64,
    cancel: Arc<AtomicBool>,
    app: &AppHandle,
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
    let scan_started_at = now_epoch_secs();
    emit_progress(
        app,
        ScanProgressEvent {
            run_id,
            workspace_id,
            status: "running".to_string(),
            files_processed: 0,
            files_indexed: 0,
            warning_count: 0,
            error_count: 0,
            phase: Some("walking".to_string()),
        },
    );

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
        if cancel.load(Ordering::Relaxed) {
            flush_batch(db, workspace_id, &mut batch)?;
            finish_scan_run(db, run_id, "cancelled", None)?;
            update_folder_scan_status(db, workspace_id, "idle", None)?;
            emit_progress(
                app,
                ScanProgressEvent {
                    run_id,
                    workspace_id,
                    status: "cancelled".to_string(),
                    files_processed,
                    files_indexed,
                    warning_count,
                    error_count,
                    phase: Some("cancelled".to_string()),
                },
            );
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
            log::warn!(
                "path traversal skipped: {} is outside {}",
                entry_path.display(),
                root.display()
            );
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
                log::warn!("metadata error for {}: {}", entry_path.display(), err);
            }
        }

        if batch.len() >= BATCH_SIZE {
            flush_batch(db, workspace_id, &mut batch)?;
            update_scan_progress(
                db,
                run_id,
                files_processed,
                files_indexed,
                warning_count,
                error_count,
                "walking",
            )?;
            emit_progress(
                app,
                ScanProgressEvent {
                    run_id,
                    workspace_id,
                    status: "running".to_string(),
                    files_processed,
                    files_indexed,
                    warning_count,
                    error_count,
                    phase: Some("walking".to_string()),
                },
            );
        }
    }

    flush_batch(db, workspace_id, &mut batch)?;

    let has_errors = error_count > 0;
    let final_status = if has_errors {
        "completed_with_errors"
    } else if warning_count > 0 {
        "completed_with_warnings"
    } else {
        "completed"
    };

    mark_removed_files(db, workspace_id, scan_started_at)?;

    let now = now_epoch_secs();
    finish_scan_run(db, run_id, final_status, None)?;
    update_folder_scan_status(db, workspace_id, "idle", Some(now))?;
    emit_progress(
        app,
        ScanProgressEvent {
            run_id,
            workspace_id,
            status: final_status.to_string(),
            files_processed,
            files_indexed,
            warning_count,
            error_count,
            phase: Some("finished".to_string()),
        },
    );

    Ok(ScanSummary {
        files_processed,
        files_indexed,
        warning_count,
        error_count,
        status: final_status.to_string(),
    })
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
    batch: &mut Vec<FileUpsert>,
) -> Result<(), AppError> {
    upsert_files_batch(db, workspace_id, batch)
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

fn emit_progress(app: &AppHandle, event: ScanProgressEvent) {
    let _ = app.emit("scan:progress", event);
}

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

    #[test]
    fn empty_folder_scan_records_zero_files() {
        let (_dir, db, folder_id) = setup();
        let run = create_scan_run(&db, folder_id).expect("create run");
        let cancel = Arc::new(AtomicBool::new(false));
        // We cannot provide an AppHandle in a unit test, so we exercise the
        // core traversal logic through a command-level integration test and
        // verify the DAO behavior here directly.
        let summary = run_scan_with_dummy_emit(&db, folder_id, run.id, cancel);
        assert_eq!(summary.files_indexed, 0);
        assert_eq!(summary.status, "completed");
    }

    fn run_scan_with_dummy_emit(
        db: &Database,
        workspace_id: i64,
        run_id: i64,
        cancel: Arc<AtomicBool>,
    ) -> ScanSummary {
        // Re-implements the core scan without event emission so unit tests do
        // not need an AppHandle.
        let root = normalize_existing_path(Path::new(
            &crate::db::indexed_folders::get_folder_path(db, workspace_id)
                .expect("get path")
                .expect("path exists"),
        ))
        .expect("normalize");
        start_scan_run(db, run_id).expect("start");
        let scan_started_at = now_epoch_secs();
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
            if cancel.load(Ordering::Relaxed) {
                upsert_files_batch(db, workspace_id, &mut batch).expect("flush");
                finish_scan_run(db, run_id, "cancelled", None).expect("finish");
                update_folder_scan_status(db, workspace_id, "idle", None).expect("update status");
                return ScanSummary {
                    files_processed,
                    files_indexed,
                    warning_count,
                    error_count,
                    status: "cancelled".to_string(),
                };
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
                    log::warn!("metadata error: {}", err);
                }
            }
            if batch.len() >= BATCH_SIZE {
                upsert_files_batch(db, workspace_id, &mut batch).expect("flush");
                update_scan_progress(
                    db,
                    run_id,
                    files_processed,
                    files_indexed,
                    warning_count,
                    error_count,
                    "walking",
                )
                .expect("progress");
            }
        }

        upsert_files_batch(db, workspace_id, &mut batch).expect("flush");
        mark_removed_files(db, workspace_id, scan_started_at).expect("mark removed");
        let final_status = if error_count > 0 {
            "completed_with_errors"
        } else if warning_count > 0 {
            "completed_with_warnings"
        } else {
            "completed"
        };
        let now = now_epoch_secs();
        finish_scan_run(db, run_id, final_status, None).expect("finish");
        update_folder_scan_status(db, workspace_id, "idle", Some(now)).expect("update status");
        ScanSummary {
            files_processed,
            files_indexed,
            warning_count,
            error_count,
            status: final_status.to_string(),
        }
    }

    #[test]
    fn nested_supported_files_are_indexed() {
        let (dir, db, folder_id) = setup();
        let sub = dir.path().join("scan_root/src");
        std::fs::create_dir_all(&sub).expect("create subdir");
        std::fs::write(sub.join("main.ts"), "console.log(1)").expect("write file");
        std::fs::write(sub.join("lib.tsx"), "export const A = 1").expect("write file");

        let run = create_scan_run(&db, folder_id).expect("create run");
        let cancel = Arc::new(AtomicBool::new(false));
        let summary = run_scan_with_dummy_emit(&db, folder_id, run.id, cancel);
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

        let run = create_scan_run(&db, folder_id).expect("create run");
        let cancel = Arc::new(AtomicBool::new(false));
        let summary = run_scan_with_dummy_emit(&db, folder_id, run.id, cancel);
        assert_eq!(summary.files_indexed, 1);
    }

    #[test]
    fn unsupported_files_are_skipped() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/app.ts"), "").expect("write ts");
        std::fs::write(dir.path().join("scan_root/logo.png"), [0u8, 1, 2]).expect("write png");
        std::fs::write(dir.path().join("scan_root/readme.md"), "#").expect("write md");

        let run = create_scan_run(&db, folder_id).expect("create run");
        let cancel = Arc::new(AtomicBool::new(false));
        let summary = run_scan_with_dummy_emit(&db, folder_id, run.id, cancel);
        assert_eq!(summary.files_indexed, 1);
    }

    #[test]
    fn symlinks_are_skipped() {
        let (dir, db, folder_id) = setup();
        let real = dir.path().join("real.ts");
        std::fs::write(&real, "//").expect("write file");
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

        let run = create_scan_run(&db, folder_id).expect("create run");
        let cancel = Arc::new(AtomicBool::new(false));
        let summary = run_scan_with_dummy_emit(&db, folder_id, run.id, cancel);
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
        let run = create_scan_run(&db, folder_id).expect("create run");
        let cancel = Arc::new(AtomicBool::new(true));
        let summary = run_scan_with_dummy_emit(&db, folder_id, run.id, cancel);
        assert_eq!(summary.status, "cancelled");
    }

    #[test]
    fn cancellation_preserves_previous_index() {
        let (dir, db, folder_id) = setup();
        std::fs::write(dir.path().join("scan_root/old.ts"), "// old").expect("write");

        let run1 = create_scan_run(&db, folder_id).expect("create");
        let cancel = Arc::new(AtomicBool::new(false));
        run_scan_with_dummy_emit(&db, folder_id, run1.id, cancel);

        // Delete the file, then start a second scan and cancel it before it
        // would mark the file as removed.
        std::fs::remove_file(dir.path().join("scan_root/old.ts")).expect("remove");
        let run2 = create_scan_run(&db, folder_id).expect("create");
        let cancel = Arc::new(AtomicBool::new(true));
        run_scan_with_dummy_emit(&db, folder_id, run2.id, cancel);

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
        run_scan_with_dummy_emit(&db, folder_id, run1.id, Arc::new(AtomicBool::new(false)));

        // Sleep to ensure the second scan has a strictly greater start
        // timestamp, because the database stores Unix epoch seconds.
        std::thread::sleep(std::time::Duration::from_secs(1));

        // Modify a, remove b, add c.
        std::fs::write(&a, "// a modified").expect("modify a");
        std::fs::remove_file(&b).expect("remove b");
        let c = dir.path().join("scan_root/c.ts");
        std::fs::write(&c, "// c").expect("add c");

        let run2 = create_scan_run(&db, folder_id).expect("create");
        run_scan_with_dummy_emit(&db, folder_id, run2.id, Arc::new(AtomicBool::new(false)));

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
}
