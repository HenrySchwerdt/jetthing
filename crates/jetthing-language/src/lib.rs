//! Tree-sitter language platform: a data-driven registry of grammar crates and
//! their bundled queries, a generic highlight engine, and IntelliJ color
//! schemes. No language has its own code path here.

mod captures;
pub mod highlight;
mod injection;
pub mod registry;
pub mod scheme;
pub mod selection;
pub mod syntax;

pub use captures::capture_key;
pub use highlight::{HighlightLayer, HighlightSpan, QueryError};
pub use jetthing_text::{ByteEdit, BytePoint};
pub use registry::{LANGUAGES, LanguageSpec};
pub use scheme::{ColorScheme, EffectType, FontStyle, Rgba, TextAttributes};
pub use syntax::SyntaxTree;

use std::path::Path;
use tree_sitter::Node;

/// Maximum number of syntax diagnostics reported per file.
const MAX_DIAGNOSTICS: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LanguageAnalysis {
    /// Display name of the detected language.
    pub language: Option<&'static str>,
    /// Syntax highlighting from the grammar's highlight query.
    pub highlights: HighlightLayer,
    pub diagnostics: Vec<Diagnostic>,
    /// The incrementally maintained syntax tree, when the language is known.
    pub syntax: Option<SyntaxTree>,
}

impl LanguageAnalysis {
    /// Extend Selection: the next larger syntactic byte range around `range`.
    pub fn expand_selection(
        &self,
        source: &str,
        range: std::ops::Range<usize>,
    ) -> Option<std::ops::Range<usize>> {
        selection::expand(self.syntax.as_ref()?.tree(), source, range)
    }

    /// Brings the analysis up to date with `source` after `edits` (as
    /// journaled by the text buffer): applies them to the syntax tree,
    /// reparses incrementally and re-highlights only the affected ranges.
    pub fn apply_edits(&mut self, path: &Path, source: &str, edits: &[ByteEdit]) {
        if edits.is_empty() {
            return;
        }
        let Some(syntax) = self.syntax.as_mut() else {
            // Plain text may become a known language (e.g. a new shebang line).
            if registry::detect(path, source).is_some() {
                *self = analyze(path, source);
            }
            return;
        };
        let mut stale: Vec<std::ops::Range<usize>> = Vec::new();
        for edit in edits {
            for range in &mut stale {
                *range = syntax::map_range(range.clone(), edit);
            }
            stale.extend(self.highlights.apply_edit(edit));
        }
        let Some(mut ranges) = syntax.apply_edits(source, edits) else {
            *self = analyze(path, source);
            return;
        };
        ranges.extend(
            stale
                .into_iter()
                .map(|range| range.start.min(source.len())..range.end.min(source.len())),
        );
        for mut range in syntax::merge_ranges(ranges) {
            // Grow until the range covers whole new captures and whole old
            // spans, so no clipped remainder keeps a stale key.
            loop {
                let (covered, replacement) = syntax.highlight_covering(source, range);
                let grown = self.highlights.covering(covered.clone());
                if grown == covered {
                    self.highlights.splice(covered, replacement);
                    break;
                }
                range = grown;
            }
        }
        self.diagnostics.clear();
        collect_diagnostics(syntax.tree().root_node(), &mut self.diagnostics);
    }
}

/// Parses and highlights `source`, detecting the language from `path`.
/// Unknown languages and grammar failures yield plain-text analysis.
pub fn analyze(path: &Path, source: &str) -> LanguageAnalysis {
    let Some(spec) = registry::detect(path, source) else {
        return LanguageAnalysis::default();
    };
    let mut result = LanguageAnalysis {
        language: Some(spec.name),
        ..Default::default()
    };
    let Some(syntax) = SyntaxTree::parse(spec, source) else {
        return result;
    };
    result.highlights = syntax.highlight(source, 0..source.len());
    collect_diagnostics(syntax.tree().root_node(), &mut result.diagnostics);
    result.syntax = Some(syntax);
    result
}

