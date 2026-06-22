use rusqlite::params;

use crate::analysis::references::SymbolReference;
use crate::db::Database;
use crate::error::AppError;

/// A persisted reference row.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct ReferenceEntry {
    pub id: i64,
    pub caller_symbol_id: Option<i64>,
    pub callee_name: String,
    pub caller_file_id: i64,
    pub resolved_callee_symbol_id: Option<i64>,
    pub reference_type: String,
    pub source_line: i64,
    pub source_column: i64,
}

/// Replaces references for a file (full delete + insert).
pub fn replace_file_references(
    db: &Database,
    file_id: i64,
    workspace_id: i64,
    refs: &[SymbolReference],
    now: i64,
) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM symbol_references WHERE caller_file_id = ?1",
        params![file_id],
    )?;
    if refs.is_empty() {
        return Ok(());
    }

    // Resolve caller symbol ID by matching enclosing_function name + file.
    for r in refs {
        let caller_sym_id: Option<i64> = r.enclosing_function.as_ref().and_then(|name| {
            conn.query_row(
                "SELECT id FROM symbols WHERE file_id = ?1 AND name = ?2 ORDER BY id DESC LIMIT 1",
                params![file_id, name],
                |row| row.get(0),
            )
            .ok()
        });

        // Try to resolve callee symbol ID.
        let callee_sym_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM symbols WHERE workspace_id = ?1 AND name = ?2 ORDER BY id LIMIT 1",
                params![workspace_id, r.callee_name],
                |row| row.get(0),
            )
            .ok();

        conn.execute(
            "INSERT INTO symbol_references \
             (workspace_id, caller_symbol_id, callee_name, caller_file_id, \
              resolved_callee_symbol_id, reference_type, source_line, source_column, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                workspace_id,
                caller_sym_id,
                r.callee_name,
                file_id,
                callee_sym_id,
                r.reference_type.as_str(),
                r.source_line,
                r.source_column,
                now,
            ],
        )?;
    }
    Ok(())
}

/// Clears all references for a workspace.
pub fn clear_workspace_references(db: &Database, workspace_id: i64) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM symbol_references WHERE workspace_id = ?1",
        params![workspace_id],
    )?;
    Ok(())
}

/// Returns references where the given symbol is called.
#[allow(dead_code)]
pub fn get_callee_references(
    db: &Database,
    symbol_id: i64,
) -> Result<Vec<ReferenceEntry>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, caller_symbol_id, callee_name, caller_file_id, \
         resolved_callee_symbol_id, reference_type, source_line, source_column \
         FROM symbol_references \
         WHERE resolved_callee_symbol_id = ?1",
    )?;
    let rows = stmt
        .query_map(params![symbol_id], map_ref_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Returns references where the symbol is the caller.
#[allow(dead_code)]
pub fn get_caller_references(
    db: &Database,
    symbol_id: i64,
) -> Result<Vec<ReferenceEntry>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, caller_symbol_id, callee_name, caller_file_id, \
         resolved_callee_symbol_id, reference_type, source_line, source_column \
         FROM symbol_references \
         WHERE caller_symbol_id = ?1",
    )?;
    let rows = stmt
        .query_map(params![symbol_id], map_ref_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

#[allow(dead_code)]
fn map_ref_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ReferenceEntry> {
    Ok(ReferenceEntry {
        id: row.get(0)?,
        caller_symbol_id: row.get(1)?,
        callee_name: row.get(2)?,
        caller_file_id: row.get(3)?,
        resolved_callee_symbol_id: row.get(4)?,
        reference_type: row.get(5)?,
        source_line: row.get(6)?,
        source_column: row.get(7)?,
    })
}
