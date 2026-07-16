# Database Schema

CodeCompass uses SQLite (via `rusqlite` with the `bundled` feature, which
compiles SQLite from C source into the binary). Migrations are managed by
[refinery](https://github.com/rust-db/refinery) and embedded at compile time.

## Location

The database file (`codecompass.db`) is stored in the platform's application
data directory:

| OS      | Path                                                               |
| ------- | ------------------------------------------------------------------ |
| Windows | `%APPDATA%\com.codecompass.app\codecompass.db`                     |
| macOS   | `~/Library/Application Support/com.codecompass.app/codecompass.db` |
| Linux   | `~/.local/share/com.codecompass.app/codecompass.db`                |

The directory is created on first launch if it does not exist.

## Migrations

Migrations live in `src-tauri/migrations/` and follow the refinery naming
convention: `V{n}__{name}.sql`.

| File                                          | Description                                                                      |
| --------------------------------------------- | -------------------------------------------------------------------------------- |
| `V1__initial_schema.sql`                      | Creates the four initial tables and indexes                                      |
| `V2__indexed_folders_and_scan_runs.sql`       | Adds indexed-folder metadata columns, scan-run table, file metadata              |
| `V3__file_fingerprints_and_present_state.sql` | Adds file fingerprints, present/deleted state, and change tracking               |
| `V4__import_relationships.sql`                | Adds imports table, analysis_diagnostics table, analysis status on indexed_files |
| `V5__symbols.sql`                              | Adds symbols table with source locations, parent hierarchy, visibility |
| `V6__symbol_references.sql`                    | Adds symbol_references table for call-graph edges |
| `V7__workspace_settings_and_git_tracking.sql` | Adds workspace settings flags and git_file_changes table |
| `V8__scan_generation.sql`                      | Adds scan_generation column for safe deletion reconciliation |
| `V9__line_count.sql`                           | Adds line_count column to indexed_files for complexity metrics |
| `V9__line_count.sql`                           | Adds line_count column to indexed_files for complexity metrics |

Migrations are applied automatically when `Database::open` is called (at
application startup). Refinery tracks applied migrations in the
`refinery_schema_history` table.

## Tables

### `workspaces`

Represents a user-registered indexed folder. The table name is kept from
Milestone 0, but each row now represents an explicit indexed root.

| Column                    | Type    | Constraints               |
| ------------------------- | ------- | ------------------------- |
| `id`                      | INTEGER | PRIMARY KEY AUTOINCREMENT |
| `name`                    | TEXT    | NOT NULL                  |
| `path`                    | TEXT    | NOT NULL UNIQUE           |
| `created_at`              | INTEGER | NOT NULL (Unix epoch)     |
| `last_opened_at`          | INTEGER | nullable (Unix epoch)     |
| `display_name`            | TEXT    | nullable                  |
| `added_at`                | INTEGER | nullable (Unix epoch)     |
| `last_successful_scan_at` | INTEGER | nullable (Unix epoch)     |
| `availability`            | TEXT    | NOT NULL                  |
| `monitoring_enabled`      | INTEGER | NOT NULL DEFAULT 0        |
| `scan_status`             | TEXT    | NOT NULL DEFAULT 'idle'   |

### `indexed_files`

Files discovered during a metadata scan. File contents are **never** stored.

| Column                 | Type    | Constraints                           |
| ---------------------- | ------- | ------------------------------------- |
| `id`                   | INTEGER | PRIMARY KEY AUTOINCREMENT             |
| `workspace_id`         | INTEGER | NOT NULL, FK → workspaces(id) CASCADE |
| `relative_path`        | TEXT    | NOT NULL                              |
| `file_hash`            | TEXT    | nullable (unused in Milestone 1/2)    |
| `language`             | TEXT    | nullable (unused in Milestone 1/2)    |
| `size_bytes`           | INTEGER | nullable                              |
| `last_indexed_at`      | INTEGER | NOT NULL (Unix epoch)                 |
| `name`                 | TEXT    | nullable                              |
| `parent_path`          | TEXT    | nullable                              |
| `extension`            | TEXT    | nullable                              |
| `created_at`           | INTEGER | nullable (Unix epoch)                 |
| `modified_at`          | INTEGER | nullable (Unix epoch)                 |
| `indexed_at`           | INTEGER | nullable (Unix epoch)                 |
| `last_seen_at`         | INTEGER | nullable (Unix epoch)                 |
| `fingerprint`          | TEXT    | nullable                              |
| `previous_fingerprint` | TEXT    | nullable                              |
| `is_present`           | INTEGER | NOT NULL DEFAULT 1                    |
| `change_status`        | TEXT    | NOT NULL DEFAULT 'unchanged'          |

Unique constraint: `(workspace_id, relative_path)`.

### `scan_runs`

A single metadata scan pass over an indexed folder.

| Column            | Type    | Constraints                           |
| ----------------- | ------- | ------------------------------------- |
| `id`              | INTEGER | PRIMARY KEY AUTOINCREMENT             |
| `workspace_id`    | INTEGER | NOT NULL, FK → workspaces(id) CASCADE |
| `status`          | TEXT    | NOT NULL DEFAULT 'pending'            |
| `started_at`      | INTEGER | NOT NULL (Unix epoch)                 |
| `completed_at`    | INTEGER | nullable                              |
| `files_processed` | INTEGER | NOT NULL DEFAULT 0                    |
| `files_indexed`   | INTEGER | NOT NULL DEFAULT 0                    |
| `warning_count`   | INTEGER | NOT NULL DEFAULT 0                    |
| `error_count`     | INTEGER | NOT NULL DEFAULT 0                    |
| `error_message`   | TEXT    | nullable                              |
| `phase`           | TEXT    | nullable                              |

Status values used in Milestone 2: `queued`, `running`, `completed`,
`completed_with_warnings`, `completed_with_errors`, `cancelled`, `failed`,
`interrupted`.

### `analysis_runs`

A single analysis pass over a workspace (reserved for future milestones).

| Column          | Type    | Constraints                           |
| --------------- | ------- | ------------------------------------- |
| `id`            | INTEGER | PRIMARY KEY AUTOINCREMENT             |
| `workspace_id`  | INTEGER | NOT NULL, FK → workspaces(id) CASCADE |
| `status`        | TEXT    | NOT NULL DEFAULT 'pending'            |
| `started_at`    | INTEGER | NOT NULL (Unix epoch)                 |
| `completed_at`  | INTEGER | nullable                              |
| `error_message` | TEXT    | nullable                              |

### `imports`

Statically analysed import relationships between source files.

| Column                    | Type    | Constraints                               |
| ------------------------- | ------- | ----------------------------------------- |
| `id`                      | INTEGER | PRIMARY KEY AUTOINCREMENT                 |
| `source_file_id`          | INTEGER | NOT NULL, FK → indexed_files(id) CASCADE  |
| `target_specifier`        | TEXT    | NOT NULL                                  |
| `resolved_target_file_id` | INTEGER | nullable, FK → indexed_files(id) SET NULL |
| `import_type`             | TEXT    | NOT NULL                                  |
| `is_external`             | INTEGER | NOT NULL DEFAULT 0                        |
| `start_line`              | INTEGER | nullable                                  |
| `start_column`            | INTEGER | nullable                                  |
| `created_at`              | INTEGER | NOT NULL (Unix epoch)                     |

### `analysis_diagnostics`

Parse or resolution diagnostics from AST analysis.

| Column         | Type    | Constraints                              |
| -------------- | ------- | ---------------------------------------- |
| `id`           | INTEGER | PRIMARY KEY AUTOINCREMENT                |
| `file_id`      | INTEGER | NOT NULL, FK → indexed_files(id) CASCADE |
| `workspace_id` | INTEGER | NOT NULL, FK → workspaces(id) CASCADE    |
| `severity`     | TEXT    | NOT NULL                                 |
| `message`      | TEXT    | NOT NULL                                 |
| `line`         | INTEGER | nullable                                 |
| `column`       | INTEGER | nullable                                 |
| `created_at`   | INTEGER | NOT NULL (Unix epoch)                    |

### `indexed_files` (analysis columns added in V4)

| Column            | Type    | Constraints                |
| ----------------- | ------- | -------------------------- |
| `analyzed_at`     | INTEGER | nullable (Unix epoch)      |
| `analysis_status` | TEXT    | NOT NULL DEFAULT 'pending' |

### `app_settings`

Key-value store for application preferences.

| Column       | Type    | Constraints           |
| ------------ | ------- | --------------------- |
| `key`        | TEXT    | PRIMARY KEY           |
| `value`      | TEXT    | NOT NULL              |
| `updated_at` | INTEGER | NOT NULL (Unix epoch) |

## Indexes

| Name                               | Table           | Column(s)                       |
| ---------------------------------- | --------------- | ------------------------------- |
| `idx_indexed_files_workspace`      | `indexed_files` | `workspace_id`                  |
| `idx_indexed_files_workspace_path` | `indexed_files` | `workspace_id`, `relative_path` |
| `idx_indexed_files_last_seen`      | `indexed_files` | `workspace_id`, `last_seen_at`  |
| `idx_indexed_files_present`        | `indexed_files` | `workspace_id`, `is_present`    |
| `idx_indexed_files_change_status`  | `indexed_files` | `workspace_id`, `change_status` |
| `idx_scan_runs_workspace`          | `scan_runs`     | `workspace_id`                  |
| `idx_scan_runs_started`            | `scan_runs`     | `workspace_id`, `started_at`    |
| `idx_analysis_runs_workspace`      | `analysis_runs` | `workspace_id`                  |

## Design Decisions

- **Unix epoch integers** for timestamps (not ISO strings). Avoids a `chrono`
  dependency and keeps comparisons fast.
- **WAL journal mode** enabled on open for better concurrent read performance.
- **Foreign keys** enforced (`PRAGMA foreign_keys=ON`).
- **CASCADE delete** on workspace foreign keys so deleting a workspace removes
  its files, scan runs, and analysis runs automatically.
