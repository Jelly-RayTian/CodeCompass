# Technical Decisions

A record of the major technology and architecture choices in CodeCompass,
with the reasoning and tradeoffs behind each.

## Why Tauri instead of Electron

**Decision:** Tauri 2.x.

**Why:**
- **Binary size & memory.** Tauri ships a small Rust binary and reuses the
  OS WebView (WebView2 on Windows) rather than bundling a full Chromium.
  CodeCompass installers are tens of MB, not hundreds.
- **Rust backend.** I wanted a single language for the heavy lifting
  (filesystem traversal, AST parsing, SQLite). Tauri lets the backend be
  Rust natively, with no Node sidecar.
- **Security model.** Tauri's capability system (`capabilities/default.json`)
  scopes exactly which APIs the frontend can call. CodeCompass grants only
  `core:default`, `dialog:default`, `opener:default` — no filesystem,
  network, or shell scope is exposed to the webview.

**Tradeoffs:**
- Windows-only testing so far. Tauri supports macOS/Linux, but I have not
  verified the WebView differences there.
- The Rust↔JS bridge is more typed but more verbose than Electron's Node
  `ipcMain`/`ipcRenderer`.
- WebView2 runtime must be present on Windows 10+ (it ships with most
  modern installs).

## Why Rust for scanning

**Decision:** `walkdir` + `std::fs::metadata` in Rust.

**Why:**
- **Speed.** Metadata-only traversal of thousands of files is fast in
  Rust with no GC pauses.
- **Safety.** Path handling, symlinks, and root-containment checks are
  trivial to make panic-free with `Result` and no `unwrap` in production
  paths.
- **Testability.** The `ScanCallbacks` trait lets the same scanner run
  under tests with `NoopCallbacks` and in production with
  `TauriCallbacks`, so the core logic is unit-tested without a Tauri
  runtime.

**Tradeoffs:**
- Rust's borrow checker adds friction for batch mutations, but the
  `FileUpsert` / `drain(..)` pattern keeps it manageable.
- No incremental filesystem watcher (e.g. `notify`) yet — rescans are
  full traversals. This is acceptable for the alpha's batch model.

## Why SQLite

**Decision:** `rusqlite` (bundled SQLite) with `refinery` migrations.

**Why:**
- **Local-first.** SQLite is a single file in the app data directory. No
  server, no network, no credentials.
- **Schema discipline.** Eight ordered, versioned migrations (`V1`–`V8`)
  embedded at compile time. Released migrations are never edited.
- **Indexes.** Every foreign key and common filter has an index (see the
  migration files), which keeps graph and symbol queries fast at
  thousand-file scale.
- **WAL mode** allows concurrent reads while a scan writes.

**Tradeoffs:**
- `Connection` is `Send` but not `Sync`, so we wrap it in
  `Mutex<Connection>`. Lock contention is low because writes are batched
  in transactions.
- SQLite's `LIKE '%query%'` for symbol search is not full-text; it is
  adequate for the alpha but a future FTS5 virtual table would scale
  further.

## Why OXC

**Decision:** `oxc_parser` / `oxc_ast` (v0.45) for AST analysis.

**Why:**
- **Speed.** OXC is a Rust-native TypeScript/JavaScript parser — no
  V8/Node required, no JS bridge overhead.
- **Single process.** Parsing happens in the same Rust thread as the
  scanner/DB, so there is no IPC for every file.
- **Coverage.** OXC handles TS/JS syntax including JSX, decorators,
  CommonJS, and dynamic imports, which covers CodeCompass's supported
  languages.

**Tradeoffs:**
- OXC is a fast-moving project; pinning to 0.45 trades the latest fixes
  for reproducibility.
- Semantic type information (the type checker) is out of scope —
  CodeCompass does structural analysis, not type checking. This is a
  deliberate scope limit.

## Why local-first

**Decision:** No cloud, no accounts, no telemetry.

**Why:**
- **Trust.** Developers will not point a tool at their proprietary
  codebase if it uploads anything. Local-first is a feature, not a
  limitation, for the target audience.
- **Simplicity.** No backend to operate, no auth, no data residency
  concerns, no GDPR surface.
- **Offline.** The app works on a machine with no internet connection
  after installation.

**Tradeoffs:**
- No cross-machine sync. Users who want their index on multiple machines
  must re-scan (the index is cheap to rebuild).
- No collaborative features. A future opt-in sync would require explicit
  consent and a server — out of scope for the alpha.

## Why no cloud AI

**Decision:** No LLM/AI features that send source code off-device.

**Why:**
- **Privacy guarantee.** "Your code never leaves your machine" must be
  literally true. Any cloud AI feature would break it.
- **Determinism.** Structural analysis (imports, symbols, graph) is
  deterministic and reproducible. AI explanations are not, which makes
  them harder to test and trust.

**Tradeoffs:**
- No natural-language "explain this file" feature. The Insights engine
  uses structural heuristics (entry-point detection, reading paths,
  cycle detection) instead, which are explainable and fast.
- A future *local* AI feature (e.g. an on-device model) could be added
  without breaking the privacy guarantee, but that is out of scope.

## Why generation-based reconciliation

**Decision:** A monotonic `scan_generation` counter (migration V8) instead
of timestamp comparison for deletion detection.

**Why:**
- **Correctness.** Two scans completing within the same Unix-second
  would be ambiguous under timestamp comparison. A generation counter
  makes "files not seen this scan" unambiguous.
- **Safety.** Reconciliation is skipped on cancelled or error-degraded
  scans, so a partial snapshot never deletes the previous good index.

**Tradeoffs:**
- An extra `app_settings` row per workspace. Negligible cost.
- Slightly more schema complexity, documented in
  [docs/architecture.md](architecture.md).

## Why graph truncation instead of refusing

**Decision:** The dependency graph truncates at 500 nodes with a
`truncated` flag rather than erroring.

**Why:**
- **User experience.** A thousand-file repo should show *something*
  useful (a warning + the first 500 nodes + filters) rather than a
  blank error.
- **Safety.** Bounding the response size prevents the frontend from
  trying to render thousands of React Flow nodes at once.

**Tradeoffs:**
- The truncated view is not the complete graph. Users with very large
  repos must use the path/directory filter to see specific subgraphs.
  Cycle detection runs only on the returned subset.
