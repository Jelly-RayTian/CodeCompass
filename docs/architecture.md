# Architecture

## High-Level Diagram

```
┌──────────────────────────────────────────┐
│              React Frontend               │
│  ┌──────┐  ┌─────────┐  ┌──────────────┐ │
│  │ Pages│  │Components│  │ tauriClient  │ │
│  └──┬───┘  └─────────┘  └──────┬───────┘ │
│     │                           │ invoke  │
│     └───────────────────────────┤         │
└─────────────────────────────────┼────────┘
                                  │ Tauri IPC
┌─────────────────────────────────┼────────┐
│              Rust Backend        │        │
│  ┌────────────────┐  ┌──────────┴──────┐ │
│  │   Commands      │  │  Core Logic     │ │
│  │  (thin wrappers)│  │  db · models    │ │
│  └────────────────┘  │  scanner · tasks │ │
│                      └─────────────────┘ │
│  ┌─────────────────────────────────────┐ │
│  │     SQLite (rusqlite, bundled)      │ │
│  └─────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

## Layer Responsibilities

### Frontend (`src/`)

| Directory     | Responsibility                                            |
| ------------- | --------------------------------------------------------- |
| `app/`        | Application shell, navigation, route definitions          |
| `pages/`      | Top-level views: Home, Workspaces, Settings, Graph, Viewer, Insights |
| `components/` | Reusable UI: LoadingState, EmptyState, ErrorState, CodeViewer |
| `lib/`        | `tauriClient` (typed invoke wrapper), `useAsyncData` hook, `monacoConfig` (offline Monaco) |
| `types/`      | Shared TypeScript interfaces mirroring Rust models        |
| `styles/`     | Global CSS (system fonts, no remote resources)            |
| `test/`       | Vitest setup, Tauri mock, test files                      |

**Key rule:** Frontend components never touch the filesystem or database
directly. All data access goes through `tauriClient`.

### Rust Backend (`src-tauri/src/`)

| Module      | Responsibility                                                               |
| ----------- | ---------------------------------------------------------------------------- |
| `commands/` | `#[tauri::command]` functions — thin wrappers, no business logic             |
| `db/`       | `Database` struct, migration runner, indexed-folder/file/scan-run DAOs, imports, symbols, references, workspace settings |
| `models/`   | Serde structs mirroring frontend types                                       |
| `error.rs`  | `AppError` enum with `thiserror` + `serde::Serialize` + actionable `user_message()` |
| `platform/` | Path normalization and platform-aware path comparisons                       |
| `scanner/`  | Recursive metadata-only traversal with `ScanCallbacks` trait for testability |
| `tasks/`    | `ScanManager` and `AnalysisManager` for cancellation tokens                  |
| `analysis/` | `LanguageAnalyzer` trait, TS/JS parser (OXC), import resolver, graph builder, symbols, references, entrypoint detection, reading paths, findings, call graph, impact |
| `git/`      | Safe `git` subprocess invocation (argument lists, never shell interpolation) |

**Key rule:** Commands delegate to core logic. `commands/workspaces.rs` calls
`fetch_indexed_folders(&db)`, not the other way around. This keeps the core
logic testable without Tauri's `State` wrapper.

## Data Flow Example: scanning an indexed folder

```
React (Workspaces page)
  → listens to 'scan:progress' Tauri events
  → tauriClient.startScan(folderId)
    → invoke('start_scan')
      → commands::start_scan(State<Database>, State<ScanManager>)
        → create_scan_run(&db, folderId) + register cancellation token
        → spawn background thread
          → scanner::scan_workspace(&db, folderId, runId, cancelToken, &callbacks)
            → WalkDir over registered root (no symlinks, ignored dirs skipped)
            → filter to supported extensions
            → validate root containment per entry
            → compute metadata fingerprint (size + mtime)
            → upsert indexed_files rows in batches of 100 with change_status
            → emit 'scan:progress' events
            → update scan_runs progress counters
        ← ScanSummary
      ← serialized as JSON
    ← Promise<ScanRun>
  → receive 'scan:progress' events for live updates
  → poll getScanStatus(folderId) as fallback
  → show phase, files processed, warnings, errors
  → on completion, call listWorkspaceFiles + listScanRuns to render tree/history
```

## Scanner

`scanner::scan_workspace` is the single production-quality scanner used by
both the live app (via `run_scan`, which wraps it with `TauriCallbacks`)
and tests (via `NoopCallbacks`). It performs **metadata-only** traversal —
file contents are never read during a scan.

### Ignored directories

`node_modules`, `.git`, `dist`, `build`, `coverage`, `.next`, `out`,
`target`, `vendor`, `.idea`. Symlinks are never followed.

### Batching

Files are upserted in batches of 100 inside a SQLite transaction per
batch, which keeps lock contention low on large repos.

## AST Analysis (OXC)

`analysis/ts_js.rs` implements `LanguageAnalyzer` for TypeScript and
JavaScript using the `oxc_parser` / `oxc_ast` / `oxc_span` / `oxc_allocator`
crates (v0.45). The analyzer extracts:

- Static imports (`import x from '...'`)
- Named and namespace imports
- Re-exports (`export ... from '...'`, `export * from`)
- CommonJS `require('...')`
- Dynamic `import('...')`

