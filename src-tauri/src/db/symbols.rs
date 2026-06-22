use rusqlite::params;

use crate::analysis::symbols::SymbolRecord;
use crate::db::Database;
use crate::error::AppError;

/// A persisted symbol row returned to the frontend.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolEntry {
    pub id: i64,
    pub workspace_id: i64,
    pub file_id: i64,
    pub name: String,
    pub kind: String,
    pub parent_symbol_id: Option<i64>,
    pub source_line: i64,
    pub source_column: i64,
    pub source_end_line: i64,
    pub source_end_column: i64,
    pub signature: Option<String>,
    pub visibility: String,
    pub is_exported: bool,
    pub relative_path: Option<String>,
}

/// Search result with pagination.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolSearchResult {
    pub symbols: Vec<SymbolEntry>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// Replaces all symbols for a file (full delete + insert).
pub fn replace_file_symbols(
    db: &Database,
    file_id: i64,
    workspace_id: i64,
    symbols: &[SymbolRecord],
    now: i64,
) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute("DELETE FROM symbols WHERE file_id = ?1", params![file_id])?;

    if symbols.is_empty() {
        return Ok(());
    }

    let mut stmt = conn.prepare(
        "INSERT INTO symbols \
         (workspace_id, file_id, name, kind, parent_symbol_id, \
          source_line, source_column, source_end_line, source_end_column, \
          signature, visibility, is_exported, created_at) \
         VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
    )?;

    let mut symbol_ids: Vec<(String, i64)> = Vec::new();

    for sym in symbols {
        stmt.execute(params![
            workspace_id,
            file_id,
            sym.name,
            sym.kind.as_str(),
            sym.source_line,
            sym.source_column,
            sym.source_end_line,
            sym.source_end_column,
            sym.signature,
            sym.visibility.as_str(),
            sym.is_exported as i64,
            now,
        ])?;
        let id = conn.last_insert_rowid();
        if sym.parent_name.is_some() {
            symbol_ids.push((sym.parent_name.clone().unwrap(), id));
        }
    }

    // Second pass: set parent_symbol_id for methods/nested symbols.
    for (parent_name, child_id) in &symbol_ids {
        conn.execute(
            "UPDATE symbols SET parent_symbol_id = (\
             SELECT id FROM symbols \
             WHERE file_id = ?1 AND name = ?2 AND kind = 'class' AND id != ?3 \
             ORDER BY id DESC LIMIT 1\
             ) WHERE id = ?3",
            params![file_id, parent_name, child_id],
        )?;
    }

    Ok(())
}

fn query_symbols(
    conn: &rusqlite::Connection,
    sql: &str,
    workspace_id: i64,
    query: Option<&str>,
    kind_filter: Option<&str>,
    page_size: i64,
    offset: i64,
) -> Result<Vec<SymbolEntry>, AppError> {
    let rows: Vec<SymbolEntry> = match (query, kind_filter) {
        (Some(q), Some(k)) if !q.trim().is_empty() && !k.trim().is_empty() => conn
            .prepare(sql)?
            .query_map(
                params![workspace_id, format!("%{}%", q), k, page_size, offset],
                map_symbol_row,
            )?
            .collect::<Result<Vec<_>, _>>()?,
        (Some(q), _) if !q.trim().is_empty() => conn
            .prepare(sql)?
            .query_map(
                params![workspace_id, format!("%{}%", q), page_size, offset],
                map_symbol_row,
            )?
            .collect::<Result<Vec<_>, _>>()?,
        (_, Some(k)) if !k.trim().is_empty() => conn
            .prepare(sql)?
            .query_map(params![workspace_id, k, page_size, offset], map_symbol_row)?
            .collect::<Result<Vec<_>, _>>()?,
        _ => conn
            .prepare(sql)?
            .query_map(params![workspace_id, page_size, offset], map_symbol_row)?
            .collect::<Result<Vec<_>, _>>()?,
    };
    Ok(rows)
}

