use std::path::Path;

use crate::analysis::ts_js::{ImportRecord, ImportType, ParseResult};
use crate::analysis::LanguageAnalyzer;

/// A minimal CSS analyzer that extracts `@import` statements as import
/// records.  This serves as the **reference example** for implementing
/// new language analyzers without changing any core CodeCompass code.
///
/// ## How to add your own analyzer
///
/// 1. Implement [`LanguageAnalyzer`] for your struct.
/// 2. Register it in [`crate::analysis::plugin::build_registry`].
/// 3. Write tests that verify your parser extracts the right imports.
///
/// Only two trait methods are required:
/// - `supported_extensions()` — which file extensions your analyzer handles.
/// - `parse()` — return a [`ParseResult`] with imports and diagnostics.
///
/// The registry will automatically wire your analyzer into the scanner
/// (file discovery) and the analysis runner (query dispatch).
pub struct CssAnalyzer;

impl LanguageAnalyzer for CssAnalyzer {
    fn name(&self) -> &str {
        "CSS"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "Extracts @import and @url references from CSS files."
    }

    fn supported_extensions(&self) -> &'static [&'static str] {
        &["css"]
    }

    fn parse(
        &self,
        file_id: i64,
        _absolute_path: &Path,
        _workspace_root: &Path,
        source_text: &str,
    ) -> (ParseResult, bool) {
        let mut imports = Vec::new();
        let diagnostics = Vec::new();

        for (line_num, line) in source_text.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(specifier) = extract_import(trimmed) {
                imports.push(ImportRecord {
                    source_file_id: file_id,
                    target_specifier: specifier,
                    resolved_target: None,
                    import_type: ImportType::StaticImport,
                    is_external: false,
                    start_line: Some(line_num as i64 + 1),
                    start_column: Some(1),
                });
            }
        }

        let success = true;
        let result = ParseResult {
            imports,
            diagnostics,
        };
        (result, success)
    }
}

/// Extracts the URL from a CSS `@import` or `url()` statement.
///
/// Handles:
/// - `@import "path/to/file.css";`
/// - `@import 'path/to/file.css';`
/// - `@import url("path/to/file.css");`
/// - `@import url('path/to/file.css');`
fn extract_import(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    if !lower.starts_with("@import") {
        return None;
    }

    // After "@import", look for url( or a quote
    let rest = line[7..].trim();

    // url("...") or url('...')
    if let Some(inner) = rest
        .strip_prefix("url(")
        .and_then(|s| s.strip_suffix(')').or(Some(s)))
    {
        return extract_quoted(inner.trim());
    }

    // "..." or '...'
    extract_quoted(rest)
}

fn extract_quoted(s: &str) -> Option<String> {
    let trimmed = s.trim();
    for quote in ['"', '\''] {
        if let Some(inner) = trimmed
            .strip_prefix(quote)
            .and_then(|s| s.split(quote).next())
        {
            let cleaned = inner.split('?').next().unwrap_or(inner);
            return Some(cleaned.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_quoted_import() {
        let src = r#"@import "base.css";"#;
        let (result, success) = CssAnalyzer.parse(1, Path::new(""), Path::new(""), src);
        assert!(success);
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].target_specifier, "base.css");
    }

    #[test]
    fn single_quoted_import() {
        let src = r#"@import 'reset.css';"#;
        let (result, _success) = CssAnalyzer.parse(1, Path::new(""), Path::new(""), src);
        assert_eq!(result.imports[0].target_specifier, "reset.css");
    }

    #[test]
    fn url_import() {
        let src = r#"@import url("theme.css");"#;
        let (result, _) = CssAnalyzer.parse(1, Path::new(""), Path::new(""), src);
        assert_eq!(result.imports[0].target_specifier, "theme.css");
    }

    #[test]
    fn multiple_imports() {
        let src = "@import \"a.css\";\n@import \"b.css\";\nbody { color: red; }";
        let (result, _) = CssAnalyzer.parse(1, Path::new(""), Path::new(""), src);
        assert_eq!(result.imports.len(), 2);
    }

    #[test]
    fn no_imports() {
        let src = "body { color: red; }";
        let (result, _) = CssAnalyzer.parse(1, Path::new(""), Path::new(""), src);
        assert_eq!(result.imports.len(), 0);
    }

    #[test]
    fn empty_css() {
        let (result, success) = CssAnalyzer.parse(1, Path::new(""), Path::new(""), "");
        assert!(success);
        assert!(result.imports.is_empty());
    }

    #[test]
    fn strip_query_params() {
        let src = r#"@import "theme.css?v=2";"#;
        let (result, _) = CssAnalyzer.parse(1, Path::new(""), Path::new(""), src);
        assert_eq!(result.imports[0].target_specifier, "theme.css");
    }
}
