pub mod call_graph;
pub mod css_analyzer;
pub mod entrypoint;
pub mod evolution;
pub mod findings;
pub mod graph;
pub mod health;
pub mod impact;
pub mod plugin;
pub mod reading_path;
pub mod references;
pub mod resolver;
pub mod runner;
pub mod symbols;
pub mod ts_js;

pub use runner::run_analysis;
pub use ts_js::{parse_file as parse_ts_js_file, ImportRecord, ParseDiagnostic, ParseResult};

/// Abstract interface for language-specific source-code analyzers.
///
/// Implementations parse source files and produce import records,
/// diagnostics, and any language-specific metadata.
///
/// ## Plugin system
///
/// New language analyzers implement this trait and register in
/// [`plugin::build_registry`]. See [`css_analyzer::CssAnalyzer`] for
/// a complete example.
#[allow(dead_code)]
pub trait LanguageAnalyzer: Send + Sync {
    /// Human-readable plugin name, e.g. "TypeScript/JavaScript".
    fn name(&self) -> &str;

    /// Semantic version of the analyzer plugin.
    fn version(&self) -> &str;

    /// Short description of what the analyzer extracts.
    fn description(&self) -> &str;

    /// Returns the file extensions this analyzer handles (e.g. `["ts",
    /// "tsx", "js", "jsx"]`).
    fn supported_extensions(&self) -> &'static [&'static str];

    /// Parses a source file and returns extracted imports and diagnostics.
    ///
    /// * `file_id` — the `indexed_files.id` of the file.
    /// * `absolute_path` — filesystem path to the source file.
    /// * `workspace_root` — the workspace root directory.
    /// * `source_text` — UTF-8 source content.
    fn parse(
        &self,
        file_id: i64,
        absolute_path: &std::path::Path,
        workspace_root: &std::path::Path,
        source_text: &str,
    ) -> (ParseResult, bool);
}

/// The default TypeScript/JavaScript analyzer.
pub struct TypeScriptJavaScriptAnalyzer;

impl LanguageAnalyzer for TypeScriptJavaScriptAnalyzer {
    fn name(&self) -> &str {
        "TypeScript/JavaScript"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "OXC-based AST parser for TypeScript and JavaScript. Extracts imports, exports, symbols, and references."
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx", "js", "jsx"]
    }

    fn parse(
        &self,
        file_id: i64,
        absolute_path: &std::path::Path,
        workspace_root: &std::path::Path,
        source_text: &str,
    ) -> (ParseResult, bool) {
        parse_ts_js_file(file_id, absolute_path, workspace_root, source_text)
    }
}
