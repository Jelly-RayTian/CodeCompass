use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, ExportAllDeclaration, ExportNamedDeclaration, Expression,
    ImportDeclaration, ImportExpression,
};
use oxc_ast::visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::resolver::resolve_import;

/// A single import relationship extracted from source code.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImportRecord {
    pub source_file_id: i64,
    pub target_specifier: String,
    pub resolved_target: Option<PathBuf>,
    pub import_type: ImportType,
    pub is_external: bool,
    pub start_line: Option<i64>,
    pub start_column: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportType {
    StaticImport,
    DynamicImport,
    Require,
    ReExport,
    ReExportAll,
}

impl ImportType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StaticImport => "static_import",
            Self::DynamicImport => "dynamic_import",
            Self::Require => "require",
            Self::ReExport => "re_export",
            Self::ReExportAll => "re_export_all",
        }
    }
}

/// Diagnostic emitted by the parser.
#[derive(Debug, Clone)]
pub struct ParseDiagnostic {
    pub severity: String,
    pub message: String,
    pub line: Option<i64>,
    pub column: Option<i64>,
}

/// Result of parsing a single file.
#[derive(Debug)]
pub struct ParseResult {
    pub imports: Vec<ImportRecord>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

/// Analyses a TypeScript or JavaScript source file and extracts:
/// - static imports
/// - re-exports (`export … from …`)
/// - dynamic imports (`import(…)`)
/// - CommonJS `require(…)` calls
pub fn parse_file(
    file_id: i64,
    absolute_path: &Path,
    workspace_root: &Path,
    source_text: &str,
) -> (ParseResult, bool) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(absolute_path).unwrap_or(SourceType::tsx());
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    if !ret.errors.is_empty() {
        let diagnostics: Vec<ParseDiagnostic> = ret
            .errors
            .iter()
            .map(|e| ParseDiagnostic {
                severity: "error".to_string(),
                message: format!("{}", e.message),
                line: None,
                column: None,
            })
            .collect();
        // OXC uses error recovery, so the program may still be available.
        if ret.program.is_empty() {
            return (
                ParseResult {
                    imports: vec![],
                    diagnostics,
                },
                false,
            );
        }
        let mut visitor = ImportVisitor {
            file_id,
            source_dir: absolute_path
                .parent()
                .unwrap_or(Path::new("."))
                .to_path_buf(),
            workspace_root: workspace_root.to_path_buf(),
            source_text,
            imports: vec![],
        };
        visitor.visit_program(&ret.program);
        return (
            ParseResult {
                imports: visitor.imports,
                diagnostics,
            },
            true,
        );
    }

    let program = ret.program;
    let source_dir = absolute_path.parent().unwrap_or(Path::new("."));
    let mut visitor = ImportVisitor {
        file_id,
        source_dir: source_dir.to_path_buf(),
        workspace_root: workspace_root.to_path_buf(),
        source_text,
        imports: vec![],
    };

    visitor.visit_program(&program);

    (
        ParseResult {
            imports: visitor.imports,
            diagnostics: vec![],
        },
        true,
    )
}

fn offset_to_line_col(source: &str, offset: u32) -> (i64, i64) {
    let offset = offset as usize;
    let mut line: i64 = 1;
    let mut col: i64 = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

struct ImportVisitor<'a> {
    file_id: i64,
    source_dir: PathBuf,
    workspace_root: PathBuf,
    source_text: &'a str,
    imports: Vec<ImportRecord>,
}

impl<'a> ImportVisitor<'a> {
    fn record(&mut self, specifier: &str, import_type: ImportType, offset: u32) {
        if specifier.is_empty() {
            return;
        }
        let (line, col) = offset_to_line_col(self.source_text, offset);

        let (resolved_target, is_external) =
            match resolve_import(&self.workspace_root, &self.source_dir, specifier) {
                Ok(Some(path)) => (Some(path), false),
                Ok(None) => (None, true),
                Err(()) => (None, false),
            };

        self.imports.push(ImportRecord {
            source_file_id: self.file_id,
            target_specifier: specifier.to_string(),
            resolved_target,
            import_type,
            is_external,
            start_line: Some(line),
            start_column: Some(col),
        });
    }
}

impl<'a> Visit<'a> for ImportVisitor<'a> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        let specifier = decl.source.value.to_string();
        let offset = decl.source.span.start;
        self.record(&specifier, ImportType::StaticImport, offset);
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        if let Some(source) = &decl.source {
            let specifier = source.value.to_string();
            let offset = source.span.start;
            self.record(&specifier, ImportType::ReExport, offset);
        }
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        let specifier = decl.source.value.to_string();
        let offset = decl.source.span.start;
        self.record(&specifier, ImportType::ReExportAll, offset);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Expression::StringLiteral(s) = &expr.source {
            let specifier = s.value.to_string();
            let offset = s.span.start;
            self.record(&specifier, ImportType::DynamicImport, offset);
        }
    }

    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        match &expr.callee {
            Expression::Identifier(ident) if ident.name == "require" => {
                if let Some(Argument::StringLiteral(s)) = expr.arguments.first() {
                    let specifier = s.value.to_string();
                    let offset = s.span.start;
                    self.record(&specifier, ImportType::Require, offset);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn parse_source(source: &str, filename: &str) -> ParseResult {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join(filename);
        let (result, _) = parse_file(1, &path, dir.path(), source);
        result
    }

    #[test]
    fn static_import_default() {
        let result = parse_source("import React from 'react';", "a.tsx");
        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].is_external);
        assert_eq!(result.imports[0].import_type, ImportType::StaticImport);
    }

    #[test]
    fn static_import_named() {
        let result = parse_source("import { useState } from 'react';", "a.tsx");
        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].is_external);
    }

    #[test]
    fn re_export() {
        let result = parse_source("export { foo } from './bar';", "a.ts");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].import_type, ImportType::ReExport);
    }

    #[test]
    fn re_export_all() {
        let result = parse_source("export * from './bar';", "a.ts");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].import_type, ImportType::ReExportAll);
    }

    #[test]
    fn require_call() {
        let result = parse_source("const fs = require('fs');", "a.js");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].import_type, ImportType::Require);
    }

    #[test]
    fn dynamic_import() {
        let result = parse_source("const m = import('./lazy');", "a.ts");
        assert_eq!(result.imports.len(), 1);
        assert_eq!(result.imports[0].import_type, ImportType::DynamicImport);
    }

    #[test]
    fn relative_import_resolves() {
        let dir = tempdir().expect("tempdir");
        let src = dir.path().join("src");
        std::fs::create_dir(&src).expect("create");
        std::fs::write(src.join("bar.ts"), "export const x = 1;").expect("write");
        let file_path = src.join("app.ts");
        let source = "import { x } from './bar';";
        let (result, _) = parse_file(1, &file_path, dir.path(), source);
        assert_eq!(result.imports.len(), 1);
        assert!(result.imports[0].resolved_target.is_some());
    }

    #[test]
    fn malformed_file_produces_diagnostics() {
        let result = parse_source("import { from 'react'", "a.ts");
        assert!(!result.diagnostics.is_empty());
    }

    #[test]
    fn empty_file_no_imports() {
        let result = parse_source("", "a.ts");
        assert!(result.imports.is_empty());
    }

    #[test]
    fn multiple_imports() {
        let result = parse_source(
            "import A from 'a';\nimport B from 'b';\nconst C = require('c');",
            "a.ts",
        );
        assert_eq!(result.imports.len(), 3);
    }
}
