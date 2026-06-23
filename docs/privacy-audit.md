# Privacy Audit

**Audit date:** 2026-06-24
**Scope:** CodeCompass runtime (frontend + Rust backend + Tauri config + dependencies)
**Auditor method:** static source search + dependency manifest review

## Summary

CodeCompass is **local-first**. After installation the application makes **no
network requests** during normal operation. Source code, file names, and
analysis results never leave the user's machine.

One issue was found and **fixed** during this audit: the Monaco Editor was
configured by its default loader to fetch its runtime from a CDN. This has
been corrected — Monaco is now bundled locally (see
[Correction: Monaco CDN loader](#correction-monaco-cdn-loader) below).

## What the application accesses

| Resource                        | When                            | Purpose                  | Network? |
| ------------------------------- | ------------------------------- | ------------------------ | -------- |
| Filesystem (read metadata only) | Register/scan a folder          | Build file index         | No       |
| Filesystem (read source text)   | Analyze/view a file             | AST parse, code viewer   | No       |
| App data directory (read/write) | Always                          | Store the SQLite index   | No       |
| `git` child process             | Git panel, on a Git repo        | Branch/status/commits    | No (local subprocess) |

## What the application does **not** do

- No telemetry, analytics, or usage tracking.
- No remote font loading (`src/styles/global.css` has no `@font-face` or
  remote `@import`; the UI uses system fonts).
- No CDN loading (Monaco is bundled — see correction below).
- No runtime update checks (no `tauri-plugin-updater`).
- No remote image loading (no `<img src="http…">`, no remote CSS
  backgrounds).
- No source upload (file contents are read locally and shown in the
  viewer; nothing is transmitted).
- No hidden network requests in the Rust backend (no `reqwest`, `ureq`,
  `hyper`, or `tokio::net` dependencies; `git` is invoked as a local
  subprocess only).

## Evidence

### Frontend (`src/`)

Searches for `fetch(`, `XMLHttpRequest`, `axios`, `WebSocket`,
`EventSource`, `navigator.sendBeacon`, `gtag`, `analytics`, `telemetry`,
`http://`, `https://` in `src/**/*.{ts,tsx,html,css}` returned **no
application-level network calls**.

- `src/lib/tauriClient.ts` — all data access goes through Tauri `invoke`
  (IPC), never `fetch`.
- `src/styles/global.css` — no `@font-face`; no remote `@import`.
- `index.html` — no external script, font, or stylesheet tags.

### Monaco Editor

`@monaco-editor/react` depends on `@monaco-editor/loader`, whose default
configuration loads the Monaco runtime from
`https://cdn.jsdelivr.net/npm/monaco-editor@…/min/vs`. This would have
been a network request on first use of the Code Viewer.

**Correction applied:** `src/lib/monacoConfig.ts` now calls
`loader.config({ monaco })` with the locally-imported `monaco-editor`
package and registers Vite-bundled web workers via
`self.MonacoEnvironment`. The `CodeViewer` imports this module for its
side effect before mounting `<Editor>`. `monaco-editor` is declared as an
explicit dependency in `package.json`. A production build confirms the
Monaco runtime is emitted into `dist/assets/` rather than fetched.

### Rust backend (`src-tauri/src/`)

No HTTP client crate is used. The only external process invocation is
`git` via `std::process::Command` in `src-tauri/src/git/mod.rs`, which
runs locally and never transmits data.

### Tauri configuration

- `src-tauri/tauri.conf.json` — no updater, no HTTP plugin, no remote
  resources.
- `src-tauri/capabilities/default.json` — permissions are
  `core:default`, `dialog:default`, `opener:default`. No network scope.

### Dependencies

`src-tauri/Cargo.toml` contains no `reqwest`, `ureq`, `hyper`, `isahc`,
`attohttpc`, `surf`, or other HTTP client crate. `tauri-plugin-log`,
`tauri-plugin-dialog`, and `tauri-plugin-opener` are local-only plugins.

`package.json` dependencies: React, React Router, Tauri API, React Flow,
Monaco. None of these perform network requests at runtime once Monaco is
bundled locally.

## Data stored locally

The SQLite database in the app data directory contains only **metadata**
(folder paths, file names, sizes, mtimes, import relationships, symbol
names, scan-run records, app settings). **Source code contents are never
persisted** — they are re-read from disk on demand for the viewer.

See [docs/privacy.md](privacy.md) for the user-facing privacy statement
and [docs/database.md](database.md) for the database location.

## How to re-audit

1. `grep -rn "fetch\|XMLHttpRequest\|WebSocket\|EventSource\|sendBeacon\|http://\|https://" src/ index.html`
2. `grep -rn "reqwest\|ureq\|hyper\|tokio::net\|TcpStream" src-tauri/src/`
3. Confirm `src/lib/monacoConfig.ts` is imported before any `<Editor>`.
4. Confirm `package.json` lists `monaco-editor` as a dependency.
5. `npm run build` and confirm Monaco assets appear in `dist/assets/`
   rather than being fetched.

## Limitations of this audit

- This is a **static** audit based on source review. A dynamic
  network-capture test (e.g. running the app behind a local proxy with
  no upstream) is recommended as a final smoke-test step before release;
  see [docs/smoke-test-checklist.md](smoke-test-checklist.md)
  "Offline Operation".
- Tauri and WebView2 themselves may make OS-level network calls outside
  CodeCompass's control (e.g. WebView2 runtime telemetry). These are
  governed by the OS / WebView2, not by CodeCompass, and are out of scope
  for this application-level audit.
