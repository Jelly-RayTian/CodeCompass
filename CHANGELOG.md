# Changelog

All notable changes to CodeCompass are documented in this file.

## [1.0.0] — 2026-06-23

### Foundation

- Tauri v2 + React 18 + TypeScript strict + Vite desktop application
- SQLite via rusqlite with refinery versioned migrations (V1–V7)
- Typed Rust error handling with `thiserror` and serde serialization
- Sidebar navigation: Home, Workspaces, Graph, Insights, Viewer, Settings

### Repository Scanning

- Recursive filesystem traversal with `walkdir`
- Ignore rules: `.git`, `node_modules`, `dist`, `build`, `.next`, `target`, etc.
- File extension filter: `.ts`, `.tsx`, `.js`, `.jsx`
- Metadata fingerprint (`size:mtime`) for incremental change detection
- Scan states: queued/running/completed/failed/cancelled/interrupted
- Progress events and cancellation support
- Last-successful-index preservation on failure

### AST Analysis

- OXC-based TypeScript/JavaScript parser for import extraction
- Static imports, re-exports, dynamic imports, CommonJS `require`
- Relative path resolution with extension and index-file fallback
- Symbol extraction: functions, classes, interfaces, types, enums, variables, React components
- Symbol references: function calls, `new` expressions, property access
- Call graph with focus mode, depth limits, and cycle detection
- Parse diagnostics that don't stop repository-wide analysis

### Dependency Graph

- File-level directed graph from persisted imports
- Node/edge counts, isolated files, cycle detection
- React Flow visualization with zoom, pan, node selection
- File details panel: imports, imported-by, diagnostics
- Filename and directory filters

### Symbol Search

- Symbol search with name/kind filters and pagination
- File outline for individual files
- Search results clickable to source viewer

### Code Viewer

- Monaco Editor in read-only mode
- Syntax highlighting (TS, TSX, JS, JSX)
- Line numbers, code folding, minimap
- Navigation from tree, graph, and symbol search
- Large-file safeguard (>1 MB truncated)

### Insights

- Heuristic entry-point detection with confidence scores
- BFS-based beginner reading path from entry points
- Structural findings: unresolved imports, large files, highly-connected modules, orphaned files, potentially unused exports
- Evidence, limitations, and investigation steps on every finding

### Impact Analysis

- Symbol-level call/reference graph
- Direct and transitive dependents
- Change risk scoring based on dependent count, export status, cycle participation
- "Potentially affected" wording — never presents heuristics as facts

### Git Integration

- Git repository detection without requiring Git for ordinary folders
- Branch, status, commit count, last commit info
- File change history and co-change hotspot detection
- Workspace settings: Git analysis toggle, auto-reanalysis toggle
- Safe subprocess invocation — no shell interpolation

### Internationalization

- Full Chinese/English UI translation
- Language switcher in navigation bar
- Persistent language preference via localStorage
- Auto-detection from browser language on first launch

### Distribution & Quality

- Windows MSI and NSIS installers via Tauri
- GitHub Actions CI: frontend (lint, typecheck, test, build) + Rust (fmt, clippy, test, check)
- 64 Rust unit tests + 5 frontend integration tests
- Migration chain verified V1→V7
- AGENTS.md with engineering rules and quality gates
