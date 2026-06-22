use rusqlite::params;

use crate::analysis::ImportRecord;
use crate::db::Database;
use crate::error::AppError;
use crate::models::ImportEntry;

/// Batch-inserts import records for a single file. Any existing imports for
/// that file are deleted first (full-replace strategy).
pub fn replace_file_imports(
    db: &Database,
    file_id: i64,
    imports: &[ImportRecord],
    now: i64,
) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM imports WHERE source_file_id = ?1",
        params![file_id],
    )?;

    if imports.is_empty() {
        return Ok(());
    }

    let mut stmt = conn.prepare(
        "INSERT INTO imports \
         (source_file_id, target_specifier, resolved_target_file_id, import_type, \
          is_external, start_line, start_column, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
    )?;

    for import in imports {
        let resolved_id = import
            .resolved_target
            .as_ref()
            .and_then(|p| resolve_file_id_strict(&conn, p).ok());
        stmt.execute(params![
            file_id,
            import.target_specifier,
            resolved_id,
            import.import_type.as_str(),
            import.is_external as i64,
            import.start_line,
            import.start_column,
            now,
        ])?;
    }

    Ok(())
}

/// Try to find the `indexed_files.id` for an absolute path. Returns an error
/// if none matches (the file may be outside the workspace or not indexed).
#[allow(dead_code)]
fn resolve_file_id_strict(
    conn: &rusqlite::Connection,
    path: &std::path::Path,
) -> Result<i64, AppError> {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let parent = path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let id: i64 = conn.query_row(
        "SELECT id FROM indexed_files \
         WHERE name = ?1 AND ?2 LIKE '%' || parent_path || '%' \
         LIMIT 1",
        params![file_name, parent],
        |row| row.get(0),
    )?;
    Ok(id)
}

/// Lists imports for a given source file.
pub fn list_imports_for_file(db: &Database, file_id: i64) -> Result<Vec<ImportEntry>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, source_file_id, target_specifier, resolved_target_file_id, \
         import_type, is_external, start_line, start_column \
         FROM imports WHERE source_file_id = ?1 ORDER BY start_line, start_column",
    )?;
    let rows = stmt.query_map(params![file_id], |row| {
        Ok(ImportEntry {
            id: row.get(0)?,
            source_file_id: row.get(1)?,
            target_specifier: row.get(2)?,
            resolved_target_file_id: row.get(3)?,
            import_type: row.get(4)?,
            is_external: row.get::<_, i64>(5)? != 0,
            start_line: row.get(6)?,
            start_column: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)
}

/// Removes all imports for files belonging to a workspace. Called when
/// starting a fresh analysis run.
pub fn clear_workspace_imports(db: &Database, workspace_id: i64) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "DELETE FROM imports WHERE source_file_id IN \
         (SELECT id FROM indexed_files WHERE workspace_id = ?1)",
        params![workspace_id],
    )?;
    Ok(())
}

/// Returns all imports for a workspace (used for graph building in future
/// milestones).
#[allow(dead_code)]
pub fn list_imports_for_workspace(
    db: &Database,
    workspace_id: i64,
) -> Result<Vec<ImportEntry>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT i.id, i.source_file_id, i.target_specifier, i.resolved_target_file_id, \
         i.import_type, i.is_external, i.start_line, i.start_column \
         FROM imports i \
         JOIN indexed_files f ON i.source_file_id = f.id \
         WHERE f.workspace_id = ?1 \
         ORDER BY i.source_file_id, i.start_line",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(ImportEntry {
            id: row.get(0)?,
            source_file_id: row.get(1)?,
            target_specifier: row.get(2)?,
            resolved_target_file_id: row.get(3)?,
            import_type: row.get(4)?,
            is_external: row.get::<_, i64>(5)? != 0,
            start_line: row.get(6)?,
            start_column: row.get(7)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::ts_js::ImportType;
    use crate::db::indexed_files::upsert_files_batch;
    use crate::db::indexed_folders::insert_indexed_folder;
    use crate::db::Database;
    use tempfile::tempdir;

    #[test]
    fn replace_and_list_round_trip() {
        let dir = tempdir().expect("temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open");
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).expect("create");
        let fid = insert_indexed_folder(&db, &folder).expect("insert").id;

        // Create a file record.
        use crate::db::indexed_files::FileUpsert;
        let mut batch = vec![FileUpsert {
            relative_path: "a.ts".to_string(),
            name: "a.ts".to_string(),
            parent_path: ".".to_string(),
            extension: Some("ts".to_string()),
            size_bytes: 100,
            created_at: Some(1),
            modified_at: Some(2),
            fingerprint: "fp".to_string(),
            indexed_at: 1000,
            last_seen_at: 1000,
        }];
        upsert_files_batch(&db, fid, 0, &mut batch).expect("upsert");
        let file_id = db
            .lock()
            .unwrap()
            .query_row(
                "SELECT id FROM indexed_files WHERE workspace_id = ?1 AND relative_path = 'a.ts'",
                params![fid],
                |row| row.get::<_, i64>(0),
            )
            .expect("query");

        let imports = vec![ImportRecord {
            source_file_id: file_id,
            target_specifier: "react".to_string(),
            resolved_target: None,
            import_type: ImportType::StaticImport,
            is_external: true,
            start_line: Some(1),
            start_column: Some(1),
        }];

        let now = 2000;
        replace_file_imports(&db, file_id, &imports, now).expect("replace");

        let entries = list_imports_for_file(&db, file_id).expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].target_specifier, "react");
        assert!(entries[0].is_external);
    }
}
