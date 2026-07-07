# Test Matrix

Last updated: 2026-07-07 | Total: 108 tests

## Rust Tests (98)

### Unit Tests (78)

| Module                 | Tests | Coverage                                                                                                                                                                                                                                                                                                                                               |
| ---------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `platform`             | 3     | Path normalization, case sensitivity, descendant detection                                                                                                                                                                                                                                                                                             |
| `db::connection`       | 3     | Database open, migration runner, path retrieval                                                                                                                                                                                                                                                                                                        |
| `db::mod`              | 1     | All V1–V8 tables and columns created                                                                                                                                                                                                                                                                                                                   |
| `db::indexed_folders`  | 7     | Insert, list, duplicate detection, nested parent, remove, persist, lifecycle                                                                                                                                                                                                                                                                           |
| `db::indexed_files`    | 4     | Upsert change detection (new/changed/unchanged), generation-based removal                                                                                                                                                                                                                                                                              |
| `db::scan_runs`        | 2     | Create/start/finish round-trip, interrupted run marking                                                                                                                                                                                                                                                                                                |
| `db::imports`          | 1     | Replace and list round-trip                                                                                                                                                                                                                                                                                                                            |
| `commands::workspaces` | 1     | Empty fetch for new database                                                                                                                                                                                                                                                                                                                           |
| `tasks`                | 2     | Register and cancel, cancel unknown run                                                                                                                                                                                                                                                                                                                |
| `scanner`              | 18    | Empty folder, nested files, ignored dirs, unsupported files, symlinks, cancellation, preserve previous, incremental detection, warnings reconciliation, cancelled never reconciles, successful removal, same-second gens, failed preserves snapshot, partial upserts, unchanged rescan, new file detection, completed_with_errors skips reconciliation |
| `analysis::ts_js`      | 10    | Static default, named, re-export, re-export all, require, dynamic import, relative resolution, malformed, empty, multiple                                                                                                                                                                                                                              |
| `analysis::resolver`   | 6     | External package, relative with ext, without ext, index fallback, unresolved, path traversal                                                                                                                                                                                                                                                           |
| `analysis::graph`      | 5     | Empty graph, two-node, cycle detection, isolated node, large-graph truncation                                                                                                                                                                                                                                                                          |
| `analysis::symbols`    | 11    | Function, class+method, interface, type alias, enum, arrow, react component, source location, malformed, exported const, class declaration                                                                                                                                                                                                             |
| `analysis::references` | 6     | Simple call, method call, new expression, call inside function, multiple calls, no calls                                                                                                                                                                                                                                                               |
| `error`                | 3     | Stable/unique error codes, user message explains recovery, payload round-trip                                                                                                                                                                                                                                                                          |

### Integration Tests — fixture project (9)

| Test                                             | Coverage                                      |
| ------------------------------------------------ | --------------------------------------------- |
| `fixture_scan_indexes_all_files`                 | Full scan over 10-file TS project             |
| `fixture_static_imports_resolved`                | Static import resolution + external detection |
| `fixture_commonjs_require_detected`              | CommonJS require() extraction                 |
| `fixture_dynamic_import_detected`                | Dynamic import() does not crash               |
| `fixture_malformed_file_does_not_block_analysis` | Error recovery during analysis                |
| `fixture_symbols_extracted`                      | Symbol extraction across all kinds            |
| `fixture_circular_dependency_detected_in_graph`  | Cycle detection via graph builder             |
| `fixture_restart_persistence`                    | DB close/reopen preserves all data            |
| `fixture_workspace_lifecycle`                    | Register → persist → verify files untouched   |

### Failure-Path Tests (10)

| Test                                                 | Coverage                                            |
| ---------------------------------------------------- | --------------------------------------------------- |
| `git_functions_return_none_for_non_git_directory`    | Missing/invalid git repo → no panic                 |
| `git_failure_does_not_panic_on_invalid_path`         | Git against nonexistent path → false/None           |
| `large_file_truncation_marks_truncated_and_caps_size`| >1 MB file viewer truncation + 1 MB cap             |
| `analysis_cancellation_stops_early_without_panic`    | Cancel token stops the analysis loop                |
| `concurrent_scan_rejected_with_scan_already_running` | Second scan while one is running is rejected        |
| `deleted_workspace_directory_reports_missing`        | Deleted folder → availability = missing             |
| `malformed_utf8_file_does_not_crash_scan_or_analysis`| Invalid UTF-8 bytes → lossy read, no panic          |
| `interrupted_runs_marked_on_database_reopen`         | Crash-recovery marks running scans interrupted      |
| `graph_truncation_caps_nodes_at_limit`               | >500-node graph truncates with `truncated` flag     |

## Frontend Tests (10)

| Test                           | Coverage                            |
| ------------------------------ | ----------------------------------- |
| Application shell + brand text | App renders correctly               |
| Home page version              | Application info loads              |
| Database status display        | DB connected + path visible         |
| Workspaces empty state         | Navigation + empty UI               |
| Settings page                  | Navigation + database status label  |
| Insights page folder selector  | Navigation + dropdown visible       |
| Error state on failure         | Workspace list error → error banner |
| Retry button                   | Error state includes retry action   |

## Benchmarks

Reproducible Criterion + single-shot benchmarks generate 100 / 1,000 /
5,000-file fixtures at runtime and measure scan, analyze, graph,
unchanged rescan, and modified rescan. See [docs/benchmarks.md](benchmarks.md).

## Untested Behavior

| Area                                 | Gap                              | Priority |
| ------------------------------------ | -------------------------------- | -------- |
| Monaco Editor rendering              | No DOM-level test for CodeViewer | Low      |
| React Flow graph interaction         | No node click/drag simulation    | Low      |
| Permission-denied file (Windows ACL) | Hard to set deterministically    | Medium   |
| Peak memory                          | No allocation-tracking harness   | Low      |
