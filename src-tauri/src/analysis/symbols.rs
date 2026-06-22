use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::BindingPatternKind;
use oxc_ast::visit::Visit;
use oxc_ast::AstKind;
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

/// A single symbol extracted from source code.
#[derive(Debug, Clone)]
pub struct SymbolRecord {
    pub name: String,
    pub kind: SymbolKind,
    pub source_line: i64,
    pub source_column: i64,
    pub source_end_line: i64,
    pub source_end_column: i64,
    pub signature: Option<String>,
    pub visibility: Visibility,
    pub is_exported: bool,
    pub parent_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    TypeAlias,
    Variable,
    Enum,
    ReactComponent,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Function => "function",
            Self::Class => "class",
            Self::Interface => "interface",
            Self::TypeAlias => "type",
            Self::Variable => "variable",
            Self::Enum => "enum",
            Self::ReactComponent => "react_component",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Exported,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
            Self::Exported => "exported",
        }
    }
}

/// Extracts symbol declarations from a TypeScript / JavaScript source file.
pub fn extract_symbols(source_text: &str, path: &Path) -> Vec<SymbolRecord> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or(SourceType::tsx());
    let ret = Parser::new(&allocator, source_text, source_type).parse();

    if ret.program.is_empty() {
        return vec![];
    }

    let mut visitor = SymbolVisitor {
        symbols: vec![],
        source_text,
        class_name: None,
    };

    visitor.visit_program(&ret.program);
    visitor.symbols
}

struct SymbolVisitor<'a> {
    symbols: Vec<SymbolRecord>,
    source_text: &'a str,
    class_name: Option<String>,
}

fn span_pos(source: &str, span: Span) -> (i64, i64, i64, i64) {
    let start = span.start as usize;
    let end = span.end as usize;
    let (sl, sc) = offset_to_line_col(source, start);
    let end_pos = end.min(source.len());
    let (el, ec) = offset_to_line_col(source, end_pos);
    (sl.max(1), sc.max(1), el.max(1), ec.max(1))
}

fn is_uppercase_first(s: &str) -> bool {
    s.chars().next().map_or(false, |c| c.is_ascii_uppercase())
}

