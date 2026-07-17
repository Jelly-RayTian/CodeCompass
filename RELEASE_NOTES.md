# CodeCompass v0.4.0 Release Notes

**Release date:** 2026-07-16  
**Full changelog:** [CHANGELOG.md](./CHANGELOG.md)

## Overview

v0.4.0 introduces a **lightweight plugin API** for extending CodeCompass with
new language analyzers without modifying core code. A CSS analyzer is included
as a reference example, demonstrating the full plugin lifecycle from registration
to import extraction.

## Highlights

### AnalyzerRegistry

- New `AnalyzerRegistry` maps file extensions to `LanguageAnalyzer` implementations.
- Both the **scanner** (file discovery) and the **analysis runner** (dispatch)
  consult the registry — adding a new analyzer automatically includes its
  files in scanning and analysis.
- SQL queries for file selection are built dynamically from registered extensions.

### Enhanced LanguageAnalyzer trait

Every analyzer now exposes metadata:
- `name()` — human-readable plugin name
- `version()` — semantic version
- `description()` — what the analyzer extracts

### CSS analyzer (reference plugin)

A complete, minimal example of how to add a new language:
- Handles `.css` files
- Extracts `@import "file.css"`, `@import url(...)`, and single-quoted variants
- Strips query parameters (`file.css?v=2` → `file.css`)
- 7 unit tests

### Plugin info command

New `get_plugin_info` Tauri command exposes registered plugins to the frontend.

### Architecture documentation

New `docs/plugin-architecture.md` with a step-by-step guide (3 steps) for
adding a new language analyzer.

## Plugin architecture

```
Register in build_registry()
         │
    ┌────▼────┐
    │Registry │ extension → Arc<dyn LanguageAnalyzer>
    └─┬────┬──┘
      │    │
  Scanner  Runner
```

## Known limitations

- **Compile-time only.** Plugins are statically linked — no runtime `dlopen`
  or dynamic loading. New analyzers require rebuilding the binary.
- CSS analyzer does not extract symbols or references — only imports.
- Extensions are limited to those known at compile time.

## Installers

- **NSIS:** `CodeCompass_0.4.0_x64-setup.exe`
- **MSI:** `CodeCompass_0.4.0_x64_en-US.msi`

> Installers are **unsigned** — Windows SmartScreen may warn. Click "More info" → "Run anyway".

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
