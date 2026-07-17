# CodeCompass v0.5.0 Release Notes

**Release date:** 2026-07-17  
**Full changelog:** [CHANGELOG.md](./CHANGELOG.md)

## Overview

v0.5.0 is a **performance release** focused on large-repository throughput.
All optimizations are data-driven, based on benchmarks in
[docs/benchmarks.md](./docs/benchmarks.md).

## Highlights

### Measured speed improvements (release build, 5,000 files)

| Phase | Before (v0.4.0) | After (v0.5.0) | Speedup |
|-------|----------------:|---------------:|--------:|
| Analyze | 24.1 seconds | 2.8 seconds | **8.5×** |
| Scan | 362 ms | 255 ms | 1.4× |

At 1,000 files: Analyze is **16.7× faster** (4.7s → 0.28s).

### What changed

1. **Incremental analysis.** Removed workspace-level clearing of imports,
   symbols, and references at the start of each analysis run. Per-file
   replace functions already handle their own clearing. Files that are
   unchanged after a rescan now skip analysis entirely — only new and
   changed files are re-parsed.

2. **Larger scanner batches.** Batch flush size increased from 100 to 500
   files, reducing SQLite transaction overhead by 5×. Progress events
   emit every 50 files instead of 10.

3. **SQLite pragma tuning.** `PRAGMA synchronous=NORMAL` + 8 MB page cache
   (`cache_size=-8000`) — safe with WAL journal mode already enabled.

### Benchmark methodology

All optimizations were verified by the existing reproducible benchmark
harness (`cargo run --release --example bench_summary`). The benchmark
generates synthetic TypeScript projects at runtime (100, 1,000, 5,000 files)
and measures scan, analysis, graph construction, and rescan performance.

## Installers

- **NSIS:** `CodeCompass_0.5.0_x64-setup.exe`
- **MSI:** `CodeCompass_0.5.0_x64_en-US.msi`

## Verification

All checks passed before building:

- `npm run lint`
- `npm run typecheck`
- `npm run test` (10 frontend tests)
- `npm run build` (frontend production build)
- `cd src-tauri && cargo fmt --check`
- `cd src-tauri && cargo clippy --all-targets -- -D warnings`
- `cd src-tauri && cargo test` (117 Rust tests)
- `cd src-tauri && cargo check`
- `npm run check:versions`
- `cd src-tauri && cargo run --release --example bench_summary`
