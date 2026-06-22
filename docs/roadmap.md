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

## Milestone 3 — AST-based Import Analysis

**Status:** Complete

- Modular `LanguageAnalyzer` trait with `TypeScriptJavaScriptAnalyzer` implementation
- OXC-based AST parsing for `.ts`, `.tsx`, `.js`, `.jsx`
- Static imports, re-exports, dynamic imports, and CommonJS `require` extraction
- Relative import resolution with `.ts/.tsx/.js/.jsx` and index-file fallback
- External package detection (no `node_modules` traversal)
- Parse diagnostics that don't stop repository-wide analysis
- Persistent `imports` and `analysis_diagnostics` tables
- Incremental invalidation via `analysis_status` on `indexed_files`
- "Analyze" button and imports display in the Workspaces UI
- Rust tests for parser, resolver, imports, and diagnostics across all supported languages

## Milestone 4 — Structural Visualization

**Status:** Not started

- File dependency graph (interactive, zoomable)
- Directory tree view
- File detail panel (size, language, dependencies)
- Graph filtering by language, file type, directory
- Graph export (PNG/SVG)

## Milestone 4 — File Dependency Graph Visualization

**Status:** Complete

- Rust graph builder from persisted imports table
- Nodes (files) with incoming/outgoing edge counts, isolated-node handling
- Directed edges representing import relationships
- Circular dependency detection with explicit evidence
- React Flow interactive visualization with zoom, pan, fit-to-view
- Node selection with file details panel (imports, imported-by, diagnostics)
- Filename filter, cycle warnings, large-graph limit (500 nodes)
- Summary bar: total files, imports, graph nodes, cycle count
- Rust tests for graph construction, edge direction, cycles, isolated nodes

## Milestone 5 — Symbol Indexing and Search

**Status:** Complete

- OXC-based symbol extraction: functions, classes, methods, interfaces, types, enums, variables, React components
- V5 database migration: `symbols` table with source locations, parent hierarchy, visibility, export state
- Symbol search with pagination and kind/name filters
- File outline via `get_file_outline` command
- Symbol extraction integrated into analysis pipeline (runs with Analyze)
- Incremental replacement: file symbols deleted and re-inserted on re-analysis
- Symbol Search UI component with filter dropdowns and pagination
- Rust tests for all symbol kinds, source locations, malformed input

## Milestone 6 — Impact Analysis

## Milestone 6 — Impact Analysis

**Status:** Not started

- "What is affected?" query for a selected file or symbol
- Change impact highlighting on the graph
- Coupling and cohesion metrics
- Hotspot detection (high-churn, high-coupling files)

## Milestone 7 — Polish and Distribution

**Status:** Not started

- Auto-update mechanism
- Installer configuration (MSI/NSIS for Windows)
- Application icon and branding
- Keyboard shortcuts
- Theme support (light/dark)
- Performance optimization for large repositories
- Cross-platform testing (macOS, Linux)
