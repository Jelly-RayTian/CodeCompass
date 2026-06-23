# Test Matrix

Last updated: 2026-06-23 | Total: 91 tests

## Rust Tests (83)

### Unit Tests (74)

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
| `analysis::graph`      | 4     | Empty graph, two-node, cycle detection, isolated node                                                                                                                                                                                                                                                                                                  |
| `analysis::symbols`    | 11    | Function, class+method, interface, type alias, enum, arrow, react component, source location, malformed, exported const, class declaration                                                                                                                                                                                                             |
| `analysis::references` | 6     | Simple call, method call, new expression, call inside function, multiple calls, no calls                                                                                                                                                                                                                                                               |

### Integration Tests (9)

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

## Frontend Tests (8)

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

## Untested Behavior

| Area                                 | Gap                              | Priority |
| ------------------------------------ | -------------------------------- | -------- |
| Monaco Editor rendering              | No DOM-level test for CodeViewer | Low      |
| React Flow graph interaction         | No node click/drag simulation    | Low      |
| Git command failure                  | No test for missing `git` binary | Medium   |
| Large file (>1 MB) truncation        | No performance/memory test       | Medium   |
| Concurrent scan + analysis           | No multi-thread test             | Low      |
| Cancellation of in-progress analysis | Not covered by integration test  | Medium   |
