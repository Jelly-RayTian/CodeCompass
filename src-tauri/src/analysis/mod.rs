pub mod graph;
pub mod resolver;
pub mod runner;
pub mod ts_js;

pub use runner::run_analysis;
pub use ts_js::{parse_file as parse_ts_js_file, ImportRecord, ParseDiagnostic, ParseResult};

/// Abstract interface for language-specific source-code analyzers.
///
/// Implementations parse source files and produce import records,
/// diagnostics, and any language-specific metadata.
pub trait LanguageAnalyzer: Send + Sync {
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
