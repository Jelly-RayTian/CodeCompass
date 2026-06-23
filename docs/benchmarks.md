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
**Date:** 2026-06-24.

| Files | Scan (ms) | Analyze (ms) | Graph (ms) | Unchanged rescan (ms) | Modified rescan (ms) | Imports | Symbols |
|------:|----------:|-------------:|-----------:|----------------------:|---------------------:|--------:|--------:|
|   100 |      14.0 |         473.8 |        0.3 |                  16.1 |                 16.1 |      14 |     108 |
| 1,000 |      92.0 |       4,725.2 |        0.7 |                  86.6 |                 82.0 |     142 |   1,077 |
| 5,000 |     361.9 |      24,112.9 |        2.9 |                 411.9 |                402.0 |     714 |   5,385 |

### Reading the table

- **Scan** — full `walkdir` traversal + batched SQLite upserts (metadata
  only; file contents are not read).
- **Analyze** — OXC AST parse + import/symbol/reference extraction +
  persistence for every file.
- **Graph** — `build_graph` over the `imports` table (counting, node
  collection, edge collection, cycle detection).
- **Unchanged rescan** — a second scan with no filesystem changes; the
  upsert path still runs but all rows resolve to `unchanged`.
- **Modified rescan** — every 10th file is rewritten (size + mtime
  change) before rescanning, exercising the `changed` path.
- **Imports / Symbols** — counts produced by analysis, for scale
  context.

### Observations

- **Scan scales near-linearly** with file count and is fast (metadata
  only). Unchanged rescan is slightly slower than first scan due to the
  upsert comparison path.
- **Analysis dominates** total time, as expected: it reads and parses
  every file. ~4.8 ms/file at 1,000 files; ~4.8 ms/file at 5,000 files
  — linear, no super-linear blowup.
- **Graph construction is sub-3 ms** even at 5,000 files, thanks to the
  SQLite indexes on `imports.source_file_id` and
  `imports.resolved_target_file_id`.
- **Graph truncation** (500-node cap) is not exercised by these
  fixtures because only ~14% of files have imports. The truncation path
  is covered by `analysis::graph::tests::large_graph_is_truncated_not_refused`
  and `tests/failure_paths.rs::graph_truncation_caps_nodes_at_limit`.

## What is not measured here

- **Peak memory.** Criterion does not report memory. A future
  `iai-callgrind` or manual `#[track_alloc]` harness could add this.
- **Frontend render time** for very large React Flow graphs. The
  backend caps graph responses at 500 nodes, so this is bounded by
  design rather than measured here.
- **Debug-build numbers.** Debug builds are 5–10× slower for the
  analysis phase; always benchmark with `--release`.

## Investigated bottlenecks

Based on the measurements, the analysis phase is the clear bottleneck
(~96% of total time at 5,000 files). It is dominated by OXC parsing and
per-file `read_to_string`. No speculative rewrite was performed without
evidence. Potential future improvements, if measurements justify them:

- Reuse parsed module ASTs across rescans when the fingerprint is
  unchanged (currently analysis re-parses all pending/changed files).
- Parallelize file parsing with `rayon` (the SQLite writes would still
  be serialized through the `Mutex<Connection>`).
- Add an FTS5 virtual table for symbol search if `LIKE` becomes a
  bottleneck at >50k symbols.
