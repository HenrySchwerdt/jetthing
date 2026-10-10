//! Live templates and postfix templates (PLATFORM-EDIT-125/126/128).
//!
//! Templates are data: bundled JSON sets per language (`templates/*.json`) with
//! `$NAME$` variables, `$END$` (final caret) and `$SELECTION$` (surround
//! templates). Expanding a template starts a [`TemplateSession`] that edits its
//! variables in place: occurrences of a variable mirror each other, Tab and
//! Shift+Tab move between variables, Enter/Esc finish.

use serde::Deserialize;
use std::path::Path;
use std::sync::OnceLock;

mod session;

pub use session::{SessionState, TemplateSession};

/// A variable's definition inside a template.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VariableDef {
    pub name: String,
    /// Expression computing the initial value, e.g. `className()`,
    /// `suggestIndexName()`, `"literal"`.
    pub expression: Option<String>,
    /// Value used when the expression yields nothing.
    pub default: Option<String>,
    /// Do not stop on this variable when the expression produced a value.
    pub skip_if_defined: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Template {
    pub abbreviation: String,
    pub description: String,
    /// Body with `$VAR$`, `$END$`, `$SELECTION$`; `$$` is a literal `$`.
    /// Nested lines are written with 4-space indentation steps.
    pub text: String,
    pub variables: Vec<VariableDef>,
}

impl Template {
    /// Surround templates wrap the selection (`$SELECTION$`).
    pub fn is_surround(&self) -> bool {
        self.text.contains("$SELECTION$")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TemplateSet {
    /// Group name shown in settings, e.g. `Java`.
    pub group: String,
    /// File extensions the set applies to; empty means every file.
    pub extensions: Vec<String>,
    pub templates: Vec<Template>,
    /// Postfix templates: `abbreviation` is the key after the dot and `$EXPR$`
    /// is the expression before it.
    pub postfix: Vec<Template>,
}

impl TemplateSet {
    fn applies_to(&self, path: &Path) -> bool {
        if self.extensions.is_empty() {
            return true;
        }
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase);
        extension.is_some_and(|ext| self.extensions.iter().any(|e| *e == ext))
    }
}

const BUNDLED: [(&str, &str); 4] = [
    ("java.json", include_str!("../templates/java.json")),
    ("kotlin.json", include_str!("../templates/kotlin.json")),
    ("rust.json", include_str!("../templates/rust.json")),
    ("other.json", include_str!("../templates/other.json")),
];

#[derive(Debug, Clone, Default)]
pub struct TemplateRegistry {
    pub sets: Vec<TemplateSet>,
    /// Bundled or user sets that failed to parse, with the reason.
    pub errors: Vec<String>,
}

impl TemplateRegistry {
    /// Parses sets; malformed sets are reported in `errors` and skipped.
    pub fn from_sources<'a>(sources: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut registry = Self::default();
        for (name, json) in sources {
            match serde_json::from_str::<TemplateSet>(json) {
                Ok(set) => registry.sets.push(set),
                Err(error) => registry.errors.push(format!("{name}: {error}")),
            }
        }
        registry
    }

    /// The bundled Java, Kotlin, Rust and language-independent sets.
    pub fn bundled() -> &'static Self {
        static REGISTRY: OnceLock<TemplateRegistry> = OnceLock::new();
        REGISTRY.get_or_init(|| Self::from_sources(BUNDLED))
    }

    /// Templates applicable to `path`, language sets first.
    pub fn templates_for<'a>(&'a self, path: &'a Path) -> impl Iterator<Item = &'a Template> + 'a {
        let (specific, general): (Vec<&TemplateSet>, Vec<&TemplateSet>) = self
            .sets
            .iter()
            .filter(|set| set.applies_to(path))
            .partition(|set| !set.extensions.is_empty());
        specific
            .into_iter()
            .chain(general)
            .flat_map(|set| set.templates.iter())
    }

    pub fn find(&self, path: &Path, abbreviation: &str) -> Option<&Template> {
        self.templates_for(path)
            .find(|template| template.abbreviation == abbreviation)
    }

    pub fn find_postfix(&self, path: &Path, key: &str) -> Option<&Template> {
        self.sets
            .iter()
            .filter(|set| set.applies_to(path))
            .flat_map(|set| set.postfix.iter())
            .find(|template| template.abbreviation == key)
    }
}

