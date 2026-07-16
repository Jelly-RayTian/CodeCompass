# CodeCompass v0.3.0 Release Notes

**Release date:** 2026-07-16  
**Full changelog:** [CHANGELOG.md](./CHANGELOG.md)

## Overview

v0.3.0 introduces the **Git Evolution** dashboard — visualize how your repository
has changed over time through commit history, file churn, and co-change hotspots.
All analysis is local-first; no repository data is ever uploaded.

## Highlights

### Git Evolution Dashboard

- **Commit Timeline** chart showing commits and file changes per month as a bar
  chart, giving an at-a-glance view of development activity over time.
- **File Churn** ranking showing the 20 most frequently changed files, with
  visual proportional bars.
- **Co-Change Hotspots** listing file pairs that changed together most often.
- **Summary cards** with total commits, unique files changed, total file changes,
  most active month, and date range of tracked history.

### Git data improvements

- `git_file_changes.timestamp` now stores real Unix timestamps from git log
  (previously always 0), enabling time-based aggregation.
- Commit depth increased from 50 to 200 commits with a 1000-file cap.
- New `git::commit_log()` infrastructure for future commit-level features.

### Data sources

| Feature          | Source                     |
|------------------|----------------------------|
| Commit timeline  | `git_file_changes` (month buckets) |
| File churn       | `git_file_changes` (group by path) |
| Co-change pairs  | `git_file_changes` (self-join)     |
| Summary stats    | `git_file_changes` (aggregates)    |

## Known limitations

- Evolution data reflects the last 200 commits only. Older history is not
  imported automatically.
- Timestamps are from the git commit author date (`%ct`), not the commit date.
- Commit message content is not stored — only counts and file paths.
- Git analysis must be enabled in workspace settings for data to be populated.

## Installers

- **NSIS:** `CodeCompass_0.3.0_x64-setup.exe`
- **MSI:** `CodeCompass_0.3.0_x64_en-US.msi`

> Installers are **unsigned** — Windows SmartScreen may warn. Click "More info" → "Run anyway".

## Verification

All checks passed before building:

- `npm run lint`
- `npm run typecheck`
- `npm run test` (10 frontend tests)
- `npm run build` (frontend production build)
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`
- `cd src-tauri && cargo test` (105 Rust tests)
- `cd src-tauri && cargo check`
- `npm run check:versions`
