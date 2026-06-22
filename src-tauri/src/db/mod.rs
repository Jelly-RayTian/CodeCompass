pub mod analysis;
pub mod connection;
pub mod imports;
pub mod indexed_files;
pub mod indexed_folders;
pub mod references;
pub mod scan_runs;
pub mod symbols;
pub mod workspace_settings;

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
        // V4: import relationships
        assert!(
            tables.contains(&"imports".to_string()),
            "imports table missing (V4), got: {tables:?}",
        );
        assert!(
            tables.contains(&"analysis_diagnostics".to_string()),
            "analysis_diagnostics table missing (V4), got: {tables:?}",
        );
        // V5: symbols
        assert!(
            tables.contains(&"symbols".to_string()),
            "symbols table missing (V5), got: {tables:?}",
        );
        // V6: symbol references
        assert!(
            tables.contains(&"symbol_references".to_string()),
            "symbol_references table missing (V6), got: {tables:?}",
        );
        // V7: git file changes
        assert!(
            tables.contains(&"git_file_changes".to_string()),
            "git_file_changes table missing (V7), got: {tables:?}",
        );

        // Verify V7 columns on workspaces
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(workspaces)")
            .expect("pragma")
            .query_map([], |row| row.get::<_, String>(1))
            .expect("query")
            .filter_map(|r| r.ok())
            .collect();
        assert!(
            cols.contains(&"git_analysis_enabled".to_string()),
            "git_analysis_enabled column missing (V7), got: {cols:?}",
        );
        assert!(
            cols.contains(&"auto_reanalyze_enabled".to_string()),
            "auto_reanalyze_enabled column missing (V7), got: {cols:?}",
        );
    }
}
