# Benchmarks

Reproducible performance measurements for CodeCompass's scan, analysis,
and graph-construction pipeline.

## Methodology

- **Fixtures are generated at runtime** in a temporary directory. No
  fixture data is committed to the repository.
- Each fixture is a synthetic TypeScript project with `n` `.ts` files in
  `src/`. Every 7th file imports its predecessor; every 13th file adds a
  class — so the analyzer and graph builder exercise realistic paths
  (imports, symbols, references, cycles).
- **Determinism:** file names and contents are derived solely from
  indices, so two runs at the same size produce identical work.
- Measurements are single-shot for the summary table and statistical
  (Criterion) for `cargo bench`.
- No internet access is required at any point.

## How to run

```bash
# Single-shot markdown table (fast, prints to stdout):
npm run bench:summary
# equivalent to: cd src-tauri && cargo run --release --example bench_summary

# Criterion statistical reports (slower, writes to target/criterion/):
cd src-tauri && cargo bench
```

## Latest measured results

**Environment:** Windows, x86_64, release build (`--release`).
**Date:** 2026-07-17. **Release:** v0.5.0.

| Files | Scan (ms) | Analyze (ms) | Graph (ms) | Unchanged rescan (ms) | Modified rescan (ms) | Imports | Symbols |
|------:|----------:|-------------:|-----------:|----------------------:|---------------------:|--------:|--------:|
|   100 |       3.4 |         28.3 |        0.2 |                   5.0 |                  4.4 |      14 |     108 |
| 1,000 |      25.1 |        282.6 |        0.6 |                  25.1 |                 25.4 |     142 |   1,077 |
| 5,000 |     255.3 |      2,837.0 |        4.4 |                 203.4 |                233.0 |     714 |   5,385 |

### Comparison with v0.4.0

| Files | Phase | v0.4.0 (ms) | v0.5.0 (ms) | Speedup |
|------:|-------|-----------:|-----------:|--------:|
| 5,000 | Analyze | 24,112.9 | 2,837.0 | **8.5×** |
| 5,000 | Scan | 361.9 | 255.3 | 1.4× |
| 1,000 | Analyze | 4,725.2 | 282.6 | **16.7×** |
| 1,000 | Scan | 92.0 | 25.1 | 3.7× |

## v0.5.0 performance optimizations

### 1. Incremental analysis (smart re-analysis)

**Before:** Every `run_analysis()` call cleared all import/symbol/reference data
workspace-wide and reset `analysis_status = 'pending'` for every file, forcing
re-analysis of unchanged files.

**After:** Per-file `replace_file_*` functions already do DELETE-then-INSERT
within their own file scope. The workspace-level clearing and blanket
`mark_pending_analysis` were removed. Files with `analysis_status = 'analyzed'`
and `change_status = 'unchanged'` are automatically skipped on re-analysis.

### 2. Scanner batch size increase

Batch flush size increased from 100 to 500 files, reducing SQLite transaction
overhead by 5×. Progress events now emit every 50 files instead of 10.

### 3. SQLite pragma tuning

Added `PRAGMA synchronous=NORMAL` (safe with WAL journal mode) and
`PRAGMA cache_size=-8000` (8 MB page cache) to improve read/write throughput.

## Previous results (v0.4.0, 2026-06-24)

| Files | Scan (ms) | Analyze (ms) | Graph (ms) | Unchanged rescan (ms) | Modified rescan (ms) | Imports | Symbols |
|------:|----------:|-------------:|-----------:|----------------------:|---------------------:|--------:|--------:|
|   100 |      14.0 |         473.8 |        0.3 |                  16.1 |                 16.1 |      14 |     108 |
| 1,000 |      92.0 |       4,725.2 |        0.7 |                  86.6 |                 82.0 |     142 |   1,077 |
| 5,000 |     361.9 |      24,112.9 |        2.9 |                 411.9 |                402.0 |     714 |   5,385 |

## What is not measured here

- **Peak memory.** Criterion does not report memory.
- **Frontend render time** for very large React Flow graphs. The
  backend caps graph responses at 500 nodes.
- **Re-analysis on unchanged repos.** Since v0.5.0, unchanged files are
  skipped entirely, so a re-analysis of an unchanged 5,000-file repo
  should show near-zero analysis time (only the query to select
  changed files runs).
- **Debug-build numbers.** Debug builds are 5–10× slower for the
  analysis phase; always benchmark with `--release`.
