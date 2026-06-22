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
| `pages/`      | Top-level views: Home, Workspaces, Settings               |
| `components/` | Reusable UI: LoadingState, EmptyState, ErrorState         |
| `lib/`        | `tauriClient` (typed invoke wrapper), `useAsyncData` hook |
| `types/`      | Shared TypeScript interfaces mirroring Rust models        |
| `styles/`     | Global CSS                                                |
| `test/`       | Vitest setup, Tauri mock, test files                      |

**Key rule:** Frontend components never touch the filesystem or database
directly. All data access goes through `tauriClient`.

### Rust Backend (`src-tauri/src/`)

| Module      | Responsibility                                                               |
| ----------- | ---------------------------------------------------------------------------- |
| `commands/` | `#[tauri::command]` functions — thin wrappers, no business logic             |
| `db/`       | `Database` struct, migration runner, indexed-folder/file/scan-run DAOs       |
| `models/`   | Serde structs mirroring frontend types                                       |
| `error.rs`  | `AppError` enum with `thiserror` + `serde::Serialize`                        |
| `platform/` | Path normalization and platform-aware path comparisons                       |
| `scanner/`  | Recursive metadata-only traversal with `ScanCallbacks` trait for testability |
| `tasks/`    | `ScanManager` and `AnalysisManager` for cancellation tokens                  |
| `analysis/` | Code analysis: `LanguageAnalyzer` trait, TS/JS parser (oxc), import resolver |

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
          → scanner::run_scan(&db, folderId, runId, cancelToken, AppHandle)
            → WalkDir over registered root (no symlinks, ignored dirs skipped)
            → filter to supported extensions
            → validate root containment per entry
            → compute metadata fingerprint (size + mtime)
            → upsert indexed_files rows in batches with change_status
            → emit 'scan:progress' events
            → update scan_runs progress counters
        ← ScanRun
      ← serialized as JSON
    ← Promise<ScanRun>
  → receive 'scan:progress' events for live updates
  → poll getScanStatus(folderId) as fallback
  → show phase, files processed, warnings, errors
  → on completion, call listWorkspaceFiles + listScanRuns to render tree/history
```

## Error Handling

Rust: `AppError` (thiserror) covers `rusqlite::Error`, `refinery::Error`,
`DatabaseLock`, `AppDir`, I/O errors, path errors, and scan errors. It
implements `serde::Serialize` so Tauri can send the error string to the
frontend. Commands return `Result<T, AppError>`.

Frontend: `useAsyncData` hook catches rejected promises and sets an error
state. Pages render `ErrorState` with a retry button.

## Database Access

`Database` wraps `Mutex<rusqlite::Connection>`. The mutex is necessary because
`Connection` is `Send` but not `Sync`. Tauri stores `Database` as managed state
(`app.manage(database)`), making it available to all command handlers via
`State<Database>`. A separate `ScanManager` is also managed state and holds
cancellation tokens for active scans.

Migrations are embedded at compile time via `refinery::embed_migrations!` and
run automatically when `Database::open` is called.

## TypeScript Strictness

The project enables `strict`, `noUncheckedIndexedAccess`, and
`exactOptionalPropertyTypes`. This means:

- Array access returns `T | undefined` — callers must narrow.
- Optional props require `| undefined` explicitly — no implicit omission.
- No `any` anywhere (enforced by ESLint).
