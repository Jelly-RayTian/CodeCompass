-- V4: Import relationships and analysis diagnostics.
--
-- Records static imports, re-exports, dynamic imports, and CommonJS require
-- statements discovered during AST analysis, together with parse diagnostics.

CREATE TABLE IF NOT EXISTS imports (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    source_file_id  INTEGER NOT NULL REFERENCES indexed_files(id) ON DELETE CASCADE,
    target_specifier TEXT   NOT NULL,
    resolved_target_file_id INTEGER REFERENCES indexed_files(id) ON DELETE SET NULL,
    import_type     TEXT    NOT NULL,
    is_external     INTEGER NOT NULL DEFAULT 0,
    start_line      INTEGER,
    start_column    INTEGER,
    created_at      INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_imports_source_file ON imports(source_file_id);
CREATE INDEX IF NOT EXISTS idx_imports_target_file ON imports(resolved_target_file_id);

CREATE TABLE IF NOT EXISTS analysis_diagnostics (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id      INTEGER NOT NULL REFERENCES indexed_files(id) ON DELETE CASCADE,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    severity     TEXT    NOT NULL,
    message      TEXT    NOT NULL,
    line         INTEGER,
    "column"     INTEGER,
    created_at   INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_analysis_diagnostics_file
    ON analysis_diagnostics(file_id);
CREATE INDEX IF NOT EXISTS idx_analysis_diagnostics_workspace
    ON analysis_diagnostics(workspace_id, severity);

-- Analysis status tracking on indexed_files.
ALTER TABLE indexed_files ADD COLUMN analyzed_at     INTEGER;
ALTER TABLE indexed_files ADD COLUMN analysis_status TEXT NOT NULL DEFAULT 'pending';