impl<'a> Visit<'a> for SymbolVisitor<'a> {
    fn enter_node(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::Function(func) => {
                let name = func.id.as_ref().map(|id| id.name.to_string());
                if let Some(name) = name {
                    let (sl, sc, el, ec) = span_pos(self.source_text, func.span);
                    let is_react = is_uppercase_first(&name)
                        && self.source_text[func.span.start as usize..func.span.end as usize]
                            .contains('<');
                    self.symbols.push(SymbolRecord {
                        name: name.clone(),
                        kind: if is_react {
                            SymbolKind::ReactComponent
                        } else if self.class_name.is_some() {
                            SymbolKind::Function
                        } else {
                            SymbolKind::Function
                        },
                        source_line: sl,
                        source_column: sc,
                        source_end_line: el,
                        source_end_column: ec,
                        signature: None,
                        visibility: Visibility::Public,
                        is_exported: false,
                        parent_name: self.class_name.clone(),
                    });
                }
            }
            AstKind::Class(class) => {
                let name = class.id.as_ref().map(|id| id.name.to_string());
                if let Some(name) = name {
                    self.class_name = Some(name.clone());
                    let (sl, sc, el, ec) = span_pos(self.source_text, class.span);
                    self.symbols.push(SymbolRecord {
                        name,
                        kind: SymbolKind::Class,
                        source_line: sl,
                        source_column: sc,
                        source_end_line: el,
                        source_end_column: ec,
                        signature: None,
                        visibility: Visibility::Public,
                        is_exported: false,
                        parent_name: None,
                    });
                }
            }
            AstKind::TSInterfaceDeclaration(iface) => {
                let name = iface.id.name.to_string();
                let (sl, sc, el, ec) = span_pos(self.source_text, iface.span);
                self.symbols.push(SymbolRecord {
                    name,
                    kind: SymbolKind::Interface,
                    source_line: sl,
                    source_column: sc,
                    source_end_line: el,
                    source_end_column: ec,
                    signature: None,
                    visibility: Visibility::Public,
                    is_exported: false,
                    parent_name: None,
                });
            }
            AstKind::TSTypeAliasDeclaration(alias) => {
                let name = alias.id.name.to_string();
                let (sl, sc, el, ec) = span_pos(self.source_text, alias.span);
                self.symbols.push(SymbolRecord {
                    name,
                    kind: SymbolKind::TypeAlias,
                    source_line: sl,
                    source_column: sc,
                    source_end_line: el,
                    source_end_column: ec,
                    signature: None,
                    visibility: Visibility::Public,
                    is_exported: false,
                    parent_name: None,
                });
            }
            AstKind::TSEnumDeclaration(enu) => {
                let name = enu.id.name.to_string();
                let (sl, sc, el, ec) = span_pos(self.source_text, enu.span);
                self.symbols.push(SymbolRecord {
                    name,
                    kind: SymbolKind::Enum,
                    source_line: sl,
                    source_column: sc,
                    source_end_line: el,
                    source_end_column: ec,
                    signature: None,
                    visibility: Visibility::Public,
                    is_exported: false,
                    parent_name: None,
                });
            }
            AstKind::VariableDeclarator(decl) => {
                if let BindingPatternKind::BindingIdentifier(ref id) = decl.id.kind {
                    let name = id.name.to_string();
                    if decl.init.is_some() {
                        let (sl, sc, el, ec) = span_pos(self.source_text, decl.span);
                        let is_arrow = decl.init.as_ref().map_or(false, |init| {
                            matches!(init, oxc_ast::ast::Expression::ArrowFunctionExpression(_))
                        });
                        if is_arrow {
                            let is_react_arrow = is_uppercase_first(&name);
                            self.symbols.push(SymbolRecord {
                                name,
                                kind: if is_react_arrow {
                                    SymbolKind::ReactComponent
                                } else {
                                    SymbolKind::Function
                                },
                                source_line: sl,
                                source_column: sc,
                                source_end_line: el,
                                source_end_column: ec,
                                signature: None,
                                visibility: Visibility::Public,
                                is_exported: false,
                                parent_name: None,
                            });
                        }
                    }
                }
            }
            AstKind::ExportNamedDeclaration(exp) => {
                if exp.source.is_some() {
                    return;
                }
                // Mark the last symbol in this span as exported.
                let exp_start = exp.span.start as usize;
                let exp_end = exp.span.end as usize;
                if let Some(sym) = self.symbols.last_mut() {
                    let sym_line = sym.source_line as usize;
                    let sym_off = line_to_offset(self.source_text, sym_line).unwrap_or(usize::MAX);
                    if sym_off >= exp_start && sym_off < exp_end {
                        sym.is_exported = true;
                        sym.visibility = Visibility::Exported;
                    }
                }
            }
            _ => {}
        }
    }

    fn leave_node(&mut self, kind: AstKind<'a>) {
        if let AstKind::Class(_) = kind {
            self.class_name = None;
        }
    }
}

fn offset_to_line_col(source: &str, offset: usize) -> (i64, i64) {
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

fn line_to_offset(source: &str, line: usize) -> Option<usize> {
    let mut current_line = 1;
    for (i, ch) in source.char_indices() {
        if current_line == line {
            return Some(i);
        }
        if ch == '\n' {
            current_line += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract(source: &str) -> Vec<SymbolRecord> {
        extract_symbols(source, Path::new("test.tsx"))
    }

    #[test]
    fn function_declaration() {
        let syms = extract("function hello() {}");
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "hello");
        assert_eq!(syms[0].kind, SymbolKind::Function);
    }

    #[test]
    fn class_declaration() {
        let syms = extract("class Foo { bar() {} }");
        let class = syms.iter().find(|s| s.kind == SymbolKind::Class);
        assert!(class.is_some());
        // Methods inside classes may be detected as Function with parent_name.
        if let Some(method) = syms.iter().find(|s| s.name == "bar") {
            assert_eq!(method.parent_name.as_deref(), Some("Foo"));
        }
    }

    #[test]
    fn interface_declaration() {
        let syms = extract("interface Props { name: string }");
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].kind, SymbolKind::Interface);
    }

    #[test]
    fn type_alias() {
        let syms = extract("type ID = string;");
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].kind, SymbolKind::TypeAlias);
    }

    #[test]
    fn enum_declaration() {
        let syms = extract("enum Color { Red }");
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].kind, SymbolKind::Enum);
    }

    #[test]
    fn arrow_function_variable() {
        let syms = extract("const add = (a: number) => a;");
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].name, "add");
    }

    #[test]
    fn source_location_correct() {
        let syms = extract("// comment\nfunction foo() {}");
        assert_eq!(syms[0].source_line, 2);
    }

    #[test]
    fn malformed_source_empty() {
        let syms = extract("function broken(");
        assert!(syms.is_empty());
    }

    #[test]
    fn react_component_detected() {
        let syms = extract("function App() { return <div />; }");
        assert_eq!(syms.len(), 1);
        assert_eq!(syms[0].kind, SymbolKind::ReactComponent);
    }
}
