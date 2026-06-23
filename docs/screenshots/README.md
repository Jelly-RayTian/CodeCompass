# Screenshots

Real screenshots must be captured manually from a running build of
CodeCompass. **Do not commit fake or AI-generated screenshots.**

## Required screenshots

Capture each at a 1440×900 window size (or similar) and save as PNG in
this directory. Crop to the application window, not the full desktop.

| File              | What to capture                                             | Page / route      |
| ----------------- | ----------------------------------------------------------- | ----------------- |
| `home.png`        | Home page with version, DB status, quick actions            | `/`               |
| `workspaces.png`  | Workspaces page with a scanned project + file tree visible  | `/workspaces`     |
| `graph.png`       | Dependency Graph page with nodes, edges, and a cycle warning| `/graph`          |
| `viewer.png`      | Code Viewer showing a TypeScript file with syntax highlight | `/viewer`         |
| `insights.png`    | Insights page with entry points and reading paths listed    | `/insights`       |

## How to capture

1. Build and run: `npm run tauri:dev`
2. Add a real TypeScript project folder (e.g. the CodeCompass repo itself)
3. Scan → Analyze
4. Navigate to each page and capture with
   `Win+Shift+S` (Snipping Tool) or your preferred tool.
5. Save into this directory with the exact filenames above.
6. The README references these paths; no further edits are needed once
   the files exist.

## Notes

- Redact any sensitive paths if your test repo contains them.
- Prefer a dark-themed OS for a consistent look with the app's `vs-dark`
  Monaco theme.
- After capturing, verify the README image links render on GitHub.
