pub mod analysis;
pub mod connection;
pub mod imports;
pub mod indexed_files;
pub mod indexed_folders;
pub mod scan_runs;
pub mod symbols;

refinery::embed_migrations!("migrations");

pub use connection::Database;

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tempfile::tempdir;

    /// Verifies that the initial migration creates all four required tables
    /// (workspaces, indexed_files, analysis_runs, app_settings) in a fresh
    /// temporary database.
    #[test]
    fn migration_creates_all_tables() {
        let dir = tempdir().expect("create temp dir");
        let db_path = dir.path().join("test.db");
        let mut conn = Connection::open(&db_path).expect("open sqlite connection");

        super::migrations::runner()
            .run(&mut conn)
            .expect("run refinery migrations");

        let tables: Vec<String> = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='table' \
                 ORDER BY name",
            )
            .expect("prepare table query")
            .query_map([], |row| row.get::<_, String>(0))
            .expect("execute table query")
            .filter_map(|r| r.ok())
            .collect();

        assert!(
            tables.contains(&"workspaces".to_string()),
            "workspaces table missing, got: {tables:?}",
        );
        assert!(
            tables.contains(&"indexed_files".to_string()),
            "indexed_files table missing, got: {tables:?}",
        );
        assert!(
            tables.contains(&"analysis_runs".to_string()),
            "analysis_runs table missing, got: {tables:?}",
        );
        assert!(
            tables.contains(&"app_settings".to_string()),
            "app_settings table missing, got: {tables:?}",
        );
    }
}
