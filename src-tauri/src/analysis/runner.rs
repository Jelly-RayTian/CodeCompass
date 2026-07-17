use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::analysis::plugin::build_registry;
use crate::analysis::references::extract_references;
use crate::analysis::symbols::extract_symbols;
use crate::db::analysis::upsert_file_diagnostics;
use crate::db::imports::replace_file_imports;
use crate::db::indexed_files::{
    mark_file_analysis_done, mark_file_parse_error, set_file_line_count,
};
use crate::db::indexed_folders::update_folder_analysis_status;
use crate::db::references::replace_file_references;
use crate::db::symbols::replace_file_symbols;
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
/// Uses the plugin registry to dispatch each file to the correct
/// language analyzer based on its extension.
pub fn run_analysis(
    db: &Database,
    workspace_id: i64,
    root_path: &std::path::Path,
    cancel: Arc<AtomicBool>,
    app: &AppHandle,
) -> Result<(), AppError> {
    let registry = build_registry();
    let now = now_epoch_secs();

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

        let line_count = source.lines().count() as i64;
        let _ = set_file_line_count(db, *file_id, line_count);

        // Resolve the right analyzer for this file's extension.
        let ext = absolute_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        let analyzer = registry.resolve(ext);
        let (result, success) = if let Some(a) = analyzer {
            a.parse(*file_id, &absolute_path, root_path, &source)
        } else {
            mark_file_parse_error(db, *file_id, &now, "no analyzer for extension")?;
            error_count += 1;
            continue;
        };

        if success {
            replace_file_imports(db, *file_id, &result.imports, now)?;
            let symbols = extract_symbols(&source, &absolute_path);
            replace_file_symbols(db, *file_id, workspace_id, &symbols, now)?;
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

/// Fetches files for analysis, filtered to extensions known to the plugin
/// registry. Builds the SQL `IN (...)` clause dynamically from registered
/// extensions so that adding a new analyzer automatically includes its
/// files without query changes.
fn get_files_for_analysis(
    db: &Database,
    workspace_id: i64,
) -> Result<Vec<(i64, String)>, AppError> {
    let registry = build_registry();
    let extensions: Vec<String> = registry.all_extensions();
    if extensions.is_empty() {
        return Ok(Vec::new());
    }

    let conn = db.lock()?;
    let placeholders: Vec<String> = extensions
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 3))
        .collect();
    let sql = format!(
        "SELECT id, relative_path FROM indexed_files \
         WHERE workspace_id = ?1 AND is_present = 1 \
         AND (analysis_status = 'pending' OR change_status IN ('new', 'changed')) \
         AND extension IN ({})",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
    params.push(Box::new(workspace_id));
    params.push(Box::new(1i64));
    for ext in &extensions {
        params.push(Box::new(ext.clone()));
    }
    let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt.query_map(refs.as_slice(), |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)
}
