# CodeCompass

**A local-first desktop application for understanding unfamiliar code repositories.**

[![CI](https://github.com/Jelly-RayTian/CodeCompass/actions/workflows/ci.yml/badge.svg)](https://github.com/Jelly-RayTian/CodeCompass/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Jelly-RayTian/CodeCompass?include_prereleases&label=release)](https://github.com/Jelly-RayTian/CodeCompass/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Status: v0.1.0 Alpha** — active development, feature-complete, undergoing stability testing.

CodeCompass analyzes TypeScript and JavaScript projects to help you navigate, understand, and assess codebases — entirely offline. No cloud uploads, no AI training on your source code.

## Why I Built CodeCompass

When I join a new codebase, I ask the same questions every time: _Where is the entry point? What order should I read the files? Which modules are tightly coupled? What would break if I change this function?_

Existing tools either upload your code to the cloud or require setting up complex language servers. I wanted a **single binary** that I could point at any folder and get instant structural understanding — no configuration, no network, no risk.

## Screenshots

> Real screenshots coming soon. For now, see the [Demo Workflow](#demo-workflow) below.

<!-- TODO: Add screenshots of: Workspaces page with file tree, Dependency Graph, Symbol Search, Code Viewer, Insights -->
<!-- Place at: docs/screenshots/workspaces.png, docs/screenshots/graph.png, etc. -->

## Features

| Category                | Highlights                                                      |
| ----------------------- | --------------------------------------------------------------- |
| **Repository Scanning** | Recursive traversal, ignore rules, incremental change detection |
| **AST Analysis**        | Static/dynamic imports, re-exports, CommonJS `require`          |
| **Symbol Indexing**     | Functions, classes, interfaces, types, enums, React components  |
| **Dependency Graph**    | Interactive React Flow, cycle detection, node details           |
| **Symbol Search**       | Name/kind filtering, pagination, click-to-view                  |
| **Code Viewer**         | Monaco Editor, syntax highlighting, search, line numbers        |
| **Insights**            | Entry-point detection, reading paths, structural findings       |
| **Impact Analysis**     | Call graph, transitive dependents, change risk scoring          |
| **Git Integration**     | Branch/status/commits, co-change hotspots                       |
| **i18n**                | Chinese / English                                               |

## Core Architecture

```
┌─ React Frontend ──────────────────────────┐
│  Workspaces · Graph · Insights · Viewer    │
│  Monaco Editor · React Flow · i18n         │
├───────────────────────────────────────────┤
│         Tauri IPC (typed invoke)           │
├─ Rust Backend ────────────────────────────┤
│  Scanner (walkdir) · Parser (OXC)          │
│  DB (rusqlite + refinery migrations)       │
│  Git commands (safe subprocess)            │
├───────────────────────────────────────────┤
│  SQLite (WAL mode, V1→V8 migrations)       │
└───────────────────────────────────────────┘
```

## Quick Start

### Installation

Download from [Releases](https://github.com/Jelly-RayTian/CodeCompass/releases):

- **NSIS**: `CodeCompass_x.x.x_x64-setup.exe` (required)
- **MSI**: `CodeCompass_x.x.x_x64_en-US.msi` (optional)

> Installers are **unsigned** — Windows SmartScreen may warn. Click "More info" → "Run anyway".

**Uninstall**: Settings → Apps → CodeCompass → Uninstall. Your source files are never modified.

### Development

```bash
git clone https://github.com/Jelly-RayTian/CodeCompass.git
cd CodeCompass
npm install
npm run tauri:dev
```

### Build

```bash
npm run tauri:build
# Output: src-tauri/target/release/bundle/
```

## Demo Workflow

1. **Add Folder** — select your project directory
2. **Scan** — indexes `.ts/.tsx/.js/.jsx` files
3. **Analyze** — extracts imports, symbols, and call references
4. **Explore** — dependency graph, symbol search, source viewer
5. **Understand** — Insights for entry points, reading paths, and risks

## Testing

```bash
npm test                       # 8 frontend tests
cd src-tauri && cargo test      # 83 Rust tests
```

## Roadmap

| Milestone                           | Status         |
| ----------------------------------- | -------------- |
| Foundation (Tauri + React + SQLite) | ✅ Complete    |
| Repository Scanning                 | ✅ Complete    |
| AST Import Analysis                 | ✅ Complete    |
| File Dependency Graph               | ✅ Complete    |
| Symbol Indexing & Search            | ✅ Complete    |
| Code Viewer & Navigation            | ✅ Complete    |
| Entry Points & Insights             | ✅ Complete    |
| Call Graph & Impact Analysis        | ✅ Complete    |
| Git Integration                     | ✅ Complete    |
| Release Engineering & Distribution  | ✅ Complete    |
| Polish & Stable Release             | 🚧 In Progress |

## Known Limitations

- **Windows only** — macOS and Linux not tested
- **Unsigned installers** — SmartScreen may block installation
- **Icons are placeholders** — custom icon set coming before stable release
- **TypeScript/JavaScript only** — no Python, Rust, or other language support yet
- **No auto-update** — users must manually download new versions
- **Large repos (>10k files)** — graph visualization may slow down; analysis is batch-only

## Privacy Guarantees

- ✅ **100% local** — your source code never leaves your machine
- ✅ **No telemetry** — zero analytics, zero usage tracking
- ✅ **No network requests** — the app works fully offline after installation
- ✅ **No secrets logging** — source file contents, API keys, and tokens are never logged
- ✅ **Read-only analysis** — CodeCompass never executes, modifies, or deletes your code

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) for development guidelines.

## License

MIT
