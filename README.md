# CodeCompass

A local-first desktop application for understanding unfamiliar codebases.

CodeCompass analyzes repository structure locally, visualizes file and symbol
relationships, identifies likely entry points, and helps you understand what may
be affected by code changes — all without sending your code to a server.

## Status

**Foundation milestone** — project scaffold, database, and minimal UI only.
Repository scanning, graph visualization, and analysis features are not yet
implemented.

## Tech Stack

| Layer      | Technology                                                        |
| ---------- | ----------------------------------------------------------------- |
| Desktop    | Tauri v2                                                          |
| Frontend   | React 18 + TypeScript (strict) + Vite                             |
| Backend    | Rust                                                              |
| Database   | SQLite (rusqlite, bundled)                                        |
| Migrations | refinery                                                          |
| Testing    | Vitest + React Testing Library (frontend), `cargo test` (Rust)    |
| Linting    | ESLint + Prettier (frontend), `cargo fmt` + `cargo clippy` (Rust) |

## Prerequisites

- **Node.js** 18+ and npm
- **Rust** stable (install via [rustup](https://rustup.rs))
- **Visual Studio 2022** with the "Desktop development with C++" workload (Windows)
- **WebView2 Runtime** (pre-installed on Windows 10/11)

## Getting Started

```bash
# Install frontend dependencies
npm install

# Run in development mode (starts Vite + Tauri)
npm run tauri:dev

# Build a production installer
npm run tauri:build
```

## NPM Scripts

| Script         | Description                       |
| -------------- | --------------------------------- |
| `dev`          | Start Vite dev server only        |
| `build`        | Type-check and build the frontend |
| `lint`         | Run ESLint                        |
| `lint:fix`     | Run ESLint and fix issues         |
| `format`       | Format with Prettier              |
| `format:check` | Check formatting without writing  |
| `typecheck`    | Run `tsc` with no emit            |
| `test`         | Run Vitest tests once             |
| `tauri:dev`    | Start Tauri in development mode   |
| `tauri:build`  | Build a production desktop app    |

## Rust Commands

| Script              | Description           |
| ------------------- | --------------------- |
| `cargo fmt --check` | Check Rust formatting |
| `cargo clippy`      | Run Rust linter       |
| `cargo test`        | Run Rust tests        |
| `cargo check`       | Fast type-check       |

Run these from the `src-tauri/` directory.

## Project Structure

```
CodeCompass/
├── src/                    # React frontend
│   ├── app/                # Application shell and navigation
│   ├── pages/              # Home, Workspaces, Settings
│   ├── components/         # Reusable UI components
│   ├── lib/                # tauriClient, useAsyncData hook
│   ├── types/              # Shared TypeScript types
│   ├── styles/             # Global CSS
│   └── test/               # Test setup and test files
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── commands/       # Thin Tauri command wrappers
│   │   ├── db/             # Database connection and migrations
│   │   ├── migrations/     # SQL migration files
│   │   ├── models/         # Data models (serde structs)
│   │   ├── platform/       # OS-specific helpers
│   │   ├── scanner/        # Repository scanner (stub)
│   │   ├── analysis/       # Code analysis (stub)
│   │   ├── tasks/          # Background tasks (stub)
│   │   ├── error.rs        # Typed error enum
│   │   ├── lib.rs          # Module wiring and Tauri setup
│   │   └── main.rs         # Entry point
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── docs/                   # Project documentation
├── .github/workflows/      # CI configuration
└── package.json
```

## Documentation

- [Product Overview](docs/product.md)
- [Architecture](docs/architecture.md)
- [Database Schema](docs/database.md)
- [Privacy](docs/privacy.md)
- [Roadmap](docs/roadmap.md)
- [Testing](docs/testing.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

MIT
