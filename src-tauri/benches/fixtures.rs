//! Reproducible benchmark fixtures.
//!
//! Generates synthetic TypeScript projects of a configurable size in a
//! temporary directory at runtime. No fixture data is committed to the
//! repository. Each file has deterministic, varied content so that the
//! scanner, analyser, and graph builder all exercise realistic paths.
//!
//! Determinism: file names and contents are derived solely from indices,
//! so two runs at the same size produce identical work.

#![allow(dead_code)]

use std::path::PathBuf;

use codecompass_lib::analysis::LanguageAnalyzer;
use codecompass_lib::analysis::TypeScriptJavaScriptAnalyzer;
use codecompass_lib::db::imports::replace_file_imports;
use codecompass_lib::db::indexed_files::{get_files_for_analysis, mark_file_analysis_done};
use codecompass_lib::db::indexed_folders::insert_indexed_folder;
use codecompass_lib::db::references::replace_file_references;
use codecompass_lib::db::scan_runs::create_scan_run;
use codecompass_lib::db::symbols::replace_file_symbols;
use codecompass_lib::db::Database;
use codecompass_lib::scanner::{scan_workspace, NoopCallbacks};

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// A generated fixture: the temp dir keeps files alive while the fixture
/// is in use.
pub struct Fixture {
    pub _dir: tempfile::TempDir,
    pub root: PathBuf,
    pub db: Database,
    pub workspace_id: i64,
    pub file_count: usize,
}

/// Creates a fixture with `n` `.ts` files arranged in `src/` with a
/// mix of cross-imports so the dependency graph is non-trivial.
pub fn make_fixture(n: usize) -> Fixture {
    let dir = tempfile::tempdir().expect("create temp dir");
    let root = dir.path().join("repo");
    let src = root.join("src");
    std::fs::create_dir_all(&src).expect("create src");

    for i in 0..n {
        let rel = format!("src/mod{:05}.ts", i);
        let path = root.join(&rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        // Deterministic, varied content. Every 7th file imports its
        // predecessor so ~14% of files have internal imports.
        let mut content = format!(
            "// auto-generated module {i}\nexport function fn{i}(x: number): number {{\n  return x + {i};\n}}\n"
        );
        if i > 0 && i % 7 == 0 {
            content.push_str(&format!(
                "import {{ fn{} }} from './mod{:05}';\n",
                i - 1,
                i - 1
            ));
            content.push_str(&format!("export const use{} = fn{}({i});\n", i, i - 1));
        }
        if i % 13 == 0 {
            // Add a class to vary symbol kinds.
            content.push_str(&format!(
                "export class Class{i} {{ method{i}(): string {{ return '{i}'; }} }}\n"
            ));
        }
        std::fs::write(&path, content).expect("write file");
    }

    let db_path = dir.path().join("bench.db");
    let db = Database::open(&db_path).expect("open db");
    let ws_id = insert_indexed_folder(&db, &root).expect("insert").id;

    Fixture {
        _dir: dir,
        root,
        db,
        workspace_id: ws_id,
        file_count: n,
    }
}

/// Runs a metadata scan and returns the elapsed time in microseconds.
pub fn bench_scan(fx: &Fixture) -> u128 {
    let run = create_scan_run(&fx.db, fx.workspace_id).expect("create run");
    let start = std::time::Instant::now();
    scan_workspace(
        &fx.db,
        fx.workspace_id,
        run.id,
        Arc::new(AtomicBool::new(false)),
        &NoopCallbacks,
    )
    .expect("scan");
    start.elapsed().as_micros()
}

/// Runs the full analysis pass over all pending files and returns the
/// elapsed time in microseconds.
pub fn bench_analyze(fx: &Fixture) -> u128 {
    let analyzer = TypeScriptJavaScriptAnalyzer;
    let now = 1000i64;
    let files = get_files_for_analysis(&fx.db, fx.workspace_id).expect("files");
    let start = std::time::Instant::now();
    for (file_id, relative_path) in &files {
        let absolute = fx.root.join(relative_path);
        let source = std::fs::read_to_string(&absolute).unwrap_or_default();
        let (result, ok) = analyzer.parse(*file_id, &absolute, &fx.root, &source);
        if ok {
            replace_file_imports(&fx.db, *file_id, &result.imports, now).expect("imports");
            let symbols = codecompass_lib::analysis::symbols::extract_symbols(&source, &absolute);
            replace_file_symbols(&fx.db, *file_id, fx.workspace_id, &symbols, now)
                .expect("symbols");
            let refs =
                codecompass_lib::analysis::references::extract_references(&source, &absolute);
            replace_file_references(&fx.db, *file_id, fx.workspace_id, &refs, now).expect("refs");
            mark_file_analysis_done(&fx.db, *file_id, &now).expect("done");
        }
    }
    start.elapsed().as_micros()
}

/// Runs graph construction and returns the elapsed time in microseconds.
pub fn bench_graph(fx: &Fixture) -> u128 {
    let start = std::time::Instant::now();
    let _graph =
        codecompass_lib::analysis::graph::build_graph(&fx.db, fx.workspace_id).expect("graph");
    start.elapsed().as_micros()
}

/// Touches every Nth file's mtime+size to simulate edits, then rescans.
pub fn bench_modified_rescan(fx: &Fixture, touch_every: usize) -> u128 {
    for i in 0..fx.file_count {
        if i % touch_every == 0 {
            let path = fx.root.join(format!("src/mod{:05}.ts", i));
            // Append a comment to change size + mtime.
            std::fs::write(&path, "// touched\n").expect("touch");
        }
    }
    bench_scan(fx)
}

/// Returns import/symbol/file counts for a fixture, for reporting.
pub struct Counts {
    pub files: usize,
    pub imports: usize,
    pub symbols: usize,
}

pub fn count_all(fx: &Fixture) -> Counts {
    let files = codecompass_lib::db::indexed_files::list_workspace_files(&fx.db, fx.workspace_id)
        .expect("list");
    let imports = codecompass_lib::db::imports::list_imports_for_workspace(&fx.db, fx.workspace_id)
        .map(|v| v.len())
        .unwrap_or(0);
    let sym = codecompass_lib::db::symbols::search_symbols(
        &fx.db,
        fx.workspace_id,
        None,
        None,
        1,
        100000,
    )
    .expect("symbols");
    Counts {
        files: files.len(),
        imports,
        symbols: sym.total as usize,
    }
}
