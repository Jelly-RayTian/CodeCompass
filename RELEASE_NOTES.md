# CodeCompass v0.1.1 Release Notes

**Release date:** 2026-07-07  
**Full changelog:** [CHANGELOG.md](./CHANGELOG.md)

## Overview

v0.1.1 is a stability and polish release for CodeCompass. It fixes small bugs and UX inconsistencies discovered during the v0.1.0 alpha, improves perceived responsiveness during long scans, and makes error messages more actionable. No major features were added.

## Highlights

### Bug fixes

- Fixed a variable-name typo in the scanner's deletion-reconciliation path (`reconciliation_failed`).
- Removed stale "Chronicle" product-name references from deletion comments.
- Split `LanguageContext.tsx` so it only exports the `LanguageProvider` component, eliminating the React Fast Refresh ESLint warning.
- Added React Router v7 future flags to remove upgrade warnings.
- Added an explicit `type="button"` to the `ErrorState` retry button.

### Better error messages

- New `AppError::FileNotFound` variant with stable code `file_not_found`.
- `read_source_file` now returns a clear "file not found" message (instead of a generic invalid-input error) when a source file was moved or deleted after the last scan.

### Faster, more responsive scans

- The scanner now emits `scan:progress` events every 10 files in addition to the existing 100-file batch flush, so the Workspaces UI no longer looks stuck on large repositories.
- Analysis progress events now emit every 10 files instead of every 50.

### Improved i18n consistency

- `Graph.tsx` and `Insights.tsx` no longer contain hardcoded English strings; they use the shared translation bundle.
- Added English and Chinese translations for graph truncation warnings and Insights labels.

### More tests

- Rust tests: 96 → 98 (added missing-source-file failure-path test).
- Frontend tests: 8 → 10 (added `i18n.test.tsx`).
- Total: 104 → 108.

## Installers

- **NSIS:** `CodeCompass_0.1.1_x64-setup.exe`
- **MSI:** `CodeCompass_0.1.1_x64_en-US.msi`

> Installers are **unsigned** — Windows SmartScreen may warn. Click "More info" → "Run anyway".

## Known limitations

- Windows only; macOS and Linux are not yet tested.
- No auto-update; users must manually download new versions.
- Large repos (>10k files) still truncate the dependency graph to 500 nodes with a warning.

## Verification

All checks passed before building:

- `npm run lint`
- `npm run typecheck`
- `npm run test` (10 frontend tests)
- `npm run build` (frontend production build)
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`
- `cd src-tauri && cargo test` (98 Rust tests)
- `cd src-tauri && cargo check`
- `npm run check:versions`
- `npm run tauri:build` (produced NSIS + MSI installers)
