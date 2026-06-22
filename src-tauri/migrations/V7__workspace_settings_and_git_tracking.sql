-- V7: Workspace settings and git tracking.
--
-- Adds per-workspace configuration flags and a simple churn-change table
-- for hotspot detection.

ALTER TABLE workspaces ADD COLUMN git_analysis_enabled INTEGER NOT NULL DEFAULT 0;
ALTER TABLE workspaces ADD COLUMN auto_reanalyze_enabled INTEGER NOT NULL DEFAULT 0;

-- Record of files changed in each commit (for hotspot / co-change detection).
CREATE TABLE IF NOT EXISTS git_file_changes (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    commit_hash   TEXT    NOT NULL,
    relative_path TEXT    NOT NULL,
    timestamp     INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_git_changes_workspace
    ON git_file_changes(workspace_id, relative_path);
CREATE INDEX IF NOT EXISTS idx_git_changes_commit
    ON git_file_changes(workspace_id, commit_hash);
