# Plugin Architecture

CodeCompass uses a lightweight, compile-time plugin system to support
multiple programming languages without modifying core analysis code.

## Overview

```
┌──────────────────────────────────────┐
│          Register your analyzer       │
│          in build_registry()          │
└──────────────┬───────────────────────┘
               │
┌──────────────▼───────────────────────┐
│          AnalyzerRegistry             │
│  extension → Arc<dyn LanguageAnalyzer>│
└──────────┬──────────────┬────────────┘
           │              │
  ┌────────▼───┐   ┌──────▼────────┐
  │  Scanner    │   │  Runner       │
  │  (discovery)│   │  (analysis)   │
  └─────────────┘   └───────────────┘
```

The scanner and analysis runner both consult the `AnalyzerRegistry`:
- **Scanner**: checks if a file's extension is handled by any registered analyzer before indexing it.
- **Runner**: looks up the right analyzer for each file's extension when performing AST analysis.

## LanguageAnalyzer trait

Every analyzer implements the `LanguageAnalyzer` trait defined in
`src-tauri/src/analysis/mod.rs`:

```rust
pub trait LanguageAnalyzer: Send + Sync {
    /// Human-readable plugin name, e.g. "TypeScript/JavaScript".
    fn name(&self) -> &str;

    /// Semantic version of the analyzer plugin.
    fn version(&self) -> &str;

    /// Short description of what the analyzer extracts.
    fn description(&self) -> &str;

    /// File extensions this analyzer handles (e.g. ["ts", "tsx", "js", "jsx"]).
    fn supported_extensions(&self) -> &'static [&'static str];

    /// Parse a source file and return imports + diagnostics.
    fn parse(
        &self,
        file_id: i64,
        absolute_path: &Path,
        workspace_root: &Path,
        source_text: &str,
    ) -> (ParseResult, bool);
}
```

## How to add a new analyzer

### Step 1: Create your analyzer module

Create a new file under `src-tauri/src/analysis/`, e.g. `my_lang.rs`:

```rust
use crate::analysis::{
    LanguageAnalyzer,
    ts_js::{ImportRecord, ImportType, ParseResult},
};

pub struct MyLanguageAnalyzer;

impl LanguageAnalyzer for MyLanguageAnalyzer {
    fn name(&self) -> &str { "My Language" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str {
        "Extracts import-like relationships from .mylang files."
    }
    fn supported_extensions(&self) -> &'static [&'static str] {
        &["mylang"]
    }

    fn parse(
        &self,
        file_id: i64,
        _absolute_path: &Path,
        _workspace_root: &Path,
        source_text: &str,
    ) -> (ParseResult, bool) {
        let mut imports = Vec::new();
        // ... parse source_text, populate imports ...
        let result = ParseResult {
            imports,
            diagnostics: Vec::new(),
        };
        (result, true)
    }
}
```

### Step 2: Register in the registry

In `src-tauri/src/analysis/plugin.rs`, add your module and register:

```rust
mod my_lang;  // in analysis/mod.rs

// in plugin.rs → build_registry():
reg.register(MyLanguageAnalyzer);
```

### Step 3: Test your analyzer

Write tests that verify your parser extracts the expected import records.
See `css_analyzer.rs` tests for examples.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_imports() {
        let src = "import helper from './helper.mylang';";
        let (result, _) = MyLanguageAnalyzer.parse(1, Path::new(""), Path::new(""), src);
        assert_eq!(result.imports.len(), 1);
    }
}
```

That's it. The scanner will now discover `.mylang` files, and the analysis
runner will dispatch them to your analyzer automatically.

## Reference: CSS Analyzer (example plugin)

`src-tauri/src/analysis/css_analyzer.rs` is a complete, minimal example:

| Feature           | Implementation                     |
|-------------------|------------------------------------|
| Language          | CSS                                |
| Extensions        | `.css`                             |
| Imports extracted | `@import "..."`, `@import url(...)` |
| symbols extracted | None                               |
| Tests             | 7 unit tests                       |

## Plugin registry API

The registry is built once at program start via `build_registry()`.
At runtime you can query:

```rust
let registry = build_registry();

// Check if an extension is handled:
registry.resolve("css").is_some();   // true
registry.resolve("png").is_none();   // true

// Get all supported extensions (for scanning):
registry.all_extensions();           // ["css", "js", "jsx", "ts", "tsx"]

// Inspect registered plugins (exposed via Tauri command):
registry.plugin_list();              // Vec<PluginInfo>
registry.plugin_count();             // 2
```

The `get_plugin_info` Tauri command exposes plugin metadata to the frontend
for diagnostics and debugging.

## Design decisions

- **Compile-time registration.** Plugins are statically linked — no `dlopen`,
  no dynamic loading. This keeps the Tauri binary self-contained and avoids
  security concerns with runtime code loading.
- **Extension-centric dispatch.** The registry maps file extensions to
  analyzers, so one analyzer can handle multiple extensions (e.g. `.ts`,
  `.tsx`, `.js`, `.jsx`).
- **Minimal trait surface.** Only 5 methods are required: 3 metadata + 2
  functional (`supported_extensions`, `parse`). This keeps the barrier to
  entry low for new language analyzers.
- **Arc-based sharing.** Analyzers are stored behind `Arc<dyn LanguageAnalyzer>`
  so multiple extensions from the same analyzer share one instance.
