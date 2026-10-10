//! Flat symbol outline of a source file for Go to Class / Go to Symbol.
//!
//! Built from the grammars' tags queries (`@definition.<kind>` with a `@name`
//! capture), so it works for every language that ships one, without a language
//! server.

use crate::registry::{self, LanguageSpec};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

/// What a symbol declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Class,
    Interface,
    Enum,
    Function,
    Method,
    Field,
    Constant,
    Module,
    Type,
    Macro,
    Other,
}

impl SymbolKind {
    fn from_capture(kind: &str) -> Self {
        match kind {
            "class" | "struct" | "object" | "record" => Self::Class,
            "interface" | "trait" | "protocol" => Self::Interface,
            "enum" => Self::Enum,
            "function" => Self::Function,
            "method" | "constructor" => Self::Method,
            "field" | "property" => Self::Field,
            "constant" | "variable" => Self::Constant,
            "module" | "namespace" | "package" => Self::Module,
            "type" => Self::Type,
            "macro" => Self::Macro,
            _ => Self::Other,
        }
    }

    /// Declarations Go to Class lists: classes, interfaces, enums and type aliases.
    pub fn is_class_like(self) -> bool {
        matches!(
            self,
            Self::Class | Self::Interface | Self::Enum | Self::Type
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Class => "class",
            Self::Interface => "interface",
            Self::Enum => "enum",
            Self::Function => "function",
            Self::Method => "method",
            Self::Field => "field",
            Self::Constant => "constant",
            Self::Module => "module",
            Self::Type => "type",
            Self::Macro => "macro",
            Self::Other => "symbol",
        }
    }
}

/// One declared name and where it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineSymbol {
    pub name: String,
    pub kind: SymbolKind,
    /// Zero-based line of the name.
    pub line: usize,
}

fn tags_query(spec: &'static LanguageSpec) -> Option<&'static Query> {
    static CACHE: OnceLock<Mutex<HashMap<&'static str, Option<&'static Query>>>> = OnceLock::new();
    let mut cache = CACHE.get_or_init(Default::default).lock().ok()?;
    *cache.entry(spec.id).or_insert_with(|| {
        let source = spec.tags?;
        let query = Query::new(&spec.grammar(), source).ok()?;
        Some(&*Box::leak(Box::new(query)))
    })
}

/// Declarations in `source`, in file order. Empty when the language has no
/// tags query or the file does not parse.
pub fn outline(path: &Path, source: &str) -> Vec<OutlineSymbol> {
    let Some(spec) = registry::detect(path, source) else {
        return Vec::new();
    };
    let Some(query) = tags_query(spec) else {
        return Vec::new();
    };
    let mut parser = Parser::new();
    if parser.set_language(&spec.grammar()).is_err() {
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        return Vec::new();
    };
    let names = query.capture_names();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());
    let mut out = Vec::new();
    while let Some(found) = matches.next() {
        let mut name = None;
        let mut kind = None;
        for capture in found.captures {
            let capture_name = names[capture.index as usize];
            if capture_name == "name" {
                name = Some(capture.node);
            } else if let Some(kind_name) = capture_name.strip_prefix("definition.") {
                kind = Some(SymbolKind::from_capture(kind_name));
            }
        }
        let (Some(name), Some(kind)) = (name, kind) else {
            continue;
        };
        let Ok(text) = name.utf8_text(source.as_bytes()) else {
            continue;
        };
        out.push(OutlineSymbol {
            name: text.to_owned(),
            kind,
            line: name.start_position().row,
        });
    }
    out.sort_by_key(|symbol| symbol.line);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_outline_lists_types_and_functions() {
        let source = "struct Greeter;\nenum Mode { A }\nfn greet() {}\nimpl Greeter { fn hello(&self) {} }\n";
        let symbols = outline(Path::new("a.rs"), source);
        let names: Vec<_> = symbols.iter().map(|s| (s.name.as_str(), s.kind)).collect();
        assert!(names.contains(&("Greeter", SymbolKind::Class)));
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "Mode" && s.kind.is_class_like())
        );
        assert!(names.contains(&("greet", SymbolKind::Function)));
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "hello" && s.line == 3 && !s.kind.is_class_like())
        );
    }

    #[test]
    fn java_outline_finds_classes_and_methods() {
        let source = "public class MainActivity {\n  void onCreate() {}\n}\n";
        let symbols = outline(Path::new("MainActivity.java"), source);
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "MainActivity" && s.kind == SymbolKind::Class)
        );
        assert!(symbols.iter().any(|s| s.name == "onCreate" && s.line == 1));
    }

    #[test]
    fn unknown_languages_have_no_outline() {
        assert!(outline(Path::new("notes.txt"), "hello").is_empty());
    }
}
