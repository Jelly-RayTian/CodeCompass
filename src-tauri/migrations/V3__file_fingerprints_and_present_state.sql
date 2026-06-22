-- V3__file_fingerprints_and_present_state.sql
-- Milestone 2: real repository scanning, extension filtering, metadata
-- fingerprints, present/deleted state, and incremental change tracking.

ALTER TABLE indexed_files ADD COLUMN fingerprint TEXT;
ALTER TABLE indexed_files ADD COLUMN previous_fingerprint TEXT;
ALTER TABLE indexed_files ADD COLUMN is_present INTEGER NOT NULL DEFAULT 1;
ALTER TABLE indexed_files ADD COLUMN change_status TEXT NOT NULL DEFAULT 'unchanged';

CREATE INDEX idx_indexed_files_present
    ON indexed_files (workspace_id, is_present);

CREATE INDEX idx_indexed_files_change_status
    ON indexed_files (workspace_id, change_status);
