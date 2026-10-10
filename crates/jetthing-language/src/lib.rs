use std::path::Path;
use tree_sitter::{Node, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    Keyword,
    String,
    Comment,
    Number,
    Type,
    Function,
    Macro,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxSpan {
    pub start_byte: usize,
    pub end_byte: usize,
    pub kind: SyntaxKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LanguageAnalysis {
    pub language: Option<&'static str>,
    pub spans: Vec<SyntaxSpan>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn analyze(path: &Path, source: &str) -> LanguageAnalysis {
    match path.extension().and_then(|value| value.to_str()) {
        Some("rs") => analyze_rust(source),
        _ => LanguageAnalysis::default(),
    }
}

fn analyze_rust(source: &str) -> LanguageAnalysis {
    let mut parser = Parser::new();
    if parser
        .set_language(&tree_sitter_rust::LANGUAGE.into())
        .is_err()
    {
        return LanguageAnalysis::default();
    }
    let Some(tree) = parser.parse(source, None) else {
        return LanguageAnalysis::default();
    };
    let mut result = LanguageAnalysis {
        language: Some("Rust"),
        ..Default::default()
    };
    collect(tree.root_node(), source, &mut result);
    result
        .spans
        .sort_by_key(|span| (span.start_byte, span.end_byte));
    result
}

fn collect(node: Node<'_>, source: &str, result: &mut LanguageAnalysis) {
    if node.is_error() || node.is_missing() {
        let point = node.start_position();
        result.diagnostics.push(Diagnostic {
            message: if node.is_missing() {
                format!("expected {}", node.kind())
            } else {
                "syntax error".into()
            },
            row: point.row + 1,
            column: point.column + 1,
        });
    }
    if let Some(kind) = classify(node, source) {
        result.spans.push(SyntaxSpan {
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            kind,
        });
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect(child, source, result);
    }
}

fn classify(node: Node<'_>, source: &str) -> Option<SyntaxKind> {
    let kind = node.kind();
    if kind.contains("comment") {
        return Some(SyntaxKind::Comment);
    }
    if kind.contains("string") || kind == "char_literal" {
        return Some(SyntaxKind::String);
    }
    if kind.contains("integer") || kind.contains("float") {
        return Some(SyntaxKind::Number);
    }
    if matches!(kind, "primitive_type" | "type_identifier") {
        return Some(SyntaxKind::Type);
    }
    if kind == "identifier" {
        let parent = node.parent().map(|p| p.kind());
        if matches!(parent, Some("function_item" | "call_expression")) {
            return Some(SyntaxKind::Function);
        }
        if matches!(parent, Some("macro_invocation")) {
            return Some(SyntaxKind::Macro);
        }
    }
    if !node.is_named() {
        let token = source.get(node.byte_range()).unwrap_or_default();
        if matches!(
            token,
            "as" | "async"
                | "await"
                | "break"
                | "const"
                | "continue"
                | "crate"
                | "dyn"
                | "else"
                | "enum"
                | "extern"
                | "false"
                | "fn"
                | "for"
                | "if"
                | "impl"
                | "in"
                | "let"
                | "loop"
                | "match"
                | "mod"
                | "move"
                | "mut"
                | "pub"
                | "ref"
                | "return"
                | "self"
                | "Self"
                | "static"
                | "struct"
                | "super"
                | "trait"
                | "true"
                | "type"
                | "unsafe"
                | "use"
                | "where"
                | "while"
        ) {
            return Some(SyntaxKind::Keyword);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rust_analysis_returns_highlights_without_diagnostics() {
        let analysis = analyze(Path::new("lib.rs"), "pub fn answer() -> u32 { 42 }\n");
        assert_eq!(analysis.language, Some("Rust"));
        assert!(analysis.diagnostics.is_empty());
        assert!(
            analysis
                .spans
                .iter()
                .any(|span| span.kind == SyntaxKind::Keyword)
        );
        assert!(
            analysis
                .spans
                .iter()
                .any(|span| span.kind == SyntaxKind::Number)
        );
    }
    #[test]
    fn rust_analysis_reports_parse_errors() {
        assert!(
            !analyze(Path::new("broken.rs"), "fn broken( {")
                .diagnostics
                .is_empty()
        );
    }
    #[test]
    fn unknown_files_remain_plain_text() {
        assert!(analyze(Path::new("notes.txt"), "hello").language.is_none());
    }
}
