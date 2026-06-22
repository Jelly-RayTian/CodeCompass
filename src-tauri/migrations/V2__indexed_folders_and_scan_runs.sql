-- V2__indexed_folders_and_scan_runs.sql
-- Milestone 1: indexed-folder management and native metadata scanning.

-- Extend workspaces so each row represents a user-registered indexed folder.
ALTER TABLE workspaces ADD COLUMN display_name TEXT;
ALTER TABLE workspaces ADD COLUMN added_at INTEGER;
ALTER TABLE workspaces ADD COLUMN last_successful_scan_at INTEGER;
ALTER TABLE workspaces ADD COLUMN availability TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE workspaces ADD COLUMN monitoring_enabled INTEGER NOT NULL DEFAULT 0;
ALTER TABLE workspaces ADD COLUMN scan_status TEXT NOT NULL DEFAULT 'idle';

-- Extend indexed_files with the metadata fields required for Milestone 1.
-- file_hash and language remain from V1 but are unused in this milestone.
ALTER TABLE indexed_files ADD COLUMN name TEXT;
ALTER TABLE indexed_files ADD COLUMN parent_path TEXT;
ALTER TABLE indexed_files ADD COLUMN extension TEXT;
ALTER TABLE indexed_files ADD COLUMN created_at INTEGER;
ALTER TABLE indexed_files ADD COLUMN modified_at INTEGER;
ALTER TABLE indexed_files ADD COLUMN indexed_at INTEGER;
ALTER TABLE indexed_files ADD COLUMN last_seen_at INTEGER;

CREATE INDEX idx_indexed_files_workspace_path
    ON indexed_files (workspace_id, relative_path);

CREATE INDEX idx_indexed_files_last_seen
    ON indexed_files (workspace_id, last_seen_at);

-- Scan runs record the progress and outcome of a single metadata scan.
CREATE TABLE scan_runs (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id      INTEGER NOT NULL,
    status            TEXT    NOT NULL DEFAULT 'pending',
    started_at        INTEGER NOT NULL,
    completed_at      INTEGER,
    files_processed   INTEGER NOT NULL DEFAULT 0,
    files_indexed     INTEGER NOT NULL DEFAULT 0,
    warning_count     INTEGER NOT NULL DEFAULT 0,
    error_count       INTEGER NOT NULL DEFAULT 0,
    error_message     TEXT,
    phase             TEXT,
    FOREIGN KEY (workspace_id) REFERENCES workspaces (id) ON DELETE CASCADE
);

CREATE INDEX idx_scan_runs_workspace
    ON scan_runs (workspace_id);

CREATE INDEX idx_scan_runs_started
    ON scan_runs (workspace_id, started_at DESC);
