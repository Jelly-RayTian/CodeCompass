use rusqlite::params;

use crate::analysis::ParseDiagnostic;
use crate::db::Database;
use crate::error::AppError;
use crate::models::AnalysisDiagnostic;

/// Inserts a batch of diagnostics for a file.
pub fn upsert_file_diagnostics(
    db: &Database,
    file_id: i64,
    workspace_id: i64,
    diagnostics: &[ParseDiagnostic],
    now: i64,
) -> Result<(), AppError> {
    let conn = db.lock()?;
    // Delete old diagnostics for this file.
    conn.execute(
        "DELETE FROM analysis_diagnostics WHERE file_id = ?1",
        params![file_id],
    )?;

    if diagnostics.is_empty() {
        return Ok(());
    }

    let mut stmt = conn.prepare(
        "INSERT INTO analysis_diagnostics \
         (file_id, workspace_id, severity, message, line, \"column\", created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?;

    for d in diagnostics {
        stmt.execute(params![
            file_id,
            workspace_id,
            d.severity,
            d.message,
            d.line,
            d.column,
            now,
        ])?;
    }

    Ok(())
}

/// Lists diagnostics for a workspace, optional severity filter.
pub fn list_diagnostics(
    db: &Database,
    workspace_id: i64,
    severity: Option<&str>,
) -> Result<Vec<AnalysisDiagnostic>, AppError> {
    let conn = db.lock()?;
    let query = match severity {
        Some(_) => {
            "SELECT id, file_id, workspace_id, severity, message, line, \"column\", created_at \
                    FROM analysis_diagnostics \
                    WHERE workspace_id = ?1 AND severity = ?2 \
                    ORDER BY file_id, line"
        }
        None => {
            "SELECT id, file_id, workspace_id, severity, message, line, \"column\", created_at \
                 FROM analysis_diagnostics \
                 WHERE workspace_id = ?1 \
                 ORDER BY file_id, line"
        }
    };

    let mut stmt = conn.prepare(query)?;
    let rows = if let Some(sev) = severity {
        stmt.query_map(params![workspace_id, sev], map_row)?
    } else {
        stmt.query_map(params![workspace_id], map_row)?
    };

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<AnalysisDiagnostic> {
    Ok(AnalysisDiagnostic {
        id: row.get(0)?,
        file_id: row.get(1)?,
        workspace_id: row.get(2)?,
        severity: row.get(3)?,
        message: row.get(4)?,
        line: row.get(5)?,
        column: row.get(6)?,
        created_at: row.get(7)?,
    })
}

/// Clears diagnostics for a workspace.
pub fn clear_workspace_diagnostics(db: &Database, workspace_id: i64) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM analysis_diagnostics WHERE workspace_id = ?1",
        params![workspace_id],
    )?;
    Ok(())
}