`analysis/resolver.rs` resolves relative specifiers to indexed files,
falling back to `index` files and detecting external packages. Path
traversal is rejected by `platform::path_is_inside_or_equal`.

`analysis/symbols.rs` extracts functions, classes, interfaces, type
aliases, enums, arrow functions, and React components with source
locations. `analysis/references.rs` extracts call-site references for the
call-graph / impact features.

Malformed files produce diagnostics but **never block** analysis of the
rest of the workspace — the runner records a `parse_error` status and
continues.

## SQLite Schema

Eight ordered, versioned migrations (`V1`–`V8`) embedded at compile time
via `refinery::embed_migrations!` and run automatically on
`Database::open`.

Key tables:

| Table                 | Purpose                                              |
| --------------------- | ---------------------------------------------------- |
| `workspaces`          | Indexed folders (path, scan/analysis status, flags) |
| `indexed_files`       | File metadata + fingerprints + change status + generation |
| `imports`             | Import relationships (source, target, type, external)|
| `analysis_diagnostics`| Per-file parse diagnostics                           |
| `symbols`             | Extracted symbols (name, kind, location, parent)    |
| `symbol_references`   | Caller→callee edges for call graph                   |
| `git_file_changes`    | Co-change history from `git log`                     |
| `scan_runs`           | Scan run records (status, counts, timestamps)       |
| `app_settings`        | Key-value (scan generation counters, prefs)         |

WAL mode is enabled for concurrent read access. Indexes exist on all
foreign-key and common filter columns (see the migration files for the
full list).

See [docs/database.md](database.md) for the schema diagram.

## Graph Construction

`analysis/graph::build_graph` builds a file-level dependency graph from
the `imports` table. Nodes are files that participate in imports; edges
are resolved import relationships. Cycle detection uses DFS with
colour marks and returns at most 20 cycles.

**Large-repo safety:** when more than `MAX_GRAPH_NODES` (500) files
participate in imports, the graph is **truncated** — only the first 500
nodes (by relative path) are returned, `truncated` is set, and
`total_graph_nodes` reports the true count. Edges and cycles are computed
over the returned node set only, bounding response size. The frontend
shows a warning banner and offers path/directory filters to narrow the
view.

## Persistence Flow

1. `Database::open` creates the app-data directory if needed.
2. Refinery runs all pending migrations in order.
3. `mark_interrupted_runs` marks any scan left "running" by a previous
   crash as "interrupted" — the app never starts in an inconsistent state.
4. `Database` is stored as Tauri managed state (`app.manage(database)`).
5. All writes go through `Mutex<Connection>`; reads use the same
   connection under the lock.

## Cancellation

`tasks::ScanManager` and `tasks::AnalysisManager` hold `Arc<AtomicBool>`
cancellation tokens keyed by run id. The `cancel_scan` /
`cancel_analysis` commands set the token; the worker loop checks it
between files. On cancellation:

- Already-batched file upserts are **persisted** (they remain indexed).
- Deletion reconciliation is **skipped** (the previous complete snapshot
  is preserved).
- The run is marked `"cancelled"` and the folder returns to `"idle"`.

This means cancellation is **not** a database rollback — indexed files
stay visible, which matches user expectations.

## Generation-Based Reconciliation

`scan_generation` (migration V8) is a monotonic counter stored in
`app_settings`, incremented per workspace per scan. Each upserted file row
is stamped with the current generation. After a **complete** scan
(`"completed"` or `"completed_with_warnings"`), files whose generation is
older than the current one are marked `is_present = 0` /
`change_status = 'removed'`.

Reconciliation is **skipped** for `"completed_with_errors"` (the snapshot
may be incomplete due to permission failures) and for cancelled scans.
If reconciliation itself fails, the run is degraded to
`"completed_with_errors"` rather than silently ignoring the failure.

## Error Handling

Rust: `AppError` (thiserror) covers `rusqlite::Error`,
`refinery::Error`, `DatabaseLock`, `AppDir`, I/O errors, path errors,
and scan errors. It implements `serde::Serialize` so Tauri can send the
error string to the frontend. Commands return `Result<T, AppError>`.
Each variant exposes a stable `code()` and an actionable
`user_message()` explaining what failed, the likely cause, and the next
step the user can take.

Frontend: `useAsyncData` hook catches rejected promises and sets an error
state. Pages render `ErrorState` with a retry button.

## Database Access

`Database` wraps `Mutex<rusqlite::Connection>`. The mutex is necessary
because `Connection` is `Send` but not `Sync`. Tauri stores `Database` as
managed state (`app.manage(database)`), making it available to all
command handlers via `State<Database>`. A separate `ScanManager` is also
managed state and holds cancellation tokens for active scans.

Migrations are embedded at compile time via `refinery::embed_migrations!`
and run automatically when `Database::open` is called. Released
migrations are never edited; new schema changes add a new `Vn__*.sql` file.

## TypeScript Strictness

The project enables `strict`, `noUncheckedIndexedAccess`, and
`exactOptionalPropertyTypes`. This means:

- Array access returns `T | undefined` — callers must narrow.
- Optional props require `| undefined` explicitly — no implicit omission.
- No `any` anywhere (enforced by ESLint).
