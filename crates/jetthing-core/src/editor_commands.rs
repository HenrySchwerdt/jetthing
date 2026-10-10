//! Editor actions exposed as typed commands (menus, Find Action, keymap, CLI).

use crate::{IdeState, spec, string_param, usize_param};
use jetthing_command::{Command, CommandError, CommandRegistry};
use jetthing_text::ops::{self, FindOptions};
use jetthing_text::{Selection, TextError};
use serde_json::{Value, json};
use std::path::Path;

/// Line comment prefix for a file, by extension.
pub fn line_comment_prefix(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("py" | "sh" | "bash" | "zsh" | "toml" | "yaml" | "yml" | "rb" | "pl" | "r") => "#",
        Some("sql" | "lua" | "hs") => "--",
        Some("lisp" | "clj" | "el") => ";",
        Some("bat" | "cmd") => "REM",
        _ => "//",
    }
}

fn text_error(error: TextError) -> String {
    error.to_string()
}

fn bool_param(params: &Value, name: &str) -> bool {
    params.get(name).and_then(Value::as_bool).unwrap_or(false)
}

fn find_options(params: &Value) -> FindOptions {
    FindOptions {
        match_case: bool_param(params, "matchCase"),
        whole_words: bool_param(params, "wholeWords"),
    }
}

