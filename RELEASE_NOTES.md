# CodeCompass v0.2.0 Release Notes

**Release date:** 2026-07-16  
**Full changelog:** [CHANGELOG.md](./CHANGELOG.md)

## Overview

v0.2.0 introduces the **Repository Health Dashboard** — a new page that aggregates
information from CodeCompass's existing static analysis, dependency graph, symbol
index, and git history into a unified codebase health report. No AI features are used;
all metrics are computed from local data only.

## Highlights

### Repository Health Dashboard

- **Summary cards** show total files, analyzed files, internal imports, symbol
  count, detected cycles, and average risk score at a glance.
- **Risk distribution** badges break down files into low / medium / high / critical
  risk categories.
- **Per-file risk scoring** (0–100) using five weighted signals:
  - File size (bytes)
  - Line count (proxy for complexity)
  - Import degree (coupling — both in and out)
  - Git change churn (when available)
  - Parse diagnostics count
- Files in **circular dependencies** receive a 15% risk boost with an explicit
  `is_in_cycle` flag.
- **Risk table** listing files by descending risk with columns for score (with
  color-coded bar), lines, imports, symbols, churn count, and cycle indicator.
- Toggle between **top 20 risk files** and **full file list**.
- **Limitations warning** on every page: risk scores are investigative heuristics,
  not quality or correctness judgments.

### Line counting

- Analysis runner now counts source lines after reading each file and stores the
  count in `indexed_files.line_count` (V9 migration).
- Previously analyzed files show `line_count = 0` until re-analyzed.

### Internationalization

- Health dashboard is fully translated into English and Chinese.

## Data sources

The health report reuses existing data without new scanning or parsing:

| Signal            | Source table(s)        |
|-------------------|------------------------|
| File size         | `indexed_files`        |
| Line count        | `indexed_files` (V9)   |
| Import degree     | `imports`              |
| Symbol count      | `symbols`              |
| Diagnostics       | `analysis_diagnostics` |
| Change churn      | `git_file_changes`     |
| Cycle membership  | DFS over `imports`     |

## Known limitations

- Risk scores approximate maintainability; they do not measure actual code
  quality, runtime behavior, or correctness.
- Cyclomatic complexity is approximated via line count — not AST-level branch counting.
- Change churn is only available when Git analysis is enabled per workspace.
- Line counts are only available for files analyzed after this release.

## Installers

- **NSIS:** `CodeCompass_0.2.0_x64-setup.exe`
- **MSI:** `CodeCompass_0.2.0_x64_en-US.msi`

> Installers are **unsigned** — Windows SmartScreen may warn. Click "More info" → "Run anyway".

## Verification

All checks passed before building:

- `npm run lint`
- `npm run typecheck`
- `npm run test` (10 frontend tests)
- `npm run build` (frontend production build)
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`
- `cd src-tauri && cargo test` (102 Rust tests)
- `cd src-tauri && cargo check`
- `npm run check:versions`
