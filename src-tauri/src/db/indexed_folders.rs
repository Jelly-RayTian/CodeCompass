use std::path::Path;

use rusqlite::{params, OptionalExtension};

use crate::error::AppError;
use crate::models::IndexedFolder;
use crate::platform::{normalize_existing_path, path_is_strict_descendant, paths_equal};

use super::Database;

/// Returns all indexed folders, computing availability from the filesystem.
pub fn list_indexed_folders(db: &Database) -> Result<Vec<IndexedFolder>, AppError> {
    let conn = db.lock()?;
    let mut stmt = conn.prepare(
        "SELECT id, name, path, added_at, last_successful_scan_at, \
         monitoring_enabled, scan_status \
         FROM workspaces ORDER BY added_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let path: String = row.get(2)?;
        let added_at: i64 = row.get(3)?;
        let last_successful_scan_at: Option<i64> = row.get(4)?;
        let monitoring_enabled: i64 = row.get(5)?;
        let scan_status: String = row.get(6)?;
        let availability = compute_availability(&path);
        Ok(IndexedFolder {
            id,
            name,
            path,
            added_at,
            last_successful_scan_at,
            availability,
            monitoring_enabled: monitoring_enabled != 0,
            scan_status,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(AppError::Database)
}

/// Computes the availability string for a stored path.
fn compute_availability(path: &str) -> String {
    let path = Path::new(path);
    if !path.exists() {
        return "missing".to_string();
    }
    match std::fs::metadata(path) {
        Ok(meta) if meta.is_dir() => match std::fs::read_dir(path) {
            Ok(_) => "available".to_string(),
            Err(_) => "inaccessible".to_string(),
        },
        Ok(_) => "not_a_directory".to_string(),
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            "permission_denied".to_string()
        }
        Err(_) => "inaccessible".to_string(),
    }
}

/// Checks whether a normalized path is already indexed exactly.
pub fn find_duplicate(db: &Database, path: &Path) -> Result<Option<i64>, AppError> {
    let folders = list_indexed_folders(db)?;
    Ok(folders.into_iter().find_map(|f| {
        if paths_equal(Path::new(&f.path), path) {
            Some(f.id)
        } else {
            None
        }
    }))
}

/// Returns the normalized path of every indexed folder that is a parent of
/// `candidate`, together with a human-readable warning.
pub fn find_nested_parents(
    db: &Database,
    candidate: &Path,
) -> Result<Vec<(String, String)>, AppError> {
    let folders = list_indexed_folders(db)?;
    let mut parents = Vec::new();
    for folder in folders {
        let folder_path = Path::new(&folder.path);
        if path_is_strict_descendant(folder_path, candidate) {
            parents.push((
                folder.path.clone(),
                format!(
                    "{} is already covered by the indexed folder {}",
                    candidate.display(),
                    folder.path
                ),
            ));
        }
    }
    Ok(parents)
}

/// Inserts a new indexed folder and returns it.
pub fn insert_indexed_folder(db: &Database, path: &Path) -> Result<IndexedFolder, AppError> {
    let normalized = normalize_existing_path(path)?;
    let name = normalized
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unnamed folder".to_string());
    let path_str = normalized.to_string_lossy().to_string();
    let now = now_epoch_secs();

    let conn = db.lock()?;
    conn.execute(
        "INSERT INTO workspaces \
         (name, path, created_at, added_at, availability, monitoring_enabled, scan_status) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![name, path_str, now, now, "available", 0, "idle"],
    )?;
    let id = conn.last_insert_rowid();
    Ok(IndexedFolder {
        id,
        name,
        path: path_str,
        added_at: now,
        last_successful_scan_at: None,
        availability: "available".to_string(),
        monitoring_enabled: false,
        scan_status: "idle".to_string(),
    })
}

/// Removes an indexed folder by id, deleting Chronicle's index through CASCADE.
pub fn remove_indexed_folder(db: &Database, id: i64) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
    Ok(())
}

/// Returns the path of the indexed folder with the given id, if it exists.
pub fn get_folder_path(db: &Database, id: i64) -> Result<Option<String>, AppError> {
    let conn = db.lock()?;
    conn.query_row(
        "SELECT path FROM workspaces WHERE id = ?1",
        params![id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(AppError::Database)
}

/// Updates the scan-related status fields on a folder.
pub fn update_folder_scan_status(
    db: &Database,
    id: i64,
    status: &str,
    last_successful_scan_at: Option<i64>,
) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "UPDATE workspaces SET scan_status = ?1, last_successful_scan_at = ?2 WHERE id = ?3",
        params![status, last_successful_scan_at, id],
    )?;
    Ok(())
}

/// Updates the analysis status field on a folder.
pub fn update_folder_analysis_status(db: &Database, id: i64, status: &str) -> Result<(), AppError> {
    let conn = db.lock()?;
    conn.execute(
        "UPDATE workspaces SET scan_status = ?1 WHERE id = ?2",
        params![status, id],
    )?;
    Ok(())
}

fn now_epoch_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use tempfile::tempdir;

    #[test]
    fn insert_and_list_round_trip() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folder_dir = dir.path().join("my_folder");
        std::fs::create_dir(&folder_dir).expect("create folder");

        let inserted = insert_indexed_folder(&db, &folder_dir).expect("insert");
        let listed = list_indexed_folders(&db).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, inserted.id);
        assert_eq!(listed[0].name, "my_folder");
        assert!(listed[0].availability == "available");
    }

    #[test]
    fn duplicate_detection_is_case_insensitive_on_windows() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folder_dir = dir.path().join("MyFolder");
        std::fs::create_dir(&folder_dir).expect("create folder");

        insert_indexed_folder(&db, &folder_dir).expect("insert");
        let dup_path = if cfg!(windows) {
            dir.path().join("myfolder")
        } else {
            folder_dir.clone()
        };
        let duplicate = find_duplicate(&db, &dup_path).expect("check duplicate");
        assert!(duplicate.is_some());
    }

    #[test]
    fn nested_parent_detection_warns() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let parent = dir.path().join("parent");
        let child = parent.join("child");
        std::fs::create_dir_all(&child).expect("create dirs");

        insert_indexed_folder(&db, &parent).expect("insert parent");
        let nested = find_nested_parents(&db, &child).expect("find nested");
        assert_eq!(nested.len(), 1);
    }

    #[test]
    fn remove_folder_deletes_index_but_not_files() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folder_dir = dir.path().join("to_keep");
        std::fs::create_dir(&folder_dir).expect("create folder");
        let file = folder_dir.join("file.txt");
        std::fs::write(&file, "data").expect("write file");

        let inserted = insert_indexed_folder(&db, &folder_dir).expect("insert");
        remove_indexed_folder(&db, inserted.id).expect("remove");

        assert!(folder_dir.exists());
        assert!(file.exists());
        let remaining = list_indexed_folders(&db).expect("list");
        assert!(remaining.is_empty());
    }

    #[test]
    fn folders_persist_after_database_reopen() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let folder_dir = dir.path().join("persistent");
        std::fs::create_dir(&folder_dir).expect("create folder");

        {
            let db = Database::open(&db_path).expect("open database");
            insert_indexed_folder(&db, &folder_dir).expect("insert");
        }

        let db = Database::open(&db_path).expect("reopen database");
        let listed = list_indexed_folders(&db).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(
            listed[0].path,
            normalize_existing_path(&folder_dir)
                .expect("normalize")
                .to_string_lossy()
        );
    }

    #[test]
    fn missing_folder_reports_missing_availability() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let db = Database::open(&db_path).expect("open database");
        let folder_dir = dir.path().join("will_vanish");
        std::fs::create_dir(&folder_dir).expect("create folder");

        insert_indexed_folder(&db, &folder_dir).expect("insert");
        std::fs::remove_dir(&folder_dir).expect("remove folder");

        let listed = list_indexed_folders(&db).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].availability, "missing");
    }
}
