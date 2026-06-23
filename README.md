# CodeCompass

**A local-first desktop application for understanding unfamiliar code repositories.**

CodeCompass analyzes TypeScript and JavaScript projects to help you navigate, understand, and assess codebases — entirely offline. No cloud uploads, no AI training on your source code.

## Features

| Category                | Highlights                                                                 |
| ----------------------- | -------------------------------------------------------------------------- |
| **Repository Scanning** | Recursive filesystem traversal, ignore rules, incremental change detection |
| **AST Analysis**        | Import extraction, static/dynamic imports, re-exports, CommonJS `require`  |
| **Symbol Indexing**     | Functions, classes, interfaces, types, enums, React components             |
| **Dependency Graph**    | Interactive React Flow visualization, cycle detection, node details        |
| **Symbol Search**       | Name/kind filtering, pagination, clickable results                         |
| **Code Viewer**         | Monaco Editor with syntax highlighting, search, line numbers               |
| **Insights**            | Entry-point detection, beginner reading paths, structural findings         |
| **Impact Analysis**     | Call graph, transitive dependents, change risk scoring                     |
| **Git Integration**     | Branch/status/commits, co-change hotspots                                  |
| **i18n**                | Chinese / English language switching                                       |

## Quick Start

### Installation

Download the latest installer from [Releases](https://github.com/Jelly-RayTian/CodeCompass/releases) and run it.

- **NSIS installer**: `CodeCompass_x.x.x_x64-setup.exe`
- **MSI installer**: `CodeCompass_x.x.x_x64_en-US.msi`

> **Note**: Installers are unsigned — Windows may show a SmartScreen warning. Click "More info" → "Run anyway".

**Uninstall**: Use _Settings → Apps → Installed apps_ → CodeCompass → Uninstall.

### Prerequisites (Development)

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.77
- [Git](https://git-scm.com/) (optional, for Git features)
- Windows: [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#windows)

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
```

Outputs are at `src-tauri/target/release/bundle/`.

## Demo Workflow

1. **Add a folder** — click "Add folder", select your project directory
2. **Scan** — CodeCompass indexes all `.ts/.tsx/.js/.jsx` files
3. **Analyze** — extract imports, symbols, and call references
4. **Explore** — browse the dependency graph, search symbols, view source
5. **Understand** — check Insights for entry points, reading paths, and risks

## Architecture

```
React Frontend (TypeScript, Monaco, React Flow)
    ↕ Tauri IPC
Rust Backend (OXC parser, SQLite, walkdir, git)
    ↕ SQLite
Local database + source files (never uploaded)
```

## Testing

```bash
npm test                  # Frontend tests
cd src-tauri && cargo test  # Rust tests (64 tests)
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) for development guidelines.

## License

MIT
