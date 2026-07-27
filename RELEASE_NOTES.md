# CodeCompass v1.0.2 Release Notes

**Release date:** 2026-07-27
**Full changelog:** [CHANGELOG.md](./CHANGELOG.md)

## Overview

v1.0.2 hardens the stable CodeCompass release for public distribution. It
reduces startup JavaScript, adds enforceable bundle budgets, validates the
installed application on a clean Windows runner, and publishes checksums for
every installer.

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
- **Faster startup bundle:** the Viewer and bundled Monaco runtime now load on
  demand, reducing the initial production entry from about 4.21 MB to 408.6 KB.
- **Release hardening:** NSIS and MSI are both required; the release runner
  installs, launches, and uninstalls the NSIS package before publication.
- **Safe preflight:** maintainers can run the complete release pipeline on a
  branch without creating a public tag or Release.
- **Verifiable downloads:** each release includes `SHA256SUMS.txt`.
- **Exact artifact selection:** the workflow only uploads filenames matching
  the manifest version, preventing stale bundles from entering a release.

## Enforced bundle budgets

The production build fails if the initial JavaScript entry exceeds 500 KiB or
the lazy Viewer chunk exceeds 4,000 KiB. This preserves the startup improvement
and ensures Monaco cannot silently move back into the initial bundle.

## Previously measured performance

The v0.5 release benchmark measured 5,000 generated TypeScript files on a
release build: analysis improved from 24.1 seconds to 2.8 seconds (8.5×), while
scan time improved from 362 ms to 255 ms (1.4×). These are historical benchmark
results, not measurements from every machine. See
[docs/benchmarks.md](./docs/benchmarks.md) for the method and raw table.

## Installers

- **NSIS:** `CodeCompass_1.0.2_x64-setup.exe`
- **MSI:** `CodeCompass_1.0.2_x64_en-US.msi`
- **Checksums:** `SHA256SUMS.txt`

The installers are unsigned, so Windows SmartScreen may show a warning.

## Verification

Release-candidate verification on Windows:

- Prettier, ESLint, strict TypeScript, and version-alignment checks passed.
- 13 frontend tests passed.
- 118 Rust tests passed (99 unit, 10 failure-path, 9 fixture integration).
- Frontend production build, `cargo check`, and Clippy with warnings denied
  passed.
- NSIS and MSI installers were produced from the same v1.0.2 source tree.
- The release workflow validates an NSIS install-launch-uninstall cycle on its
  clean Windows runner before publishing.

## Known limitations

- Windows x64 only; macOS and Linux are not release-tested.
- Installers are unsigned and there is no automatic updater.
- First-party analyzers cover TypeScript, JavaScript, and CSS; other languages
  are not yet supported.
- The dependency graph is capped at 500 nodes and reports truncation.
- Health and impact scores are heuristics for navigation, not proof of defects.
- The Viewer remains a large on-demand chunk because Monaco is bundled locally;
  its size is guarded by the release build budget.
