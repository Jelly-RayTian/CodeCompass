use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::analysis::references::extract_references;
use crate::analysis::symbols::extract_symbols;
use crate::analysis::LanguageAnalyzer;
use crate::analysis::TypeScriptJavaScriptAnalyzer;
use crate::db::analysis::{clear_workspace_diagnostics, upsert_file_diagnostics};
use crate::db::imports::{clear_workspace_imports, replace_file_imports};
use crate::db::indexed_files::{
    get_files_for_analysis, mark_file_analysis_done, mark_file_parse_error, mark_pending_analysis,
};
use crate::db::indexed_folders::update_folder_analysis_status;
use crate::db::references::{clear_workspace_references, replace_file_references};
use crate::db::symbols::{clear_workspace_symbols, replace_file_symbols};
use crate::db::Database;
use crate::error::AppError;
use crate::models::AnalysisProgressEvent;

fn now_epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Runs analysis for a workspace.
///
/// * `db` — database handle.
/// * `workspace_id` — the workspace to analyse.
/// * `root_path` — the workspace root directory.
/// * `cancel` — cancellation token.
/// * `app` — for emitting progress events.
pub fn run_analysis(
    db: &Database,
    workspace_id: i64,
    root_path: &std::path::Path,
    cancel: Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<(), AppError> {
    let analyzer = TypeScriptJavaScriptAnalyzer;
    let now = now_epoch_secs();

    // Clear previous analysis results.
    clear_workspace_imports(db, workspace_id)?;
    clear_workspace_diagnostics(db, workspace_id)?;
    clear_workspace_symbols(db, workspace_id)?;
    clear_workspace_references(db, workspace_id)?;
    mark_pending_analysis(db, workspace_id)?;

    let files = get_files_for_analysis(db, workspace_id)?;
    let total = files.len() as i64;
    let mut processed: i64 = 0;
    let mut parsed: i64 = 0;
    let mut error_count: i64 = 0;

    emit_progress(
        app,
        AnalysisProgressEvent {
            workspace_id,
            status: "running".to_string(),
            files_processed: 0,
            files_total: total,
            files_parsed: 0,
            error_count: 0,
        },
    );

    for (file_id, relative_path) in &files {
        if cancel.load(Ordering::Relaxed) {
            update_folder_analysis_status(db, workspace_id, "idle")?;
            emit_progress(
                app,
                AnalysisProgressEvent {
                    workspace_id,
                    status: "cancelled".to_string(),
                    files_processed: processed,
                    files_total: total,
                    files_parsed: parsed,
                    error_count,
                },
            );
            return Ok(());
        }

        processed += 1;
        let absolute_path = root_path.join(relative_path);

        if !absolute_path.exists() || !absolute_path.is_file() {
            mark_file_parse_error(db, *file_id, &now, "file not found")?;
            error_count += 1;
            continue;
        }

        let source = match std::fs::read_to_string(&absolute_path) {
            Ok(s) => s,
            Err(e) => {
                mark_file_parse_error(db, *file_id, &now, &e.to_string())?;
                error_count += 1;
                continue;
            }
        };

        let (result, success) = analyzer.parse(*file_id, &absolute_path, root_path, &source);

        if success {
            replace_file_imports(db, *file_id, &result.imports, now)?;
            // Extract symbols from the same source.
            let symbols = extract_symbols(&source, &absolute_path);
            replace_file_symbols(db, *file_id, workspace_id, &symbols, now)?;
            // Extract symbol-level references (calls).
            let refs = extract_references(&source, &absolute_path);
            replace_file_references(db, *file_id, workspace_id, &refs, now)?;
            parsed += 1;
        }

        if !result.diagnostics.is_empty() {
            upsert_file_diagnostics(db, *file_id, workspace_id, &result.diagnostics, now)?;
            if !success {
                error_count += 1;
                mark_file_parse_error(db, *file_id, &now, &result.diagnostics[0].message)?;
            } else {
                mark_file_analysis_done(db, *file_id, &now)?;
            }
        } else if success {
            mark_file_analysis_done(db, *file_id, &now)?;
        }

        if processed % 10 == 0 {
            emit_progress(
                app,
                AnalysisProgressEvent {
                    workspace_id,
                    status: "running".to_string(),
                    files_processed: processed,
                    files_total: total,
                    files_parsed: parsed,
                    error_count,
                },
            );
        }
    }

    let final_status = if error_count > 0 {
        "analyzed_with_errors"
    } else {
        "analyzed"
    };
    update_folder_analysis_status(db, workspace_id, final_status)?;

    emit_progress(
        app,
        AnalysisProgressEvent {
            workspace_id,
            status: final_status.to_string(),
            files_processed: processed,
            files_total: total,
            files_parsed: parsed,
            error_count,
        },
    );

    Ok(())
}

fn emit_progress(app: &AppHandle, event: AnalysisProgressEvent) {
    let _ = app.emit("analysis:progress", event);
}