fn collect_diagnostics(node: Node<'_>, diagnostics: &mut Vec<Diagnostic>) {
    if diagnostics.len() >= MAX_DIAGNOSTICS || !(node.has_error() || node.is_missing()) {
        return;
    }
    let push = |diagnostics: &mut Vec<Diagnostic>, point: tree_sitter::Point, message: String| {
        diagnostics.push(Diagnostic {
            message,
            row: point.row + 1,
            column: point.column + 1,
        });
    };
    if node.is_missing() {
        push(
            diagnostics,
            node.start_position(),
            format!("expected {}", node.kind()),
        );
        return;
    }
    if node.is_error() {
        push(diagnostics, node.start_position(), "syntax error".into());
    }
    let mut cursor = node.walk();
    let mut child_error = false;
    for child in node.children(&mut cursor) {
        child_error |= child.has_error() || child.is_missing();
        collect_diagnostics(child, diagnostics);
    }
    // Some grammars flag an error on a node without exposing an ERROR or
    // MISSING child (e.g. a hidden missing token); report it on the node.
    if !child_error && !node.is_error() {
        let point = if node.child_count() == 0 {
            node.start_position()
        } else {
            node.end_position()
        };
        push(
            diagnostics,
            point,
            format!("syntax error in {}", node.kind()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the attribute key highlighting the first occurrence of `needle`.
    fn key_of(analysis: &LanguageAnalysis, source: &str, needle: &str) -> Option<&'static str> {
        let start = source.find(needle)?;
        analysis
            .highlights
            .spans()
            .iter()
            .find(|span| span.start_byte <= start && start < span.end_byte)
            .map(|span| span.key)
    }

    #[test]
    fn rust_analysis_returns_highlights_without_diagnostics() {
        let source = "/// Docs\npub fn answer() -> u32 { 42 }\n";
        let analysis = analyze(Path::new("lib.rs"), source);
        assert_eq!(analysis.language, Some("Rust"));
        assert!(analysis.diagnostics.is_empty());
        assert_eq!(key_of(&analysis, source, "pub"), Some("DEFAULT_KEYWORD"));
        assert_eq!(key_of(&analysis, source, "42"), Some("DEFAULT_NUMBER"));
        assert_eq!(
            key_of(&analysis, source, "answer"),
            Some("DEFAULT_FUNCTION_DECLARATION")
        );
        assert_eq!(
            key_of(&analysis, source, "/// Docs"),
            Some("DEFAULT_DOC_COMMENT")
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
        let analysis = analyze(Path::new("notes.txt"), "hello");
        assert!(analysis.language.is_none());
        assert!(analysis.highlights.is_empty());
    }

    #[test]
    fn java_tokens_resolve_to_islands_dark_colors() {
        let source = "package com.example;\n\n/** App. */\npublic class App {\n    // greet\n    public String greeting(String name) { return \"Hello, \" + name + 1; }\n    void run() { greeting(\"x\"); }\n}\n";
        let analysis = analyze(Path::new("App.java"), source);
        assert_eq!(analysis.language, Some("Java"));
        let scheme = ColorScheme::islands_dark();
        let color = |needle: &str| {
            key_of(&analysis, source, needle).map(|key| scheme.attributes(key).foreground)
        };
        assert_eq!(color("package"), Some(Rgba::parse("cf8e6d")));
        assert_eq!(color("public class"), Some(Rgba::parse("cf8e6d")));
        assert_eq!(color("\"Hello"), Some(Rgba::parse("6aab73")));
        assert_eq!(color("// greet"), Some(Rgba::parse("7a7e85")));
        assert_eq!(color("/** App"), Some(Rgba::parse("5f826b")));
        assert_eq!(color("greeting(String"), Some(Rgba::parse("56a8f5")));
        assert_eq!(color("1;"), Some(Rgba::parse("2aacb8")));
        // Calls and type references use the plain identifier color, as in IntelliJ.
        assert_eq!(color("greeting(\"x"), Some(Rgba::parse("bcbec4")));
        assert_ne!(key_of(&analysis, source, "String"), Some("DEFAULT_KEYWORD"));
        // Class declarations and type uses are not function declarations.
        assert_eq!(color("App {"), Some(Rgba::parse("bcbec4")));
        assert_eq!(color("String name)"), Some(Rgba::parse("bcbec4")));
        let errors = analyze(Path::new("Broken.java"), "class { void }");
        assert!(!errors.diagnostics.is_empty());
        assert_eq!(
            scheme.attributes("ERRORS_ATTRIBUTES").effect,
            Some((EffectType::WaveUnderline, Rgba::parse("fa6675")))
        );
    }

    #[test]
    fn scene_04_languages_highlight() {
        let cases: &[(&str, &str, &str, &str)] = &[
            (
                "Main.kt",
                "fun main() { val x = \"hi\" }\n",
                "fun",
                "DEFAULT_KEYWORD",
            ),
            (
                "Main.kt",
                "fun main() { val x = \"hi\" }\n",
                "\"hi\"",
                "DEFAULT_STRING",
            ),
            (
                "Main.kt",
                "fun main() { val x = \"hi\" }\n",
                "main",
                "DEFAULT_FUNCTION_DECLARATION",
            ),
            (
                "AndroidManifest.xml",
                "<manifest package=\"a\"/>\n",
                "manifest",
                "XML_TAG_NAME",
            ),
            (
                "AndroidManifest.xml",
                "<manifest package=\"a\"/>\n",
                "\"a\"",
                "DEFAULT_STRING",
            ),
            (
                "BUILD.bazel",
                "# c\njava_binary(name = \"app\", srcs = [\"A.java\"])\n",
                "\"app\"",
                "DEFAULT_STRING",
            ),
            (
                "BUILD.bazel",
                "# c\njava_binary(name = \"app\")\n",
                "# c",
                "DEFAULT_LINE_COMMENT",
            ),
            (
                "defs.bzl",
                "def rule_impl(ctx):\n    return None\n",
                "def",
                "DEFAULT_KEYWORD",
            ),
            (
                "Cargo.toml",
                "[package]\nname = \"x\"\n",
                "\"x\"",
                "DEFAULT_STRING",
            ),
            ("a.json", "{\"k\": 1}", "1", "DEFAULT_NUMBER"),
            ("a.yaml", "k: true\n", "true", "DEFAULT_KEYWORD"),
            ("a.py", "import os\n", "import", "DEFAULT_KEYWORD"),
            (
                "build.gradle",
                "plugins {\n    id 'java'\n}\ndef x = 1\n",
                "'java'",
                "DEFAULT_STRING",
            ),
            (
                "build.gradle",
                "plugins {\n    id 'java'\n}\ndef x = 1\n",
                "def",
                "DEFAULT_KEYWORD",
            ),
            ("a.py", "x = None\n", "None", "DEFAULT_KEYWORD"),
            ("lib.rs", "fn f(x: u32) {}\n", "u32", "DEFAULT_KEYWORD"),
            ("A.java", "class A { int x; }\n", "int", "DEFAULT_KEYWORD"),
            (
                "Main.kt",
                "package a\nfun f(s: String) {}\n",
                "String",
                "DEFAULT_CLASS_REFERENCE",
            ),
            (
                "Main.kt",
                "package a\nfun f(s: String) {}\n",
                "package",
                "DEFAULT_KEYWORD",
            ),
            (
                "Main.kt",
                "class A { override fun g() {} }\n",
                "g()",
                "DEFAULT_FUNCTION_DECLARATION",
            ),
            ("a.go", "package main\n", "package", "DEFAULT_KEYWORD"),
            ("a.ts", "const x: number = 1;\n", "const", "DEFAULT_KEYWORD"),
            (
                "a.cpp",
                "int main() { return 0; }\n",
                "return",
                "DEFAULT_KEYWORD",
            ),
            (
                "a.sh",
                "if true; then echo hi; fi\n",
                "if",
                "DEFAULT_KEYWORD",
            ),
            (
                "a.html",
                "<div class=\"x\"></div>\n",
                "div",
                "HTML_TAG_NAME",
            ),
            (
                "a.css",
                "a { color: red; }\n",
                "color",
                "DEFAULT_INSTANCE_FIELD",
            ),
        ];
        for (path, source, needle, expected) in cases {
            let analysis = analyze(Path::new(path), source);
            assert!(analysis.language.is_some(), "{path} not detected");
            assert_eq!(
                key_of(&analysis, source, needle),
                Some(*expected),
                "{path}: `{needle}` in {:?}",
                analysis.highlights
            );
        }
    }

    #[test]
    fn injected_languages_are_highlighted() {
        let cases: &[(&str, &str, &str, &str)] = &[
            (
                "README.md",
                "# T\n\n```kotlin\nfun main() {}\n```\n",
                "fun",
                "DEFAULT_KEYWORD",
            ),
            (
                "README.md",
                "# T\n\n```java\nclass A {}\n```\n",
                "class",
                "DEFAULT_KEYWORD",
            ),
            (
                "README.md",
                "Use `code` and [link](http://x).\n",
                "`code`",
                "DEFAULT_STRING",
            ),
            (
                "a.html",
                "<script>let x = 1;</script>\n",
                "let",
                "DEFAULT_KEYWORD",
            ),
            (
                "a.html",
                "<style>a { color: red; }</style>\n",
                "color",
                "DEFAULT_INSTANCE_FIELD",
            ),
            (
                "main.rs",
                "fn main() { println!(\"hi {}\", 1); }\n",
                "\"hi",
                "DEFAULT_STRING",
            ),
        ];
        for (path, source, needle, expected) in cases {
            let analysis = analyze(Path::new(path), source);
            assert_eq!(
                key_of(&analysis, source, needle),
                Some(*expected),
                "{path}: `{needle}` in {:?}",
                analysis.highlights
            );
        }
    }

    #[test]
    fn highlights_never_split_or_overlap() {
        for spec in LANGUAGES {
            let path = format!("x.{}", spec.extensions.first().copied().unwrap_or("txt"));
            let source = "fn x() { \"ä\" } // ü\n<a b=\"c\">ä</a> # 1.5e3\n";
            let analysis = analyze(Path::new(&path), source);
            let mut last = 0;
            for span in analysis.highlights.spans() {
                assert!(
                    span.start_byte >= last && span.start_byte < span.end_byte,
                    "{path}"
                );
                assert!(
                    source.is_char_boundary(span.start_byte)
                        && source.is_char_boundary(span.end_byte)
                );
                last = span.end_byte;
            }
        }
    }
}
