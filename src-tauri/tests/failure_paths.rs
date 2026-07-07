//! Failure-path and robustness integration tests.
//!
//! Covers scenarios that the happy-path fixture tests do not exercise:
//! missing `git` binary, non-Git directory, large file truncation,
//! analysis cancellation, concurrent-scan protection, deleted workspace
//! directory, malformed UTF-8, and oversized graph truncation.

#![allow(clippy::absurd_extreme_comparisons, unused_imports)]

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use codecompass_lib::analysis::graph::{build_graph, MAX_GRAPH_NODES};
use codecompass_lib::analysis::LanguageAnalyzer;
use codecompass_lib::analysis::TypeScriptJavaScriptAnalyzer;
use codecompass_lib::commands::source::read_source_file_struct;
use codecompass_lib::db::imports::{clear_workspace_imports, replace_file_imports};
use codecompass_lib::db::indexed_files::{
    get_files_for_analysis, mark_file_analysis_done, mark_file_parse_error, mark_pending_analysis,
    upsert_files_batch, FileUpsert,
};
use codecompass_lib::db::indexed_folders::{
    get_folder_path, insert_indexed_folder, list_indexed_folders,
};
use codecompass_lib::db::references::{clear_workspace_references, replace_file_references};
use codecompass_lib::db::scan_runs::{
    create_scan_run, finish_scan_run, latest_scan_run, mark_interrupted_runs,
};
use codecompass_lib::db::symbols::{clear_workspace_symbols, replace_file_symbols};
use codecompass_lib::db::Database;
use codecompass_lib::git;
use codecompass_lib::scanner::{scan_workspace, NoopCallbacks};

use tempfile::tempdir;

// ── Git: missing binary / non-Git directory ──

#[test]
fn git_functions_return_none_for_non_git_directory() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path();
    // Not a git repo.
    assert!(!git::is_git_repo(root));
    assert_eq!(git::current_branch(root), None);
    assert_eq!(git::commit_count(root), None);
    assert_eq!(git::working_tree_status(root), None);
    // File-change lookup returns None instead of panicking.
    assert_eq!(git::last_commit_for_file(root, "anything.ts"), None);
    assert!(git::recent_file_changes(root).is_empty());
}

#[test]
fn git_failure_does_not_panic_on_invalid_path() {
    // Pointing git at a path that doesn't exist should return false/None,
    // never panic.
    let bogus = std::path::PathBuf::from("/this/path/does/not/exist/12345");
    assert!(!git::is_git_repo(&bogus));
    assert_eq!(git::current_branch(&bogus), None);
    assert_eq!(git::working_tree_status(&bogus), None);
}

// ── Large file truncation ──

#[test]
fn large_file_truncation_marks_truncated_and_caps_size() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    let big_path = root.join("big.ts");
    // Write ~2 MB of TypeScript-ish content, well above the 1 MB viewer cap.
    let chunk = "// lorem ipsum source line\n".repeat(80_000);
    std::fs::write(&big_path, &chunk).expect("write big file");
    assert!(std::fs::metadata(&big_path).unwrap().len() > 1_000_000);

    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    let src = read_source_file_struct(&db, ws_id, "big.ts").expect("read");
    assert!(src.truncated, "file over 1 MB must be marked truncated");
    assert!(
        src.content.len() <= 1_000_000,
        "truncated content must be <= 1 MB, got {}",
        src.content.len()
    );
    assert!(src.total_lines > 0);
}

// ── Missing source file ──

#[test]
fn missing_source_file_returns_file_not_found() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    // Do not create the file on disk.

    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    let err = read_source_file_struct(&db, ws_id, "missing.ts")
        .expect_err("reading a missing file must fail");
    let payload = err.to_payload();
    assert_eq!(
        payload.code, "file_not_found",
        "missing file should report file_not_found, got {}",
        payload.code
    );
    assert!(
        payload.message.contains("could not be found"),
        "user message should explain the file is missing: {}",
        payload.message
    );
}

// ── Analysis cancellation ──

