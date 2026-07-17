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

## Milestone 6 — Read-only Code Viewer & Navigation

**Status:** Complete

- Monaco Editor integration with syntax highlighting, line numbers, search, folding
- Read-only mode — no source editing or execution
- Navigation from file tree, dependency graph, and symbol search to source viewer
- File metadata and import/reference side panels
- Large-file safeguard (>1 MB truncated with warning)
- Path containment validation on every read
- Imports and "referenced by" lists in the viewer side panel

## Milestone 7 — Entry Points, Reading Paths & Structural Findings

**Status:** Complete

- Heuristic entry-point detection (filenames, directories, import-degree, naming)
- Confidence scores with explicit reasons — never presents heuristics as facts
- BFS-based beginner reading path from entry points with depth tracking
- Structural findings: unresolved imports, large files, highly-connected modules, orphaned files, potentially unused exports
- Every finding includes evidence, limitations, and investigation steps
- Insights page with entry point list, numbered reading path, and categorized findings
- No fake quality scores or AI-generated text

## Milestone 8 — Call Graph & Impact Analysis

**Status:** Complete

- AST-based symbol reference extraction (calls, `new` expressions, property access)
- V6 migration: `symbol_references` table with caller/callee resolution
- Call graph with focus mode and depth limits, cycle detection
- Change impact analysis: direct + transitive dependents, affected files/symbols
- Risk formula based on dependent count, export status, and cycle participation
- Cautious wording: "potentially affected based on statically detected references"
- Evidence, limitations, and static-analysis caveats on every finding
- Frontend API for call graph + impact queries

## Milestone 13 — Performance Optimization

**Status:** Complete

- **Incremental analysis**: removed workspace-level clearing + blanket `mark_pending_analysis`
- Files unchanged after rescan skip analysis entirely
- **Analyze speedup**: 8.5× at 5,000 files (24.1s → 2.8s), 16.7× at 1,000 files
- Scanner batch size 100 → 500 (5× fewer SQLite transactions)
- SQLite `PRAGMA synchronous=NORMAL` + 8 MB page cache
- Progress interval 10 → 50 files (reduced IPC overhead)
- Updated benchmarks with before/after comparison

## Milestone 12 — Plugin Architecture

**Status:** Complete

- Enhanced `LanguageAnalyzer` trait with `name()`, `version()`, `description()` metadata
- `AnalyzerRegistry` mapping file extensions to analyzers via `Arc<dyn LanguageAnalyzer>`
- Scanner and runner both use registry (no more hardcoded extension lists)
- Dynamic SQL `IN (...)` clause built from registry extensions
- CSS analyzer reference plugin with 7 unit tests
- `get_plugin_info` Tauri command
- Plugin architecture documentation (`docs/plugin-architecture.md`)
- 12 new Rust tests (117 total)

## Milestone 11 — Git Evolution Dashboard

**Status:** Complete

- Evolution dashboard page with commit timeline, file churn, and co-change hotspots
- Commit timeline bar chart (monthly buckets) from `git_file_changes`
- Top-20 file churn ranking with proportional bars
- Co-change hotspot cards
- Summary statistics: total commits, files, changes, active month, date range
- `git::recent_file_changes` fixed to return real timestamps (was always 0)
- Commit depth increased from 50 to 200
- New `git::commit_log` function for future use
- New `analysis::evolution` module with `build_evolution_report`
- English and Chinese i18n
- 3 new Rust tests (105 total)

## Milestone 10 — Repository Health Dashboard

**Status:** Complete

- Health dashboard page with summary cards and risk distribution
- Per-file composite risk score from size, line count, coupling, churn, diagnostics
- Cycle-bonus risk adjustment for files in circular dependencies
- Top-N risk files table and full sortable file list
- V9 migration: line_count column on indexed_files
- Line counting integrated into analysis runner
- English and Chinese i18n
- Rust tests for health report, risk scoring, cycle flagging

## Milestone 9 — Git Integration & Incremental Analysis

**Status:** Complete

- Auto-update mechanism
- Installer configuration (MSI/NSIS for Windows)
- Application icon and branding
- Keyboard shortcuts
- Theme support (light/dark)
- Performance optimization for large repositories
- Cross-platform testing (macOS, Linux)
