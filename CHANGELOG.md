# Changelog

All notable changes to CodeCompass are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Foundation Milestone

- Tauri v2 + React 18 + TypeScript (strict) + Vite desktop application scaffold
- ESLint (flat config), Prettier, Vitest, React Testing Library configuration
- Application shell with sidebar navigation (Home, Workspaces, Settings pages)
- Loading, empty, and error states for all pages
- Typed Tauri client (`tauriClient`) with shared frontend types
- SQLite database with refinery version-controlled migrations
- Initial database schema: `workspaces`, `indexed_files`, `analysis_runs`,
  `app_settings` tables
- Tauri commands: `get_application_info`, `get_database_status`,
  `list_workspaces`
- Typed Rust error enum (`AppError`) with serde serialization
- Frontend tests: application renders, navigation works
- Rust tests: initial migration creates all tables, `list_workspaces` returns
  empty list for a new database
- Documentation: README, AGENTS, CONTRIBUTING, CHANGELOG, product, architecture,
  database, privacy, roadmap, testing
- GitHub Actions CI workflow (frontend lint, typecheck, test, build; cargo fmt,
  clippy, test, check)
