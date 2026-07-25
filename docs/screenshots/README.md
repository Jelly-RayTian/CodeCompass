# Screenshots

Real screenshots must be captured from a running CodeCompass release build,
whether the capture step is manual or automated. **Do not commit fake or
AI-generated product screenshots.**

## Required screenshots

Capture each at a 1440×900 window size (or similar) and save as PNG in
this directory. Crop to the application window, not the full desktop.

| File             | What to capture                                              | Page / route  |
| ---------------- | ------------------------------------------------------------ | ------------- |
| `home.png`       | Home page with version, DB status, quick actions             | `/`           |
| `workspaces.png` | Workspaces page with a scanned project + file tree visible   | `/workspaces` |
| `graph.png`      | Dependency Graph page with nodes, edges, and a cycle warning | `/graph`      |
| `viewer.png`     | Code Viewer showing a TypeScript file with syntax highlight  | `/viewer`     |
| `insights.png`   | Insights page with entry points and reading paths listed     | `/insights`   |
| `demo.gif`       | Short sequence using the same verified release-build views   | multiple      |

## How to capture

1. Build and run: `npm run tauri:dev`
2. Add a real TypeScript project folder (e.g. the CodeCompass repo itself)
3. Scan → Analyze
4. Navigate to each page and capture the application window.
5. Save into this directory with the exact filenames above.
6. The README references these paths; no further edits are needed once
   the files exist.

## Notes

- Redact any sensitive paths if your test repo contains them. The Home page
  masks the Windows user profile in its displayed database path.
- Prefer a dark-themed OS for a consistent look with the app's `vs-dark`
  Monaco theme.
- After capturing, verify the README image links render on GitHub.