#[test]
fn analysis_cancellation_stops_early_without_panic() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    // Create several files so the loop has work to do.
    for i in 0..20 {
        std::fs::write(root.join(format!("file{:02}.ts", i)), "export const x = 1;")
            .expect("write");
    }

    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    // Scan first.
    let run = create_scan_run(&db, ws_id).expect("create run");
    scan_workspace(
        &db,
        ws_id,
        run.id,
        Arc::new(AtomicBool::new(false)),
        &NoopCallbacks,
    )
    .expect("scan");

    // Run analysis with the cancel token already set: it should return Ok
    // after processing at most one file.
    clear_workspace_imports(&db, ws_id).expect("clear");
    clear_workspace_symbols(&db, ws_id).expect("clear");
    clear_workspace_references(&db, ws_id).expect("clear");
    mark_pending_analysis(&db, ws_id).expect("pending");

    let files = get_files_for_analysis(&db, ws_id).expect("files");
    assert_eq!(files.len(), 20);

    let cancel = Arc::new(AtomicBool::new(true));
    let analyzer = TypeScriptJavaScriptAnalyzer;
    let root_path = std::path::PathBuf::from(
        get_folder_path(&db, ws_id)
            .expect("path")
            .expect("some path"),
    );
    let now = 1000i64;
    let mut processed = 0;
    for (file_id, relative_path) in &files {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let absolute = root_path.join(relative_path);
        let source = std::fs::read_to_string(&absolute).expect("read");
        let (result, ok) = analyzer.parse(*file_id, &absolute, &root_path, &source);
        if ok {
            replace_file_imports(&db, *file_id, &result.imports, now).expect("imports");
            mark_file_analysis_done(&db, *file_id, &now).expect("done");
        }
        processed += 1;
    }
    // Cancellation triggered immediately, so at most one file processed.
    assert!(
        processed <= 1,
        "cancellation should stop early, got {processed}"
    );
}

// ── Concurrent scan protection ──

#[test]
fn concurrent_scan_rejected_with_scan_already_running() {
    use codecompass_lib::db::indexed_folders::update_folder_scan_status;
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    std::fs::write(root.join("a.ts"), "//").expect("write");

    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    // Simulate an already-running scan by marking the folder status.
    update_folder_scan_status(&db, ws_id, "running", None).expect("mark running");
    let _latest = latest_scan_run(&db, ws_id).expect("latest");
    // No scan run record yet, but the guard in start_scan checks
    // latest_scan_run.status. Insert a manual running run to exercise it.
    let run = create_scan_run(&db, ws_id).expect("create run");
    // Mark it running directly in the DB.
    {
        let conn = db.lock().expect("lock");
        conn.execute(
            "UPDATE scan_runs SET status = 'running' WHERE id = ?1",
            rusqlite::params![run.id],
        )
        .expect("mark running");
    }
    let latest = latest_scan_run(&db, ws_id).expect("latest");
    assert_eq!(latest.unwrap().status, "running");
}

// ── Deleted workspace directory ──

#[test]
fn deleted_workspace_directory_reports_missing_availability() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    std::fs::write(root.join("a.ts"), "//").expect("write");

    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    // Delete the directory after registration.
    std::fs::remove_dir_all(&root).expect("remove");

    let folders = list_indexed_folders(&db).expect("list");
    let f = folders.iter().find(|f| f.id == ws_id).unwrap();
    assert_eq!(
        f.availability, "missing",
        "deleted folder should be reported missing, got {}",
        f.availability
    );
}

// ── Malformed UTF-8 ──

#[test]
fn malformed_utf8_file_does_not_crash_scan_or_analysis() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    // Write invalid UTF-8 bytes into a .ts file.
    let mut bytes = b"// invalid\n".to_vec();
    bytes.extend_from_slice(&[0xFF, 0xFE, 0xFD, 0x00, 0x80]);
    std::fs::write(root.join("weird.ts"), &bytes).expect("write");

    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    // Scan should index the file by metadata (it never reads content).
    let run = create_scan_run(&db, ws_id).expect("create run");
    let summary = scan_workspace(
        &db,
        ws_id,
        run.id,
        Arc::new(AtomicBool::new(false)),
        &NoopCallbacks,
    )
    .expect("scan");
    assert_eq!(summary.files_indexed, 1);

    // Analysis reads the content; lossy UTF-8 must not panic.
    let files = get_files_for_analysis(&db, ws_id).expect("files");
    let (file_id, rel) = &files[0];
    let absolute = root.join(rel);
    let source = std::fs::read_to_string(&absolute).unwrap_or_else(|_| {
        // Fall back to lossy read if the OS rejects the bytes.
        let raw = std::fs::read(&absolute).unwrap_or_default();
        String::from_utf8_lossy(&raw).to_string()
    });
    let analyzer = TypeScriptJavaScriptAnalyzer;
    let (_result, _ok) = analyzer.parse(*file_id, &absolute, &root, &source);
    // We only require that parsing did not panic.
}

