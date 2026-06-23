# Smoke Test Checklist

Run these steps before tagging a release.

## Clean Installation

- [ ] Delete `%LOCALAPPDATA%\io.github.jellyraytian.codecompass\`
- [ ] Run NSIS installer: `CodeCompass_x.x.x_x64-setup.exe`
- [ ] App installs without error
- [ ] Desktop shortcut created (if NSIS option selected)

## First Launch

- [ ] App starts without crash
- [ ] Home page shows "CodeCompass" + version
- [ ] Database status shows "Connected"
- [ ] No error banners on first load

## Workspace Registration

- [ ] Click "Add folder" → native folder picker opens
- [ ] Select a folder → folder appears in list with "Available" status
- [ ] Adding same folder again → shows duplicate error
- [ ] Adding nested folder → shows nesting warning

## Scanning

- [ ] Click "Scan folder" → phase shows "walking"
- [ ] Progress counter increments
- [ ] Scan completes with status "completed"
- [ ] "View files" → file tree populated
- [ ] Click file → file details panel shows metadata

## Cancellation

- [ ] Start scan → click "Cancel scan"
- [ ] Scan stops → status shows "cancelled"
- [ ] Previously indexed files remain visible

## Analysis

- [ ] After successful scan, click "Analyze"
- [ ] Progress bar shows file counts
- [ ] Analysis completes
- [ ] Click file → imports panel shows resolved imports

## Symbol Search

- [ ] In Workspaces sidebar, type symbol name → results appear
- [ ] Click result → navigates to Viewer page with source displayed
- [ ] Kind filter dropdown filters correctly

## Dependency Graph

- [ ] Navigate to Dependency Graph page
- [ ] Select workspace → graph renders with nodes/edges
- [ ] Click node → detail panel shows imports/imported-by
- [ ] "View source" navigates to Viewer

## Viewer

- [ ] Source code displayed with syntax highlighting
- [ ] Line numbers visible
- [ ] Search within file works (Ctrl+F in Monaco)
- [ ] Large files show truncation warning

## Insights

- [ ] Navigate to Insights page
- [ ] Select workspace → entry points listed
- [ ] Reading path shows numbered order
- [ ] Structural findings appear with evidence

## Git Panel

- [ ] On a Git workspace → branch/status/commit count shown
- [ ] Workspace settings toggles work
- [ ] Co-change hotspots shown (if commit history available)

## Restart Persistence

- [ ] Close app
- [ ] Reopen app
- [ ] Workspace still listed
- [ ] Files, imports, symbols, graph data intact

## Language Switching

- [ ] Click "中文" button → UI switches to Chinese
- [ ] Click "EN" → switches back to English
- [ ] Close and reopen → language preference persists

## Uninstall

- [ ] Settings → Apps → CodeCompass → Uninstall
- [ ] Application removed from system
- [ ] Original source files remain untouched

## Offline Operation

- [ ] Disconnect from internet
- [ ] App launches and all features work
- [ ] No telemetry, no network requests visible
