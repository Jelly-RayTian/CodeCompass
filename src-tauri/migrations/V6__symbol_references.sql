-- V6: Symbol-level references (calls, instantiations, property accesses).
--
-- Records statically resolved references between symbols within and across
-- files. Supports the call-graph and impact-analysis features.

CREATE TABLE IF NOT EXISTS symbol_references (
    id                     INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id           INTEGER NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    caller_symbol_id       INTEGER REFERENCES symbols(id) ON DELETE CASCADE,
    callee_name            TEXT    NOT NULL,
    caller_file_id         INTEGER NOT NULL REFERENCES indexed_files(id) ON DELETE CASCADE,
    resolved_callee_symbol_id INTEGER REFERENCES symbols(id) ON DELETE SET NULL,
    reference_type         TEXT    NOT NULL,
    source_line            INTEGER NOT NULL,
    source_column          INTEGER NOT NULL,
    created_at             INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_refs_caller ON symbol_references(caller_symbol_id);
CREATE INDEX IF NOT EXISTS idx_refs_callee ON symbol_references(resolved_callee_symbol_id);
CREATE INDEX IF NOT EXISTS idx_refs_workspace ON symbol_references(workspace_id);