/// Inputs for computing variable values and indentation.
#[derive(Debug, Clone, Default)]
pub struct ExpansionContext {
    /// File name, e.g. `Main.java`.
    pub file_name: String,
    /// Indentation of the line the template is inserted on.
    pub line_indent: String,
    /// One indentation level (`"    "` or `"\t"`).
    pub indent_unit: String,
    /// Selected text for surround templates.
    pub selection: Option<String>,
    /// Expression for postfix templates (`$EXPR$`).
    pub expression: Option<String>,
    /// Nearby text, used to pick unused names (`suggestIndexName()`).
    pub surrounding: String,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Evaluates a variable expression; `None` when it yields nothing.
pub fn evaluate(expression: &str, context: &ExpansionContext) -> Option<String> {
    let expression = expression.trim();
    if let Some(literal) = expression
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return Some(literal.to_owned());
    }
    let stem = || {
        Path::new(&context.file_name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(str::to_owned)
    };
    match expression {
        "className()" | "fileNameWithoutExtension()" => stem(),
        "fileName()" => Some(context.file_name.clone()),
        "expression()" => context.expression.clone(),
        "suggestIndexName()" => {
            let used = |name: &str| {
                context
                    .surrounding
                    .split(|c: char| !is_word(c))
                    .any(|word| word == name)
            };
            ["i", "j", "k", "l", "m", "n"]
                .into_iter()
                .find(|name| !used(name))
                .map(str::to_owned)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Segment {
    Text(String),
    Var(usize),
    End,
}

/// A template with resolved text segments and initial variable values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    pub(crate) segments: Vec<Segment>,
    pub(crate) names: Vec<String>,
    pub(crate) values: Vec<String>,
    /// Variables visited by Tab, in order of first appearance.
    pub(crate) stops: Vec<usize>,
}

impl Instance {
    pub fn text(&self) -> String {
        self.segments
            .iter()
            .map(|segment| match segment {
                Segment::Text(text) => text.as_str(),
                Segment::Var(index) => self.values[*index].as_str(),
                Segment::End => "",
            })
            .collect()
    }

    pub fn variable_names(&self) -> &[String] {
        &self.names
    }
}

/// Reindents template line starts: each leading 4-space step becomes one
/// indent unit, and every line after the first gets the insertion line's indent.
fn indent_text(text: &str, at_line_start: bool, context: &ExpansionContext) -> String {
    let mut out = String::new();
    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        if index > 0 || at_line_start {
            let spaces = line.len() - line.trim_start_matches(' ').len();
            let (steps, rest) = (spaces / 4, &line[spaces - spaces % 4..]);
            if index > 0 && !line.is_empty() {
                out.push_str(&context.line_indent);
            }
            for _ in 0..steps {
                out.push_str(&context.indent_unit);
            }
            out.push_str(rest);
        } else {
            out.push_str(line);
        }
    }
    out
}

/// Selection text re-indented to sit at `indent` inside the template.
fn reindent_selection(selection: &str, indent: &str) -> String {
    let lines: Vec<&str> = selection.trim_end_matches('\n').split('\n').collect();
    let common = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.len() - line.trim_start_matches([' ', '\t']).len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let body = line.get(common..).unwrap_or(line.trim_start());
            if index == 0 || body.is_empty() {
                body.to_owned()
            } else {
                format!("{indent}{body}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parses `template` into segments and computes initial variable values.
pub fn instantiate(template: &Template, context: &ExpansionContext) -> Instance {
    let mut segments = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut literal = String::new();
    let mut rest = template.text.as_str();
    let mut at_line_start = false;
    let flush = |literal: &mut String, segments: &mut Vec<Segment>, at_line_start: &mut bool| {
        if !literal.is_empty() {
            let text = indent_text(literal, *at_line_start, context);
            *at_line_start = literal.ends_with('\n');
            segments.push(Segment::Text(text));
            literal.clear();
        }
    };
    while let Some(dollar) = rest.find('$') {
        literal.push_str(&rest[..dollar]);
        let after = &rest[dollar + 1..];
        if let Some(stripped) = after.strip_prefix('$') {
            literal.push('$');
            rest = stripped;
            continue;
        }
        let name_len = after
            .find(|c: char| !is_word(c))
            .filter(|&end| end > 0 && after[end..].starts_with('$'));
        let Some(name_len) = name_len else {
            // A lone `$` is literal text.
            literal.push('$');
            rest = after;
            continue;
        };
        let name = &after[..name_len];
        rest = &after[name_len + 1..];
        // Indentation of the current template line, for `$SELECTION$`.
        let line_prefix = literal.rsplit('\n').next().unwrap_or("").to_owned();
        let on_new_line = literal.contains('\n') || at_line_start;
        flush(&mut literal, &mut segments, &mut at_line_start);
        match name {
            "END" => segments.push(Segment::End),
            "SELECTION" => {
                let steps = line_prefix.len() / 4;
                let mut indent = if on_new_line {
                    context.line_indent.clone()
                } else {
                    String::new()
                };
                for _ in 0..steps {
                    indent.push_str(&context.indent_unit);
                }
                let text = reindent_selection(context.selection.as_deref().unwrap_or(""), &indent);
                segments.push(Segment::Text(text));
            }
            _ => {
                let index = names.iter().position(|n| n == name).unwrap_or_else(|| {
                    names.push(name.to_owned());
                    names.len() - 1
                });
                segments.push(Segment::Var(index));
            }
        }
    }
    literal.push_str(rest);
    flush(&mut literal, &mut segments, &mut at_line_start);

    let mut values = Vec::with_capacity(names.len());
    let mut stops = Vec::new();
    for (index, name) in names.iter().enumerate() {
        let definition = template.variables.iter().find(|v| &v.name == name);
        let computed = if name == "EXPR" {
            context.expression.clone()
        } else {
            definition
                .and_then(|d| d.expression.as_deref())
                .and_then(|expression| evaluate(expression, context))
        };
        let skip = name == "EXPR"
            || (computed.is_some() && definition.is_some_and(|d| d.skip_if_defined));
        let value = computed
            .or_else(|| definition.and_then(|d| d.default.clone()))
            .unwrap_or_default();
        values.push(value);
        if !skip {
            stops.push(index);
        }
    }
    Instance {
        segments,
        names,
        values,
        stops,
    }
}

/// Characters allowed in abbreviations (`else-if`, `psvm`).
fn is_abbreviation_char(c: char) -> bool {
    is_word(c) || c == '-'
}

/// The abbreviation ending at `offset` and its start, if any.
pub fn abbreviation_before(doc: &jetthing_text::TextDocument, offset: usize) -> Option<(usize, String)> {
    let mut start = offset;
    while start > 0 && doc.char_at(start - 1).is_some_and(is_abbreviation_char) {
        start -= 1;
    }
    // `a.sout` is postfix syntax, not the `sout` live template.
    let after_dot = start > 0 && doc.char_at(start - 1) == Some('.');
    (start < offset && !after_dot)
        .then(|| doc.slice(start, offset).ok())
        .flatten()
        .map(|text| (start, text))
}

/// Start of the expression that ends at `dot` (the `.` of `expr.key`). Scans
/// back over identifiers, member access, calls/indexing with balanced brackets
/// and string literals, stopping at whitespace or operators at depth zero.
pub fn expression_before(doc: &jetthing_text::TextDocument, dot: usize) -> Option<usize> {
    const LIMIT: usize = 2_000;
    let mut index = dot;
    let mut depth = 0usize;
    let mut steps = 0;
    while index > 0 && steps < LIMIT {
        steps += 1;
        let c = doc.char_at(index - 1)?;
        match c {
            ')' | ']' => depth += 1,
            '(' | '[' if depth > 0 => depth -= 1,
            '(' | '[' => break,
            '"' | '\'' => {
                // Skip a string literal backwards.
                let quote = c;
                index -= 1;
                while index > 0 && doc.char_at(index - 1) != Some(quote) {
                    index -= 1;
                }
                if index == 0 {
                    return None;
                }
            }
            _ if depth > 0 => {}
            c if is_word(c) || c == '.' || c == ':' || c == '!' && index == dot => {}
            _ => break,
        }
        index -= 1;
    }
    (index < dot && depth == 0).then_some(index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jetthing_text::TextDocument;

    fn context() -> ExpansionContext {
        ExpansionContext {
            file_name: "Main.java".into(),
            line_indent: "    ".into(),
            indent_unit: "    ".into(),
            ..Default::default()
        }
    }

    #[test]
    fn bundled_sets_parse_and_have_unique_abbreviations() {
        let registry = TemplateRegistry::bundled();
        assert!(registry.errors.is_empty(), "{:?}", registry.errors);
        assert_eq!(registry.sets.len(), BUNDLED.len());
        for set in &registry.sets {
            let mut seen = std::collections::HashSet::new();
            for template in set.templates.iter().chain(&set.postfix) {
                assert!(
                    seen.insert((template.is_surround(), set.postfix.contains(template), &template.abbreviation)),
                    "{}: duplicate {}",
                    set.group,
                    template.abbreviation
                );
                assert!(!template.description.is_empty());
            }
        }
        let java = Path::new("A.java");
        for abbreviation in ["psvm", "main", "sout", "soutv", "fori", "iter", "ifn", "inn", "thr"] {
            assert!(registry.find(java, abbreviation).is_some(), "{abbreviation}");
        }
        assert!(registry.find(Path::new("a.kt"), "fun0").is_some());
        assert!(registry.find(Path::new("a.rs"), "tfn").is_some());
        assert!(registry.find(java, "todo").is_some(), "general set applies everywhere");
        assert!(registry.find(Path::new("a.rs"), "psvm").is_none());
        assert!(registry.find_postfix(java, "if").is_some());
    }

    #[test]
    fn malformed_sets_are_reported_not_fatal() {
        let registry = TemplateRegistry::from_sources([("bad.json", "{"), ("ok.json", "{}")]);
        assert_eq!(registry.sets.len(), 1);
        assert!(registry.errors[0].starts_with("bad.json"));
    }

    #[test]
    fn instantiates_variables_end_and_indentation() {
        let template = Template {
            abbreviation: "fori".into(),
            text: "for (int $INDEX$ = 0; $INDEX$ < $LIMIT$; $INDEX$++) {\n    $END$\n}".into(),
            variables: vec![VariableDef {
                name: "INDEX".into(),
                expression: Some("suggestIndexName()".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut ctx = context();
        ctx.surrounding = "int i = 0;".into();
        let instance = instantiate(&template, &ctx);
        assert_eq!(
            instance.text(),
            "for (int j = 0; j < ; j++) {\n        \n    }"
        );
        assert_eq!(instance.variable_names(), ["INDEX", "LIMIT"]);
        assert_eq!(instance.stops, [0, 1]);
    }

    #[test]
    fn evaluates_expressions_and_skips_defined_variables() {
        let ctx = context();
        assert_eq!(evaluate("className()", &ctx).as_deref(), Some("Main"));
        assert_eq!(evaluate("\"x\"", &ctx).as_deref(), Some("x"));
        assert_eq!(evaluate("suggestIndexName()", &ctx).as_deref(), Some("i"));
        assert_eq!(evaluate("unknown()", &ctx), None);
        let template = Template {
            text: "class $NAME$ $BODY$".into(),
            variables: vec![
                VariableDef {
                    name: "NAME".into(),
                    expression: Some("className()".into()),
                    skip_if_defined: true,
                    ..Default::default()
                },
                VariableDef {
                    name: "BODY".into(),
                    default: Some("{}".into()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        let instance = instantiate(&template, &ctx);
        assert_eq!(instance.text(), "class Main {}");
        assert_eq!(instance.stops, [1]);
    }

    #[test]
    fn surround_reindents_the_selection() {
        let template = Template {
            text: "if ($COND$) {\n    $SELECTION$\n}".into(),
            ..Default::default()
        };
        let mut ctx = context();
        ctx.selection = Some("    a();\n        b();\n".into());
        assert_eq!(
            instantiate(&template, &ctx).text(),
            "if () {\n        a();\n            b();\n    }"
        );
    }

    #[test]
    fn literal_dollars_and_unclosed_variables_stay_text() {
        let template = Template {
            text: "$$x $5 $A".into(),
            ..Default::default()
        };
        assert_eq!(instantiate(&template, &context()).text(), "$x $5 $A");
    }

    #[test]
    fn finds_abbreviations_and_postfix_expressions() {
        let doc = TextDocument::new("  sout x.sout foo(a, b)[0].size().if \"s\".len");
        assert_eq!(abbreviation_before(&doc, 6), Some((2, "sout".into())));
        assert_eq!(abbreviation_before(&doc, 13), None, "postfix, not a template");
        assert_eq!(abbreviation_before(&doc, 2), None);
        let dot = 33;
        assert_eq!(doc.char_at(dot), Some('.'));
        let start = expression_before(&doc, dot).unwrap();
        assert_eq!(doc.slice(start, dot).unwrap(), "foo(a, b)[0].size()");
        let dot = 40;
        assert_eq!(doc.char_at(dot), Some('.'));
        assert_eq!(doc.slice(expression_before(&doc, dot).unwrap(), dot).unwrap(), "\"s\"");
        assert_eq!(expression_before(&TextDocument::new(" .x"), 1), None);
    }
}
