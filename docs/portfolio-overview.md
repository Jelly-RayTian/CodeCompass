# Portfolio Overview

A concise, grounded summary of CodeCompass for resume, interview, and
recruiter use. Every claim below is backed by code in this repository.

## One-Paragraph Summary

CodeCompass is a local-first desktop application (Tauri + React + Rust +
SQLite) that analyzes TypeScript/JavaScript repositories to help
developers understand unfamiliar codebases. It scans a folder, parses
every `.ts/.tsx/.js/.jsx` file with the OXC AST parser, persists import
relationships, symbols, and call references to a SQLite index, and
presents an interactive dependency graph, symbol search, code viewer,
and structural insights — all entirely offline, with no telemetry and no
source-code upload.

## Major Technical Achievements

- **End-to-end analysis pipeline.** Metadata scan → AST parse → import
  resolution → symbol/reference extraction → graph construction →
  insights — all in one local app, with 96 passing Rust tests and a
  Vitest frontend suite.
- **Safe incremental rescans.** A monotonic `scan_generation` counter
  (migration V8) makes deletion reconciliation correct even when two
  scans complete in the same second. Cancelled and error-degraded scans
  preserve the previous snapshot rather than deleting files.
- **Large-repo safety.** The dependency graph truncates at 500 nodes
  with a `truncated` flag and a UI warning, so thousand-file repos
  degrade gracefully instead of freezing.
- **Offline Monaco.** The Monaco Editor runtime is bundled via Vite web
  workers (`src/lib/monacoConfig.ts`) rather than loaded from a CDN,
  preserving the no-network privacy guarantee.
- **Reproducible benchmarks.** A Criterion benchmark harness plus a
  single-shot summary runner generate fixtures of 100/1,000/5,000 files
  at runtime and measure scan, analysis, and graph-construction time.
- **Hardened release engineering.** Version-alignment script, `npm ci`
  in CI, least-privilege workflow permissions, required-NSIS validation,
  and unsigned-installer warnings in the release notes.

## Hardest Engineering Problems

1. **Deletion reconciliation correctness.** Naively "files not seen this
   scan = deleted" is wrong when the scan is cancelled or hits permission
   errors. The solution combines a generation counter, status-gated
   reconciliation, and degradation to `completed_with_errors` if
   reconciliation itself fails. Five regression tests pin this behaviour.
2. **Cross-thread SQLite.** `rusqlite::Connection` is `Send` not `Sync`,
   so all access goes through `Mutex<Connection>` managed as Tauri state.
   Batching writes in transactions of 100 keeps lock contention low.
3. **AST analysis of untrusted source.** Malformed files must not crash
   the run. OXC parses in-memory; failures produce diagnostics and a
   `parse_error` status, then the runner continues with the next file.
4. **Privacy audit.** Discovering that `@monaco-editor/loader` defaults
   to a CDN, and fixing it by bundling Monaco locally with Vite worker
   imports, was the difference between a true and a false "no network"
   claim.

## Testing Strategy

- **Rust:** 96 tests — 78 unit (scanner, db DAOs, analysis, resolver,
  graph, symbols, references, platform, error), 9 fixture integration
  (full scan+analyze+persistence lifecycle), 9 failure-path (missing
  git, large file truncation, cancellation, concurrent scan, deleted
  workspace, malformed UTF-8, interrupted-run recovery, graph
  truncation). All use temp directories and temp SQLite databases.
- **Frontend:** Vitest with jsdom, a Tauri mock, and
  `@testing-library/react` covering app shell, navigation, DB status,
  empty states, and error/retry paths.
- **Quality gates:** `cargo fmt --check`, `cargo clippy --all-targets
  -- -D warnings`, `cargo test`, `cargo check`, `npm run lint`,
  `npm run typecheck` (strict + `noUncheckedIndexedAccess` +
  `exactOptionalPropertyTypes`), `npm run test`, `npm run build`.
- **CI:** GitHub Actions on Windows runs the full suite on every push
  and PR. The release workflow re-runs everything and validates the tag
  version before building installers.

## Performance Strategy

- **Metadata-only scanning** (never reads file contents during a scan).
- **Batched SQLite upserts** in transactions of 100.
- **Indexed queries** — every foreign key and common filter has an index.
- **Graph truncation** bounds response size for large repos.
- **1 MB viewer cap** prevents loading huge files into Monaco.
- **Reproducible benchmarks** (Criterion + fixture generator) provide
  evidence rather than guesses. See [docs/benchmarks.md](benchmarks.md).

## What the Developer Should Be Able to Explain in an Interview

- The scan → analyze → graph data flow and where each piece lives
  (scanner, OXC parser, SQLite, graph builder, React Flow).
- Why `scan_generation` exists and what goes wrong with timestamp-only
  deletion detection.
- Why `Mutex<Connection>` is needed and how batching keeps it fast.
- How path traversal is prevented (`path_is_inside_or_equal`, root
  containment check per entry).
- How cancellation preserves already-indexed files without a full
  rollback.
- How the Monaco CDN issue was found and fixed.
- The tradeoffs of Tauri vs Electron and of SQLite vs a server DB.
- Why the graph truncates at 500 nodes and what the alternatives were.

## Suggested Resume Bullet Points

- Built a local-first desktop codebase-analysis tool with Tauri, React,
  TypeScript, and Rust, analyzing TypeScript/JavaScript repositories
  entirely offline with no telemetry.
- Implemented a metadata-only repository scanner with incremental change
  detection and generation-based deletion reconciliation, backed by 96
  passing Rust tests.
- Integrated the OXC AST parser to extract imports, symbols, and call
  references, resolving relative specifiers with path-traversal
  protection.
- Designed an 8-migration SQLite schema (WAL mode, indexed) with
  refinery for embedded, ordered, versioned migrations.
- Added large-repository safety via graph truncation, a 1 MB viewer cap,
  and batched transactional upserts.
- Hardened release engineering with a version-alignment script,
  least-privilege CI, and unsigned-instler warnings.
- Wrote a reproducible Criterion benchmark harness generating 100–5,000
  file fixtures at runtime to measure scan/analysis/graph performance.
- Audited and fixed a privacy gap: bundled the Monaco Editor runtime
  locally to eliminate CDN network requests.

## Suggested Interview Talking Points

- "Walk me through what happens when the user clicks Scan." — Cover the
  command, background thread, WalkDir, batching, progress events,
  fingerprinting, and reconciliation.
- "How do you handle a scan that's cancelled halfway?" — Persisted
  batches, skipped reconciliation, `cancelled` status, previous snapshot
  preserved.
- "How do you keep the UI responsive on a 5,000-file repo?" — Graph
  truncation, batched writes, metadata-only scan, 1 MB viewer cap.
- "How do you know the app makes no network requests?" — The privacy
  audit: static search, Monaco CDN fix, dependency review, capability
  review.
- "Why Rust instead of Node for the backend?" — Speed, safety, single
  process, testability via the `ScanCallbacks` trait.
- "How would you add a new language?" — Implement `LanguageAnalyzer`,
  add supported extensions, add a resolver; the DB schema is
  language-agnostic.