/// Registers a parameterless action that edits the active document.
fn action(
    registry: &mut CommandRegistry<IdeState>,
    id: &str,
    label: &str,
    shortcut: Option<&str>,
    run: fn(&mut crate::Document) -> Result<(), TextError>,
) {
    registry
        .register(Command::new(
            spec(id, label, "Editor", shortcut, "{}"),
            |state: &IdeState| state.active_document.is_some(),
            move |state: &mut IdeState, _| {
                state
                    .edit_active(|document| run(document).map_err(text_error))
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
}

/// Selects the next (or previous) match after the primary selection, wrapping around.
fn select_match(
    document: &mut crate::Document,
    query: &str,
    options: FindOptions,
    backwards: bool,
) -> Value {
    let text = document.text.text();
    let matches = ops::find_all(&text, query, options);
    let primary = document.selections.primary();
    let chosen = if backwards {
        matches
            .iter()
            .rev()
            .find(|(start, _)| *start < primary.start())
            .or(matches.last())
    } else {
        matches
            .iter()
            .find(|(start, _)| {
                *start > primary.start() || (*start == primary.start() && primary.is_caret())
            })
            .or(matches.first())
    };
    let Some(&(start, end)) = chosen else {
        return json!({"matches": 0, "index": null});
    };
    document.selections.set_single(Selection {
        anchor: start,
        head: end,
    });
    let index = matches.iter().position(|m| *m == (start, end));
    json!({"matches": matches.len(), "index": index})
}

/// Extend Selection: grows every selection to the next syntactic unit.
/// Returns false when nothing could grow (plain text or whole file selected).
pub fn extend_selection(document: &mut crate::Document) -> bool {
    let source = document.text.text();
    let before = document.selections.selections().to_vec();
    let next: Vec<Selection> = before
        .iter()
        .map(|selection| {
            let bytes = document.text.char_to_byte(selection.start())
                ..document.text.char_to_byte(selection.end());
            match document.analysis.expand_selection(&source, bytes) {
                Some(range) => Selection {
                    anchor: document.text.byte_to_char(range.start),
                    head: document.text.byte_to_char(range.end),
                },
                None => *selection,
            }
        })
        .collect();
    if next == before {
        return false;
    }
    // History only continues while the selection is the last extended one.
    let continues = document.selection_history.last().is_some_and(|previous| {
        previous.len() == before.len()
            && previous
                .iter()
                .zip(&before)
                .all(|(inner, outer)| outer.start() <= inner.start() && inner.end() <= outer.end())
    });
    if !continues {
        document.selection_history.clear();
    }
    document.selection_history.push(before);
    document.selections.set_all(next);
    true
}

/// Shrink Selection: steps back through Extend Selection history, as long as
/// each remembered selection still lies inside the current one.
pub fn shrink_selection(document: &mut crate::Document) -> bool {
    let current = document.selections.selections().to_vec();
    let Some(previous) = document.selection_history.pop() else {
        return false;
    };
    let nested = previous.len() == current.len()
        && previous
            .iter()
            .zip(&current)
            .all(|(inner, outer)| outer.start() <= inner.start() && inner.end() <= outer.end());
    if !nested {
        document.selection_history.clear();
        return false;
    }
    document.selections.set_all(previous);
    true
}

fn selection_command(
    registry: &mut CommandRegistry<IdeState>,
    id: &str,
    label: &str,
    shortcut: &str,
    run: fn(&mut crate::Document) -> bool,
) {
    registry
        .register(Command::new(
            spec(id, label, "Editor", Some(shortcut), "{}"),
            |state: &IdeState| state.active_document.is_some(),
            move |state: &mut IdeState, _| {
                let document = state
                    .active_document
                    .and_then(|index| state.documents.get_mut(index))
                    .ok_or_else(|| CommandError::Disabled("no active document".into()))?;
                Ok(json!({"changed": run(document)}))
            },
        ))
        .expect("unique built-in id");
}

pub(crate) fn register(registry: &mut CommandRegistry<IdeState>) {
    selection_command(
        registry,
        "editor.extendSelection",
        "Extend Selection",
        "⌥↑",
        extend_selection,
    );
    selection_command(
        registry,
        "editor.shrinkSelection",
        "Shrink Selection",
        "⌥↓",
        shrink_selection,
    );
    action(
        registry,
        "editor.duplicate",
        "Duplicate Line or Selection",
        Some("⌘D"),
        |d| ops::duplicate(&mut d.text, &mut d.selections),
    );
    action(
        registry,
        "editor.deleteLine",
        "Delete Line",
        Some("⌘⌫"),
        |d| ops::delete_lines(&mut d.text, &mut d.selections),
    );
    action(
        registry,
        "editor.moveLineUp",
        "Move Line Up",
        Some("⌥⇧↑"),
        |d| ops::move_lines(&mut d.text, &mut d.selections, true),
    );
    action(
        registry,
        "editor.moveLineDown",
        "Move Line Down",
        Some("⌥⇧↓"),
        |d| ops::move_lines(&mut d.text, &mut d.selections, false),
    );
    action(registry, "editor.indent", "Indent Selection", None, |d| {
        ops::indent_lines(&mut d.text, &mut d.selections)
    });
    action(
        registry,
        "editor.unindent",
        "Unindent Line or Selection",
        Some("⇧⇥"),
        |d| ops::unindent_lines(&mut d.text, &mut d.selections),
    );
    action(
        registry,
        "editor.commentLine",
        "Comment with Line Comment",
        Some("⌘/"),
        |d| {
            let prefix = line_comment_prefix(&d.path);
            ops::toggle_line_comment(&mut d.text, &mut d.selections, prefix)
        },
    );
    action(
        registry,
        "editor.selectAll",
        "Select All",
        Some("⌘A"),
        |d| {
            let len = d.text.len_chars();
            d.selections.set_single(Selection {
                anchor: 0,
                head: len,
            });
            Ok(())
        },
    );

    registry
        .register(Command::new(
            spec("editor.copy", "Copy", "Editor", Some("⌘C"), "{}"),
            |state: &IdeState| state.active_document.is_some(),
            |state: &mut IdeState, _| {
                let (text, whole_line) = state
                    .edit_active(|d| Ok(ops::copy_text(&d.text, &d.selections)))
                    .map_err(CommandError::Failed)?;
                Ok(json!({"text": text, "wholeLine": whole_line}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec("editor.cut", "Cut", "Editor", Some("⌘X"), "{}"),
            |state: &IdeState| state.active_document.is_some(),
            |state: &mut IdeState, _| {
                let (text, whole_line) = state
                    .edit_active(|d| {
                        let whole_line = d.selections.selections().iter().all(|s| s.is_caret());
                        let text = ops::cut(&mut d.text, &mut d.selections).map_err(text_error)?;
                        Ok((text, whole_line))
                    })
                    .map_err(CommandError::Failed)?;
                Ok(json!({"text": text, "wholeLine": whole_line}))
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "editor.paste",
                "Paste",
                "Editor",
                Some("⌘V"),
                r#"{"text":"string","wholeLine":"bool?"}"#,
            ),
            |state: &IdeState| state.active_document.is_some(),
            |state: &mut IdeState, params| {
                let text = string_param(&params, "text")?;
                let whole_line = bool_param(&params, "wholeLine");
                state
                    .edit_active(|d| {
                        ops::paste(&mut d.text, &mut d.selections, &text, whole_line)
                            .map_err(text_error)
                    })
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "editor.gotoLine",
                "Go to Line:Column",
                "Navigate",
                Some("⌘L"),
                r#"{"line":1,"column":"number?"}"#,
            ),
            |state: &IdeState| state.active_document.is_some(),
            |state: &mut IdeState, params| {
                let line = usize_param(&params, "line")?.max(1) - 1;
                let column = params
                    .get("column")
                    .and_then(Value::as_u64)
                    .map(|c| (c as usize).max(1) - 1)
                    .unwrap_or(0);
                state
                    .edit_active(|d| {
                        let line = line.min(d.text.len_lines().saturating_sub(1));
                        let (start, end, _) = ops::line_bounds(&d.text, line);
                        d.selections
                            .set_single(Selection::caret((start + column).min(end)));
                        Ok(())
                    })
                    .map_err(CommandError::Failed)?;
                Ok(Value::Null)
            },
        ))
        .expect("unique built-in id");
    for (id, label, shortcut, backwards) in [
        ("editor.findNext", "Find Next Occurrence", "⌘G", false),
        (
            "editor.findPrevious",
            "Find Previous Occurrence",
            "⇧⌘G",
            true,
        ),
    ] {
        registry
            .register(Command::new(
                spec(
                    id,
                    label,
                    "Editor",
                    Some(shortcut),
                    r#"{"query":"string","matchCase":"bool?","wholeWords":"bool?"}"#,
                ),
                |state: &IdeState| state.active_document.is_some(),
                move |state: &mut IdeState, params| {
                    let query = string_param(&params, "query")?;
                    let options = find_options(&params);
                    state
                        .edit_active(|d| Ok(select_match(d, &query, options, backwards)))
                        .map_err(CommandError::Failed)
                },
            ))
            .expect("unique built-in id");
    }
    registry
        .register(Command::new(
            spec(
                "editor.replace",
                "Replace Occurrence",
                "Editor",
                None,
                r#"{"query":"string","replacement":"string","matchCase":"bool?","wholeWords":"bool?"}"#,
            ),
            |state: &IdeState| state.active_document.is_some(),
            |state: &mut IdeState, params| {
                let query = string_param(&params, "query")?;
                let replacement = string_param(&params, "replacement")?;
                let options = find_options(&params);
                state
                    .edit_active(|d| {
                        let primary = d.selections.primary();
                        let selected = d
                            .text
                            .slice(primary.start(), primary.end())
                            .map_err(text_error)?;
                        let is_match = ops::find_all(&selected, &query, options)
                            .first()
                            .is_some_and(|&(s, e)| s == 0 && e == selected.chars().count());
                        if is_match {
                            ops::replace_ranges(
                                &mut d.text,
                                &[(primary.start(), primary.end())],
                                &replacement,
                            )
                            .map_err(text_error)?;
                            let caret = primary.start() + replacement.chars().count();
                            d.selections.set_single(Selection::caret(caret));
                        }
                        Ok(select_match(d, &query, options, false))
                    })
                    .map_err(CommandError::Failed)
            },
        ))
        .expect("unique built-in id");
    registry
        .register(Command::new(
            spec(
                "editor.replaceAll",
                "Replace All",
                "Editor",
                None,
                r#"{"query":"string","replacement":"string","matchCase":"bool?","wholeWords":"bool?"}"#,
            ),
            |state: &IdeState| state.active_document.is_some(),
            |state: &mut IdeState, params| {
                let query = string_param(&params, "query")?;
                let replacement = string_param(&params, "replacement")?;
                let options = find_options(&params);
                let count = state
                    .edit_active(|d| {
                        let ranges = ops::find_all(&d.text.text(), &query, options);
                        ops::replace_ranges(&mut d.text, &ranges, &replacement)
                            .map_err(text_error)?;
                        let len = d.text.len_chars();
                        d.selections.clamp(len);
                        Ok(ranges.len())
                    })
                    .map_err(CommandError::Failed)?;
                state.status = format!("{count} occurrences replaced");
                Ok(json!({"replaced": count}))
            },
        ))
        .expect("unique built-in id");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in_commands;

    fn state_with(
        text: &str,
        name: &str,
    ) -> (IdeState, CommandRegistry<IdeState>, std::path::PathBuf) {
        let root =
            std::env::temp_dir().join(format!("jetthing-editor-cmd-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(name), text).unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("editor.openFile", &mut state, json!({"path": name}))
            .unwrap();
        (state, commands, root)
    }

    #[test]
    fn extend_and_shrink_selection_follow_the_syntax_tree() {
        let source = "class A {\n    int f() { return 1 + 42; }\n}\n";
        let (mut state, commands, root) = state_with(source, "A.java");
        let caret = source.find("42").unwrap();
        state.documents[0]
            .selections
            .set_single(Selection::caret(caret));
        let selected = |state: &IdeState| {
            let selection = state.documents[0].selections.primary();
            state.documents[0]
                .text
                .slice(selection.start(), selection.end())
                .unwrap()
        };
        let extend = |state: &mut IdeState| {
            commands
                .execute("editor.extendSelection", state, Value::Null)
                .unwrap()
        };
        assert_eq!(extend(&mut state)["changed"], true);
        assert_eq!(selected(&state), "42");
        extend(&mut state);
        assert_eq!(selected(&state), "1 + 42");
        extend(&mut state);
        assert_eq!(selected(&state), "return 1 + 42;");
        commands
            .execute("editor.shrinkSelection", &mut state, Value::Null)
            .unwrap();
        assert_eq!(selected(&state), "1 + 42");
        commands
            .execute("editor.shrinkSelection", &mut state, Value::Null)
            .unwrap();
        assert_eq!(selected(&state), "42");
        // Moving the caret elsewhere invalidates the history.
        state.documents[0]
            .selections
            .set_single(Selection::caret(0));
        let shrunk = commands
            .execute("editor.shrinkSelection", &mut state, Value::Null)
            .unwrap();
        assert_eq!(shrunk["changed"], false);
        for _ in 0..20 {
            extend(&mut state);
        }
        assert_eq!(selected(&state), source);
        assert_eq!(extend(&mut state)["changed"], false);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn comment_prefix_depends_on_language() {
        assert_eq!(line_comment_prefix(Path::new("a.rs")), "//");
        assert_eq!(line_comment_prefix(Path::new("Cargo.toml")), "#");
        assert_eq!(line_comment_prefix(Path::new("q.SQL")), "--");
    }

    #[test]
    fn line_actions_run_through_the_registry() {
        let (mut state, commands, root) = state_with("a = 1\nb = 2\n", "x.py");
        commands
            .execute("editor.commentLine", &mut state, Value::Null)
            .unwrap();
        assert_eq!(state.documents[0].text.text(), "# a = 1\nb = 2\n");
        commands
            .execute("editor.duplicate", &mut state, Value::Null)
            .unwrap();
        assert_eq!(state.documents[0].text.text(), "# a = 1\nb = 2\nb = 2\n");
        commands
            .execute("editor.deleteLine", &mut state, Value::Null)
            .unwrap();
        assert_eq!(state.documents[0].text.text(), "# a = 1\nb = 2\n");
        commands
            .execute("editor.gotoLine", &mut state, json!({"line": 2}))
            .unwrap();
        let copied = commands
            .execute("editor.copy", &mut state, Value::Null)
            .unwrap();
        assert_eq!(copied, json!({"text": "b = 2\n", "wholeLine": true}));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn find_replace_and_goto_line() {
        let (mut state, commands, root) = state_with("foo bar\nfoo baz\n", "f.txt");
        let found = commands
            .execute("editor.findNext", &mut state, json!({"query": "foo"}))
            .unwrap();
        assert_eq!(found, json!({"matches": 2, "index": 0}));
        let found = commands
            .execute("editor.findNext", &mut state, json!({"query": "foo"}))
            .unwrap();
        assert_eq!(found["index"], 1);
        let found = commands
            .execute("editor.findNext", &mut state, json!({"query": "foo"}))
            .unwrap();
        assert_eq!(found["index"], 0, "wraps around");
        commands
            .execute(
                "editor.replace",
                &mut state,
                json!({"query": "foo", "replacement": "qux"}),
            )
            .unwrap();
        assert_eq!(state.documents[0].text.text(), "qux bar\nfoo baz\n");
        let replaced = commands
            .execute(
                "editor.replaceAll",
                &mut state,
                json!({"query": "BA", "replacement": "_"}),
            )
            .unwrap();
        assert_eq!(replaced["replaced"], 2);
        assert_eq!(state.documents[0].text.text(), "qux _r\nfoo _z\n");
        commands
            .execute(
                "editor.gotoLine",
                &mut state,
                json!({"line": 2, "column": 3}),
            )
            .unwrap();
        assert_eq!(state.documents[0].selections.primary().head, 9);
        std::fs::remove_dir_all(root).unwrap();
    }
}