/// Searches symbols across a workspace with optional kind and name filter.
pub fn search_symbols(
    db: &Database,
    workspace_id: i64,
    query: Option<&str>,
    kind_filter: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<SymbolSearchResult, AppError> {
    let conn = db.lock()?;

    let mut where_clauses = vec!["s.workspace_id = ?1".to_string()];
    let mut param_idx = 2;

    if let Some(q) = query {
        if !q.trim().is_empty() {
            where_clauses.push(format!("s.name LIKE ?{param_idx} COLLATE NOCASE"));
            param_idx += 1;
        }
    }
    if let Some(k) = kind_filter {
        if !k.trim().is_empty() {
            where_clauses.push(format!("s.kind = ?{param_idx}"));
            param_idx += 1;
        }
    }

    let where_clause = where_clauses.join(" AND ");

    let count_sql = format!("SELECT COUNT(*) FROM symbols s WHERE {}", where_clause);
    let select_sql = format!(
        "SELECT s.id, s.workspace_id, s.file_id, s.name, s.kind, \
         s.parent_symbol_id, s.source_line, s.source_column, \
         s.source_end_line, s.source_end_column, s.signature, \
         s.visibility, s.is_exported, \
         f.relative_path \
         FROM symbols s \
         LEFT JOIN indexed_files f ON s.file_id = f.id \
         WHERE {} \
         ORDER BY s.name COLLATE NOCASE \
         LIMIT ?{} OFFSET ?{}",
        where_clause,
        param_idx,
        param_idx + 1
    );

    let mut total: i64 = match (query, kind_filter) {
        (Some(q), Some(k)) if !q.trim().is_empty() && !k.trim().is_empty() => conn.query_row(
            &count_sql,
            params![workspace_id, format!("%{}%", q), k],
            |row| row.get(0),
        )?,
        (Some(q), _) if !q.trim().is_empty() => conn.query_row(
            &count_sql,
            params![workspace_id, format!("%{}%", q)],
            |row| row.get(0),
        )?,
        (_, Some(k)) if !k.trim().is_empty() => {
            conn.query_row(&count_sql, params![workspace_id, k], |row| row.get(0))?
        }
        _ => conn.query_row(&count_sql, params![workspace_id], |row| row.get(0))?,
    };

    let offset = (page - 1) * page_size;
    let rows: Vec<SymbolEntry> = query_symbols(
        &conn,
        &select_sql,
        workspace_id,
        query,
        kind_filter,
        page_size,
        offset,
    )?;

    Ok(SymbolSearchResult {
        symbols: rows,
        total,
        page,
        page_size,
    })
}

/// Returns symbols for a single file (the outline).
pub fn file_outline(db: &Database, file_id: i64) -> Result<Vec<SymbolEntry>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT s.id, s.workspace_id, s.file_id, s.name, s.kind, \
         s.parent_symbol_id, s.source_line, s.source_column, \
         s.source_end_line, s.source_end_column, s.signature, \
         s.visibility, s.is_exported, \
         f.relative_path \
         FROM symbols s \
         LEFT JOIN indexed_files f ON s.file_id = f.id \
         WHERE s.file_id = ?1 \
         ORDER BY s.source_line",
    )?;
    let rows = stmt
        .query_map(params![file_id], map_symbol_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Clears all symbols for a workspace.
pub fn clear_workspace_symbols(db: &Database, workspace_id: i64) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM symbols WHERE workspace_id = ?1",
        params![workspace_id],
    )?;
    Ok(())
}

fn map_symbol_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<SymbolEntry> {
    Ok(SymbolEntry {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        file_id: row.get(2)?,
        name: row.get(3)?,
        kind: row.get(4)?,
        parent_symbol_id: row.get(5)?,
        source_line: row.get(6)?,
        source_column: row.get(7)?,
        source_end_line: row.get(8)?,
        source_end_column: row.get(9)?,
        signature: row.get(10)?,
        visibility: row.get(11)?,
        is_exported: row.get::<_, i64>(12)? != 0,
        relative_path: row.get(13)?,
    })
}
