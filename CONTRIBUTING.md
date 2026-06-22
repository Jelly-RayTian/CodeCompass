# Contributing to CodeCompass

Thank you for your interest in contributing! This document covers the basics.

## Development Setup

1. Install [Node.js](https://nodejs.org/) 18+ and npm.
2. Install [Rust](https://rustup.rs) stable.
3. On Windows, install Visual Studio 2022 with the "Desktop development with
   C++" workload.
4. Clone the repository and run:

```bash
npm install
npm run tauri:dev
```

## Workflow

1. Create a branch from `main`.
2. Make your changes. Follow the architecture rules in [AGENTS.md](AGENTS.md).
3. Run all checks before submitting:

```bash
# Frontend
npm run lint
npm run typecheck
npm run test
npm run format:check

# Rust (from src-tauri/)
cargo fmt --check
cargo clippy
cargo test
cargo check
```

4. Write tests for new functionality.
5. Keep commits focused. Write clear commit messages.
6. Open a pull request.

## Code Style

- **TypeScript**: strict mode, no `any`, Prettier handles formatting.
- **Rust**: `cargo fmt` handles formatting, `cargo clippy` must pass.
- **Commits**: imperative mood, e.g. "Add workspace list command".

## Project Structure

See [README.md](README.md) for the project layout and
[docs/architecture.md](docs/architecture.md) for design decisions.

## Reporting Issues

Use the GitHub issue tracker. Include:

- Steps to reproduce
- Expected vs actual behavior
- OS and CodeCompass version
