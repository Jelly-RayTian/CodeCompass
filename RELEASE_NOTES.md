# CodeCompass v1.0.1 Release Notes

**Release date:** 2026-07-27
**Full changelog:** [CHANGELOG.md](./CHANGELOG.md)

## Overview

v1.0.1 is the first stable CodeCompass release. It packages the complete
local-first workflow developed across the v0.x releases: register a repository,
scan supported files, analyze imports and symbols, explore dependencies and
source, and review repository insights without uploading source code.

## Highlights

- **Local-first analysis:** source stays on the machine; the SQLite index stores
  metadata and analysis results, not source contents.
- **Repository understanding:** dependency graph, symbol search, Monaco source
  viewer, entry points, reading paths, impact analysis, repository health, and
  Git evolution views.
- **Incremental and resilient scanning:** batched writes, change detection,
  cancellation, generation-based deletion reconciliation, and recovery of
  interrupted runs.
- **Extensible analyzers:** the registry-based plugin architecture currently
  handles TypeScript, JavaScript, and CSS.
- **Release polish:** aligned v1.0.1 versions, stable-release CI configuration,
  updated public documentation, real screenshots, and a demo GIF.

## Correctness fix in this release

The final partial scanner batch now persists its counters before a run is
marked finished or cancelled. Previously, a repository with fewer than 500
supported files could be indexed correctly while its latest scan summary still
displayed zero files. A Rust regression test covers the fixed path.

## Previously measured performance

The v0.5 release benchmark measured 5,000 generated TypeScript files on a
release build: analysis improved from 24.1 seconds to 2.8 seconds (8.5×), while
scan time improved from 362 ms to 255 ms (1.4×). These are historical benchmark
results, not measurements from every machine. See
[docs/benchmarks.md](./docs/benchmarks.md) for the method and raw table.

## Installers

- **NSIS:** `CodeCompass_1.0.1_x64-setup.exe`
- **MSI:** `CodeCompass_1.0.1_x64_en-US.msi`

The installers are unsigned, so Windows SmartScreen may show a warning.

## Verification

Release-candidate verification on Windows:

- Prettier, ESLint, strict TypeScript, and version-alignment checks passed.
- 12 frontend tests passed.
- 118 Rust tests passed (99 unit, 10 failure-path, 9 fixture integration).
- Frontend production build, `cargo check`, and Clippy with warnings denied
  passed.
- NSIS and MSI installers were produced from the same v1.0.1 source tree.

## Known limitations

- Windows x64 only; macOS and Linux are not release-tested.
- Installers are unsigned and there is no automatic updater.
- First-party analyzers cover TypeScript, JavaScript, and CSS; other languages
  are not yet supported.
- The dependency graph is capped at 500 nodes and reports truncation.
- Health and impact scores are heuristics for navigation, not proof of defects.
- The production frontend build reports a non-blocking large-chunk warning,
  primarily from the bundled Monaco editor.
