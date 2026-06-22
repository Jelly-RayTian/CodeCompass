-- V5: Symbol indexing for search and outline navigation.
--
-- Stores extracted declarations (functions, classes, methods, interfaces,
-- type aliases, variables, exports, and React components) with source
-- locations and parent-child relationships.

CREATE TABLE IF NOT EXISTS symbols (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id     INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    file_id          INTEGER NOT NULL REFERENCES indexed_files(id) ON DELETE CASCADE,
    name             TEXT    NOT NULL,
    kind             TEXT    NOT NULL,
    parent_symbol_id INTEGER REFERENCES symbols(id) ON DELETE SET NULL,
    source_line      INTEGER NOT NULL,
    source_column    INTEGER NOT NULL,
    source_end_line  INTEGER NOT NULL,
    source_end_column INTEGER NOT NULL,
    signature        TEXT,
    visibility       TEXT    NOT NULL DEFAULT 'public',
    is_exported      INTEGER NOT NULL DEFAULT 0,
    created_at       INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_symbols_workspace_file
    ON symbols(workspace_id, file_id);

CREATE INDEX IF NOT EXISTS idx_symbols_workspace_kind
    ON symbols(workspace_id, kind);

CREATE INDEX IF NOT EXISTS idx_symbols_name_search
    ON symbols(workspace_id, name COLLATE NOCASE);

CREATE INDEX IF NOT EXISTS idx_symbols_parent
    ON symbols(parent_symbol_id);
