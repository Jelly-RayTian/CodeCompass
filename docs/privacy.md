# Privacy

## Local-First by Design

CodeCompass runs entirely on your machine. No code, file names, file contents,
or analysis results are sent to any server.

## What We Store

CodeCompass stores a SQLite database in your platform's application data
directory. This database contains:

- **Indexed folder records**: folder name, normalized local filesystem path,
  date added, last successful scan, availability, monitoring flag, and scan
  status.
- **Indexed file metadata**: relative paths, name, parent path, extension,
  size, filesystem creation/modification times, first indexed time, and last
  seen time.
- **Scan run records**: timestamps, status, phase, processed/indexed counts,
  warning/error counts, and error messages.
- **Analysis run records**: timestamps, status, error messages (reserved for
  future analysis milestones).
- **App settings**: key-value preferences.

The database does **not** store source code contents. Only metadata about
files is persisted.

## What We Access

| Resource                        | When                            | Purpose                   |
| ------------------------------- | ------------------------------- | ------------------------- |
| Filesystem (read metadata only) | When you register/scan a folder | Scan file structure       |
| App data directory (read/write) | Always                          | Store the SQLite database |

CodeCompass does **not** access:

- Network resources (no telemetry, no analytics, no update checks in the
  current version)
- System registries beyond what Tauri/WebView2 require
- Other applications' data

## Data Removal

To remove all CodeCompass data:

1. Delete the database file (see [docs/database.md](database.md) for the path).
2. Uninstall the application.

## Future Considerations

- **Auto-updates**: If added in the future, will require a network connection
  to check for new versions. This will be opt-in.
- **AI features**: If added in the future, will require explicit user consent
  for any data that leaves the machine. The local-first principle applies to
  all non-AI features.
