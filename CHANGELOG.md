# Changelog

All notable changes to CodeCompass are documented in this file.

## [0.1.1] — 2026-07-07

### Stability & bug fixes

- Fixed variable-name typo in scanner reconciliation path (`reconciliation_failed`).
- Fixed stale "Chronicle" product-name references in deletion comments.
- Split `LanguageContext.tsx` into `LanguageProvider.tsx`, `LangContext.ts`,
  `useT.ts`, and `types.ts` to satisfy React Fast Refresh and remove the ESLint
  `only-export-components` warning.
- Added React Router v7 future flags (`v7_startTransition`,
  `v7_relativeSplatPath`) to silence upgrade warnings.
- Wrapped `ErrorState` retry button with explicit `type="button"`.

### Error messages

- Added `AppError::FileNotFound` variant with stable code `file_not_found` and
  actionable user message explaining the file may have moved after the last scan.
- `read_source_file` now returns `file_not_found` instead of a generic
  `invalid_input` error when a source file is missing.

### Startup & responsiveness

- Scanner now emits `scan:progress` events every 10 files (in addition to the
  existing batch flush at 100 files), so the UI stays alive on large repositories.
- Analysis runner progress events now emit every 10 files instead of every 50.

### i18n

- `Graph.tsx` now uses translation keys instead of hardcoded English strings.
- `Insights.tsx` now uses translation keys for titles, empty states, and labels.
- Added `truncatedWarning`, and `insights.*` translation keys to both English and
  Chinese bundles.

### Testing

- Added `missing_source_file_returns_file_not_found` integration test.
- Added frontend `i18n.test.tsx` covering default language and `setLang` switch.
- Frontend test count: 8 → 10; Rust test count: 96 → 98; total: 104 → 108.

## [Unreleased]

### Icons

- Replaced Tauri placeholder icons with the original CodeCompass compass
  badge, regenerated via `npx tauri icon` from `icon-source.svg` (all
  PNG sizes, `.ico`, `.icns`, Windows Store square logos).

### Large-repository safety

- Dependency graph now **truncates** at 500 nodes with a `truncated` flag
  and UI warning instead of returning a hard error, so thousand-file
  repos degrade gracefully.
- New `totalGraphNodes` field reports the true participant count.

### Error messages

- `AppError` now exposes a stable `code()` and an actionable
  `user_message()` explaining what failed, likely cause, data-safety,
  and the next user step. Added `OversizedFile` variant.

### Testing

- 9 new failure-path integration tests (missing git, git failure, large
  file truncation, analysis cancellation, concurrent scan, deleted
  workspace, malformed UTF-8, interrupted-run recovery, graph
  truncation). Total Rust tests: 96.

### Performance

- Added a reproducible Criterion benchmark harness plus a single-shot
  summary runner generating 100/1,000/5,000-file fixtures at runtime.
- `docs/benchmarks.md` with measured results.

### Privacy

- Audited runtime network behavior. Found and **fixed** a Monaco Editor
  CDN loader default: Monaco is now bundled locally via Vite web workers
  (`src/lib/monacoConfig.ts`), preserving the no-network guarantee.
- New `docs/privacy-audit.md` with evidence.

### Release engineering

- Version-alignment script `scripts/check-versions.mjs` (package.json,
  Cargo.toml, tauri.conf.json, git tag).
- CI uses `npm ci`; least-privilege `permissions` on both workflows.
- Release workflow validates the tag version before building and clearly
  notes unsigned installers.

### Documentation

- Recruiter-friendly README with badges, logo, architecture, demo,
  privacy, benchmarks, and screenshot gallery placeholders.
- New `docs/architecture.md` (expanded), `docs/technical-decisions.md`,
  `docs/portfolio-overview.md`, `docs/benchmarks.md`,
  `docs/privacy-audit.md`.
- `LICENSE` file added (MIT).
- Test matrix, releasing, and smoke-test checklists updated with final
  totals and icon/privacy verification steps.

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