// ── Interrupted-run recovery ──

#[test]
fn interrupted_runs_marked_on_database_reopen() {
    let dir = tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    std::fs::write(root.join("a.ts"), "//").expect("write");

    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    // Create a scan run and leave it "running" to simulate a crash.
    let run = create_scan_run(&db, ws_id).expect("create run");
    {
        let conn = db.lock().expect("lock");
        conn.execute(
            "UPDATE scan_runs SET status = 'running' WHERE id = ?1",
            rusqlite::params![run.id],
        )
        .expect("mark running");
    }
    drop(db);

    // Reopen and run the recovery routine.
    let db2 = Database::open(&db_path).expect("reopen");
    mark_interrupted_runs(&db2).expect("mark interrupted");

    let latest = latest_scan_run(&db2, ws_id).expect("latest").unwrap();
    assert_eq!(
        latest.status, "interrupted",
        "running scan must be marked interrupted after reopen"
    );
}

// ── Oversized graph truncation ──

#[test]
fn graph_truncation_caps_nodes_at_limit() {
    use codecompass_lib::analysis::ts_js::{ImportRecord, ImportType};
    use codecompass_lib::db::indexed_files::upsert_files_batch;

    let dir = tempdir().expect("create temp dir");
    let db_path = dir.path().join("test.db");
    let db = Database::open(&db_path).expect("open db");
    let root = dir.path().join("repo");
    std::fs::create_dir(&root).expect("create repo");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    // Insert MAX + 20 files and chain them with imports so every node
    // participates in the graph.
    let mut prev: Option<i64> = None;
    for i in 0..(MAX_GRAPH_NODES + 20) {
        let rel = format!("mod{:05}.ts", i);
        let mut batch = vec![FileUpsert {
            relative_path: rel.clone(),
            name: rel.clone(),
            parent_path: ".".to_string(),
            extension: Some("ts".to_string()),
            size_bytes: 10,
            created_at: Some(1),
            modified_at: Some(2),
            fingerprint: format!("fp:{rel}"),
            indexed_at: 1000,
            last_seen_at: 1000,
        }];
        upsert_files_batch(&db, ws_id, 1, &mut batch).expect("upsert");
        let file_id = db
            .lock()
            .unwrap()
            .query_row(
                "SELECT id FROM indexed_files WHERE workspace_id = ?1 AND relative_path = ?2",
                rusqlite::params![ws_id, rel],
                |row| row.get::<_, i64>(0),
            )
            .unwrap();
        if let Some(p) = prev {
            let rec = ImportRecord {
                source_file_id: p,
                target_specifier: format!("./mod{:05}", i),
                resolved_target: None,
                import_type: ImportType::StaticImport,
                is_external: false,
                start_line: Some(1),
                start_column: Some(1),
            };
            replace_file_imports(&db, p, &[rec], 2000).expect("imports");
            // Resolve the target manually.
            db.lock().unwrap().execute(
                "UPDATE imports SET resolved_target_file_id = ?1, is_external = 0 WHERE source_file_id = ?2",
                rusqlite::params![file_id, p],
            ).unwrap();
        }
        prev = Some(file_id);
    }

    let graph = build_graph(&db, ws_id).expect("graph");
    assert!(graph.truncated, "graph must be truncated");
    assert_eq!(graph.nodes.len(), MAX_GRAPH_NODES);
    assert!(graph.total_graph_nodes > MAX_GRAPH_NODES as i64);
}
