-- V1__initial_schema.sql
-- Initial CodeCompass database schema.

CREATE TABLE workspaces (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    path            TEXT    NOT NULL UNIQUE,
    created_at      INTEGER NOT NULL,
    last_opened_at  INTEGER
);

CREATE TABLE indexed_files (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id    INTEGER NOT NULL,
    relative_path   TEXT    NOT NULL,
    file_hash       TEXT,
    language        TEXT,
    size_bytes      INTEGER,
    last_indexed_at INTEGER NOT NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces (id) ON DELETE CASCADE,
    UNIQUE (workspace_id, relative_path)
);

CREATE TABLE analysis_runs (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id   INTEGER NOT NULL,
    status         TEXT    NOT NULL DEFAULT 'pending',
    started_at     INTEGER NOT NULL,
    completed_at   INTEGER,
    error_message  TEXT,
    FOREIGN KEY (workspace_id) REFERENCES workspaces (id) ON DELETE CASCADE
);

CREATE TABLE app_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX idx_indexed_files_workspace
    ON indexed_files (workspace_id);

CREATE INDEX idx_analysis_runs_workspace
    ON analysis_runs (workspace_id);
