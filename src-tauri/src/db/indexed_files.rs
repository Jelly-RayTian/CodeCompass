use rusqlite::{params, OptionalExtension};

use crate::db::Database;
use crate::error::AppError;
use crate::models::FileEntry;

const SUPPORTED_EXTENSIONS: &[&str] = &["ts", "tsx", "js", "jsx"];

/// Returns `true` if the extension is one Milestone 2 currently indexes.
pub fn is_supported_extension(extension: Option<&str>) -> bool {
    match extension {
        Some(ext) => SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()),
        None => false,
    }
}

/// Information needed to upsert a single file during a scan.
pub struct FileUpsert {
    pub relative_path: String,
    pub name: String,
    pub parent_path: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub created_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub fingerprint: String,
    pub indexed_at: i64,
    pub last_seen_at: i64,
}

/// Upserts a batch of file rows for a workspace. The previous fingerprint is
/// preserved from the existing row so that change detection can be performed.
pub fn upsert_files_batch(
    db: &Database,
    workspace_id: i64,
    batch: &mut Vec<FileUpsert>,
) -> Result<(), AppError> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    {
        let mut insert = tx.prepare(
            "INSERT INTO indexed_files AS f \
             (workspace_id, relative_path, name, parent_path, extension, \
              size_bytes, created_at, modified_at, last_indexed_at, indexed_at, last_seen_at, \
              fingerprint, previous_fingerprint, is_present, change_status, language, file_hash) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, 1, ?14, NULL, NULL) \
             ON CONFLICT(workspace_id, relative_path) DO UPDATE SET \
              name = excluded.name, \
              parent_path = excluded.parent_path, \
              extension = excluded.extension, \
              size_bytes = excluded.size_bytes, \
              created_at = excluded.created_at, \
              modified_at = excluded.modified_at, \
              last_seen_at = excluded.last_seen_at, \
              previous_fingerprint = f.fingerprint, \
              fingerprint = excluded.fingerprint, \
              is_present = 1, \
              change_status = CASE \
                WHEN f.fingerprint IS NULL THEN 'new' \
                WHEN f.fingerprint = excluded.fingerprint THEN 'unchanged' \
                ELSE 'changed' \
              END",
        )?;
        for row in batch.drain(..) {
            // For a brand-new row the ON CONFLICT clause does not fire, so we
            // default to 'new' here. Conflicting rows get the correct status
            // from the CASE expression in the DO UPDATE clause.
            let change_status = "new";
            insert.execute(params![
                workspace_id,
                row.relative_path,
                row.name,
                row.parent_path,
                row.extension,
                row.size_bytes,
                row.created_at,
                row.modified_at,
                row.indexed_at,
                row.indexed_at,
                row.last_seen_at,
                row.fingerprint,
                None::<&str>,
                change_status,
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Marks all files that were not seen during the current successful scan as
/// deleted. `last_seen_before` should be the scan start timestamp.
pub fn mark_removed_files(
    db: &Database,
    workspace_id: i64,
    last_seen_before: i64,
) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "UPDATE indexed_files \
         SET is_present = 0, change_status = 'removed' \
         WHERE workspace_id = ?1 AND last_seen_at < ?2 AND is_present = 1",
        params![workspace_id, last_seen_before],
    )?;
    Ok(())
}

/// Lists all present files for a workspace, ordered by relative path.
pub fn list_workspace_files(db: &Database, workspace_id: i64) -> Result<Vec<FileEntry>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, relative_path, name, parent_path, extension, \
         size_bytes, created_at, modified_at, indexed_at, last_seen_at, \
         fingerprint, previous_fingerprint, is_present, change_status \
         FROM indexed_files \
         WHERE workspace_id = ?1 \
         ORDER BY relative_path",
    )?;
    let rows = stmt.query_map(params![workspace_id], |row| {
        Ok(FileEntry {
            id: row.get(0)?,
            workspace_id: row.get(1)?,
            relative_path: row.get(2)?,
            name: row.get(3)?,
            parent_path: row.get(4)?,
            extension: row.get(5)?,
            size_bytes: row.get(6)?,
            created_at: row.get(7)?,
            modified_at: row.get(8)?,
            indexed_at: row.get(9)?,
            last_seen_at: row.get(10)?,
            fingerprint: row.get(11)?,
            previous_fingerprint: row.get(12)?,
            is_present: row.get::<_, i64>(13)? != 0,
            change_status: row.get(14)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)
}

/// Returns the details of a single indexed file.
pub fn get_file_details(db: &Database, file_id: i64) -> Result<Option<FileEntry>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, workspace_id, relative_path, name, parent_path, extension, \
         size_bytes, created_at, modified_at, indexed_at, last_seen_at, \
         fingerprint, previous_fingerprint, is_present, change_status \
         FROM indexed_files \
         WHERE id = ?1",
    )?;
    let row = stmt
        .query_row(params![file_id], |row| {
            Ok(FileEntry {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                relative_path: row.get(2)?,
                name: row.get(3)?,
                parent_path: row.get(4)?,
                extension: row.get(5)?,
                size_bytes: row.get(6)?,
                created_at: row.get(7)?,
                modified_at: row.get(8)?,
                indexed_at: row.get(9)?,
                last_seen_at: row.get(10)?,
                fingerprint: row.get(11)?,
                previous_fingerprint: row.get(12)?,
                is_present: row.get::<_, i64>(13)? != 0,
                change_status: row.get(14)?,
            })
        })
        .optional()?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::indexed_folders::insert_indexed_folder;
    use crate::db::Database;
    use tempfile::tempdir;

    fn sample_upsert(relative_path: &str, fingerprint: &str) -> FileUpsert {
        FileUpsert {
            relative_path: relative_path.to_string(),
            name: "file".to_string(),
            parent_path: ".".to_string(),
            extension: Some("ts".to_string()),
            size_bytes: 100,
            created_at: Some(1),
            modified_at: Some(2),
            fingerprint: fingerprint.to_string(),
            indexed_at: 1000,
            last_seen_at: 1000,
        }
    }

    #[test]
    fn upsert_batch_detects_new_changed_unchanged() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).expect("create folder");
        let folder_id = insert_indexed_folder(&db, &folder).expect("insert").id;

        let mut batch = vec![sample_upsert("a.ts", "fp1")];
        upsert_files_batch(&db, folder_id, &mut batch).expect("upsert");

        let files = list_workspace_files(&db, folder_id).expect("list");
        assert_eq!(files[0].change_status, "new");

        let mut batch2 = vec![sample_upsert("a.ts", "fp2"), sample_upsert("b.ts", "fp3")];
        upsert_files_batch(&db, folder_id, &mut batch2).expect("upsert");
        let files = list_workspace_files(&db, folder_id).expect("list");
        let a = files.iter().find(|f| f.relative_path == "a.ts").unwrap();
        let b = files.iter().find(|f| f.relative_path == "b.ts").unwrap();
        assert_eq!(a.change_status, "changed");
        assert_eq!(b.change_status, "new");

        let mut batch3 = vec![sample_upsert("a.ts", "fp2")];
        upsert_files_batch(&db, folder_id, &mut batch3).expect("upsert");
        let files = list_workspace_files(&db, folder_id).expect("list");
        let a = files.iter().find(|f| f.relative_path == "a.ts").unwrap();
        assert_eq!(a.change_status, "unchanged");
    }

    #[test]
    fn mark_removed_files_sets_present_false() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folder = dir.path().join("root");
        std::fs::create_dir(&folder).expect("create folder");
        let folder_id = insert_indexed_folder(&db, &folder).expect("insert").id;

        let mut batch = vec![sample_upsert("old.ts", "fp1")];
        upsert_files_batch(&db, folder_id, &mut batch).expect("upsert");

        mark_removed_files(&db, folder_id, 2000).expect("mark removed");
        let files = list_workspace_files(&db, folder_id).expect("list");
        assert!(!files[0].is_present);
        assert_eq!(files[0].change_status, "removed");
    }
}
