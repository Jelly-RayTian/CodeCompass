# Roadmap

## Milestone 0 — Foundation (current)

**Status:** Complete

- Tauri + React + TypeScript + Vite desktop application
- Strict TypeScript, ESLint, Prettier, Vitest, React Testing Library
- SQLite with refinery migrations
- Initial schema: workspaces, indexed_files, analysis_runs, app_settings
- Tauri commands: get_application_info, get_database_status, list_workspaces
- Application shell: Home, Workspaces, Settings pages
- Loading, empty, and error states
- Frontend and Rust tests
- Documentation and CI

## Milestone 1 — Indexed Folders and Metadata Scanning

**Status:** Complete

- Native folder picker dialog (`tauri-plugin-dialog`)
- Explicit indexed-folder registration with normalized paths
- Duplicate and nested-folder detection
- Indexed-folder persistence in SQLite
- Availability, monitoring status, and scan status per folder
- Remove folder with confirmation (deletes Chronicle index only)
- Native Rust metadata scanner (`walkdir`)
- Root-containment validation, symlink skipping, no content reading
- Per-file error recovery, cancellation, progress reporting
- `scan_runs` table with counters and final summary
- Frontend list/progress UI without loading full file lists
- Comprehensive Rust tests

## Milestone 2 — Repository Scanning and Persistent File Indexing

**Status:** Complete

- Real Rust repository scanner over registered workspace roots
- Root-containment, path normalization, recursive traversal, symlink skipping
- Ignore `.git`, `node_modules`, `dist`, `build`, `coverage`, `.next`, `out`, `target`, `vendor`, `.idea`
- Index `.ts`, `.tsx`, `.js`, `.jsx` files
- Store relative path, extension, size, modification time, indexed time, fingerprint, present/deleted state
- Incremental change detection: unchanged / changed / new / removed
- `scan_runs` with queued/running/completed/failed/cancelled/interrupted states
- Progress events, cancellation, and responsive UI
- Preserve last successful index on failure or cancellation
- Indexed file tree, selected-file metadata, scan summary, scan history
- Rust tests for ignored dirs, unsupported files, empty repo, symlinks, traversal, cancellation, incremental changes

## Milestone 3 — Structural Visualization

**Status:** Not started

- File dependency graph (interactive, zoomable)
- Directory tree view
- File detail panel (size, language, dependencies)
- Graph filtering by language, file type, directory
- Graph export (PNG/SVG)

## Milestone 4 — Symbol Analysis

**Status:** Not started

- Symbol extraction (functions, classes, methods, imports)
- Symbol-level dependency graph
- Entry point detection (main functions, exported APIs)
- Call graph visualization
- Search across symbols

## Milestone 5 — Impact Analysis

**Status:** Not started

- "What is affected?" query for a selected file or symbol
- Change impact highlighting on the graph
- Coupling and cohesion metrics
- Hotspot detection (high-churn, high-coupling files)

## Milestone 6 — Polish and Distribution

**Status:** Not started

- Auto-update mechanism
- Installer configuration (MSI/NSIS for Windows)
- Application icon and branding
- Keyboard shortcuts
- Theme support (light/dark)
- Performance optimization for large repositories
- Cross-platform testing (macOS, Linux)
