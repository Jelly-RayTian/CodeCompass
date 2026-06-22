use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast::visit::Visit;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

/// A symbol reference (call, instantiation, property access).
#[derive(Debug, Clone)]
pub struct SymbolReference {
    pub callee_name: String,
    pub reference_type: ReferenceType,
    pub source_line: i64,
    pub source_column: i64,
    pub enclosing_function: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceType {
    Call,
    NewExpression,
    PropertyAccess,
}

impl ReferenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::NewExpression => "new",
            Self::PropertyAccess => "property",
        }
    }
}

/// Extracts symbol-level call/reference relationships from source.
pub fn extract_references(source_text: &str, path: &Path) -> Vec<SymbolReference> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or(SourceType::tsx());
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    if ret.program.is_empty() {
        return vec![];
    }

    let mut visitor = RefVisitor {
        references: vec![],
        source_text,
        enclosing_function: None,
    };
    visitor.visit_program(&ret.program);
    visitor.references
}

struct RefVisitor<'a> {
    references: Vec<SymbolReference>,
    source_text: &'a str,
    enclosing_function: Option<String>,
}

fn span_line_col(source: &str, span: Span) -> (i64, i64) {
    let mut line: i64 = 1;
    let mut col: i64 = 1;
    let offset = span.start as usize;
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

impl<'a> Visit<'a> for RefVisitor<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(func) => {
                self.enclosing_function = func.id.as_ref().map(|id| id.name.to_string());
            }
            AstKind::CallExpression(expr) => {
                self.record_call(expr);
            }
            AstKind::NewExpression(expr) => {
                if let Expression::Identifier(id) = &expr.callee {
                    let (line, col) = span_line_col(self.source_text, expr.span);
                    self.references.push(SymbolReference {
                        callee_name: id.name.to_string(),
                        reference_type: ReferenceType::NewExpression,
                        source_line: line,
                        source_column: col,
                        enclosing_function: self.enclosing_function.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        if matches!(kind, AstKind::Function(_)) {
            self.enclosing_function = None;
        }
    }
}

impl<'a> RefVisitor<'a> {
    fn record_call(&mut self, expr: &CallExpression<'a>) {
        let callee_name = match &expr.callee {
            Expression::Identifier(id) => Some(id.name.to_string()),
            Expression::StaticMemberExpression(member) => {
                if let Expression::Identifier(obj) = &member.object {
                    Some(format!("{}.{}", obj.name, member.property.name))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(name) = callee_name {
            let (line, col) = span_line_col(self.source_text, expr.span);
            self.references.push(SymbolReference {
                callee_name: name,
                reference_type: ReferenceType::Call,
                source_line: line,
                source_column: col,
                enclosing_function: self.enclosing_function.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(src: &str) -> Vec<SymbolReference> {
        extract_references(src, Path::new("test.ts"))
    }

    #[test]
    fn simple_call() {
        let refs = extract("foo();");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].callee_name, "foo");
        assert_eq!(refs[0].reference_type, ReferenceType::Call);
    }

    #[test]
    fn method_call() {
        let refs = extract("obj.bar();");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].callee_name, "obj.bar");
    }

    #[test]
    fn new_expression() {
        let refs = extract("new Foo();");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].reference_type, ReferenceType::NewExpression);
    }

    #[test]
    fn call_inside_function() {
        let refs = extract("function outer() { inner(); }");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].enclosing_function.as_deref(), Some("outer"));
    }

    #[test]
    fn multiple_calls() {
        let refs = extract("a(); b(); c();");
        assert_eq!(refs.len(), 3);
    }

    #[test]
    fn no_calls() {
        let refs = extract("const x = 1;");
        assert!(refs.is_empty());
    }
}
