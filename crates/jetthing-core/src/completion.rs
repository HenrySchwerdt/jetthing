//! Code completion session: requests, client-side filtering as you type,
//! auto-popup rules and acceptance (Enter inserts, Tab replaces).
//!
//! The server is asked once per session; typing more identifier characters
//! only re-filters locally unless the server marked its list `isIncomplete`.

use crate::IdeState;
use jetthing_lsp::{
    CompletionResult, CompletionTrigger,
    completion::{self as ranking, RankedItem, is_identifier_char},
    lsp_types::{CompletionItem, CompletionTextEdit, InsertTextFormat, TextEdit},
    snippet::{self, Snippet},
};
use jetthing_text::{Selection, TextChange, TextDocument};
use std::path::PathBuf;

/// Trigger characters that are better served by parameter info or that
/// would pop up while typing ordinary text.
const IGNORED_TRIGGERS: &[&str] = &["(", ",", " ", "'", "\"", "=", "\n", "\t"];

#[derive(Debug, Clone)]
pub struct CompletionSession {
    /// Project-relative path of the document being completed.
    pub path: PathBuf,
    /// Char offset where the identifier being completed starts.
    pub anchor: usize,
    /// Caret offset the last request was made at.
    pub request_caret: usize,
    /// Items from the last response, in server order.
    pub items: Vec<CompletionItem>,
    /// Visible rows for the current prefix, best first.
    pub ranked: Vec<RankedItem>,
    /// Index into `ranked`.
    pub selected: usize,
    /// Typed text between `anchor` and the caret.
    pub prefix: String,
    /// The server said more items exist; typing re-queries.
    pub incomplete: bool,
    /// A request is in flight.
    pub waiting: bool,
    /// Opened with Ctrl+Space: an empty result is reported, not hidden.
    pub explicit: bool,
}

impl CompletionSession {
    pub fn selected_item(&self) -> Option<&CompletionItem> {
        self.items.get(self.ranked.get(self.selected)?.index)
    }

    /// Whether the popup has rows to show.
    pub fn visible(&self) -> bool {
        !self.ranked.is_empty()
    }
}

/// Start of the identifier that ends at `caret`.
fn identifier_start(text: &TextDocument, caret: usize) -> usize {
    let mut start = caret;
    while start > 0 && text.char_at(start - 1).is_some_and(is_identifier_char) {
        start -= 1;
    }
    start
}

/// End of the identifier that continues after `caret` (Tab replaces it).
fn identifier_end(text: &TextDocument, caret: usize) -> usize {
    let mut end = caret;
    while text.char_at(end).is_some_and(is_identifier_char) {
        end += 1;
    }
    end
}

impl IdeState {
    /// Requests completion at the caret of the active document.
    pub fn start_completion(
        &mut self,
        trigger: CompletionTrigger,
        explicit: bool,
    ) -> Result<u64, String> {
        let index = self.active_document.ok_or("no active document")?;
        let root = self
            .project
            .as_ref()
            .ok_or("no project is open")?
            .root()
            .to_owned();
        let document = self
            .documents
            .get(index)
            .ok_or("active document does not exist")?;
        let caret = document.selections.primary().head;
        let anchor = identifier_start(&document.text, caret);
        let (line, character) = document
            .text
            .lsp_position(caret)
            .map_err(|error| error.to_string())?;
        let path = document.path.clone();
        let prefix = document
            .text
            .slice(anchor, caret)
            .map_err(|error| error.to_string())?;
        let id = self
            .lsp
            .request_completion(&root.join(&path), line, character, trigger)
            .map_err(|error| error.to_string())?;
        // Keep the visible rows while an `isIncomplete` re-query is running.
        let previous = self
            .completion
            .take()
            .filter(|session| session.path == path && session.anchor == anchor);
        self.completion = Some(CompletionSession {
            path,
            anchor,
            request_caret: caret,
            items: previous
                .as_ref()
                .map(|session| session.items.clone())
                .unwrap_or_default(),
            ranked: previous
                .as_ref()
                .map(|session| session.ranked.clone())
                .unwrap_or_default(),
            selected: 0,
            prefix,
            incomplete: false,
            waiting: true,
            explicit: explicit || previous.is_some_and(|session| session.explicit),
        });
        Ok(id)
    }

    /// Closes the popup and cancels its request.
    pub fn close_completion(&mut self) {
        if self.completion.take().is_some() {
            self.lsp.cancel_completion();
        }
    }

    /// Collects a completion response and keeps the session in sync with the caret.
    pub(crate) fn poll_completion(&mut self) {
        if self.completion.is_none() {
            // Drop a result that arrived for a session that was closed.
            let _ = self.lsp.take_completion();
            return;
        }
        if let Some(result) = self.lsp.take_completion() {
            self.receive_completion(result);
        } else {
            self.refresh_completion();
        }
    }

    /// Installs a server response into the open session and filters it.
    pub fn receive_completion(&mut self, result: CompletionResult) {
        if let Some(session) = self.completion.as_mut() {
            session.items = result.items;
            session.incomplete = result.incomplete;
            session.waiting = false;
            session.selected = 0;
            session.ranked.clear();
            if session.items.is_empty() && session.explicit {
                self.status = "No suggestions".into();
            }
        }
        self.refresh_completion();
    }

    /// Re-filters after the document or caret changed; closes the session
    /// when the caret left the identifier being completed.
    pub fn refresh_completion(&mut self) {
        let Some(session) = self.completion.as_ref() else {
            return;
        };
        let Some(document) = self
            .active_document
            .and_then(|index| self.documents.get(index))
            .filter(|document| document.path == session.path)
        else {
            self.close_completion();
            return;
        };
        let caret = document.selections.primary().head;
        let prefix = if caret >= session.anchor && document.selections.primary().is_caret() {
            document.text.slice(session.anchor, caret).ok()
        } else {
            None
        };
        let Some(prefix) = prefix.filter(|prefix| prefix.chars().all(is_identifier_char)) else {
            self.close_completion();
            return;
        };
        let Some(root) = self
            .project
            .as_ref()
            .map(|project| project.root().to_owned())
        else {
            self.close_completion();
            return;
        };
        let absolute = root.join(&session.path);
        let previous = session
            .selected_item()
            .map(|item| item.label.clone())
            .filter(|_| prefix == session.prefix);
        let ranked = self
            .lsp
            .rank_completions(&absolute, &session.items, &prefix);
        let close = !session.waiting && ranked.is_empty() && !session.explicit;
        let Some(session) = self.completion.as_mut() else {
            return;
        };
        session.selected = previous
            .and_then(|label| {
                ranked
                    .iter()
                    .position(|row| session.items[row.index].label == label)
            })
            .unwrap_or(0);
        session.ranked = ranked;
        session.prefix = prefix;
        if close {
            self.close_completion();
        }
    }

    /// Updates completion after the user typed `typed` into the active
    /// document: re-filters, re-queries incomplete lists and opens the popup
    /// automatically on identifier starts and trigger characters.
    pub fn completion_after_typing(&mut self, typed: &str) {
        let Some(last) = typed.chars().last() else {
            self.refresh_completion();
            return;
        };
        let Some(document) = self
            .active_document
            .and_then(|index| self.documents.get(index))
        else {
            return;
        };
        let caret = document.selections.primary().head;
        if self.completion.is_some() {
            self.refresh_completion();
            if let Some(session) = &self.completion {
                if session.incomplete && !session.waiting {
                    let _ = self.start_completion(CompletionTrigger::Incomplete, false);
                }
                return;
            }
        }
        if self.caret_in_comment_or_string(caret) {
            return;
        }
        let trigger = self.trigger_before_caret(caret);
        if let Some(trigger) = trigger {
            let _ = self.start_completion(CompletionTrigger::Character(trigger), false);
            return;
        }
        let Some(document) = self
            .active_document
            .and_then(|index| self.documents.get(index))
        else {
            return;
        };
        // Inside a template variable the popup only opens on request or on
        // trigger characters, so Tab keeps moving between variables.
        let starts_word = self.snippet.is_none()
            && is_identifier_char(last)
            && !last.is_ascii_digit()
            && identifier_start(&document.text, caret) + 1 == caret;
        if starts_word {
            let _ = self.start_completion(CompletionTrigger::Invoked, false);
        }
    }

    /// The server trigger sequence that ends at `caret`, if any (`.`, `::`).
    fn trigger_before_caret(&self, caret: usize) -> Option<String> {
        let document = self.documents.get(self.active_document?)?;
        let root = self.project.as_ref()?.root();
        let mut triggers = self.lsp.trigger_characters(&root.join(&document.path));
        if triggers.is_empty() {
            return None;
        }
        // rust-analyzer registers ":"; the popup should open on "::" only.
        if triggers.iter().any(|t| t == ":") {
            triggers.retain(|t| t != ":");
            triggers.push("::".into());
        }
        triggers
            .into_iter()
            .filter(|trigger| !IGNORED_TRIGGERS.contains(&trigger.as_str()))
            .find(|trigger| {
                let count = trigger.chars().count();
                caret >= count
                    && document
                        .text
                        .slice(caret - count, caret)
                        .is_ok_and(|text| &text == trigger)
            })
    }

    fn caret_in_comment_or_string(&self, caret: usize) -> bool {
        let Some(document) = self
            .active_document
            .and_then(|index| self.documents.get(index))
        else {
            return false;
        };
        let text = document.text.text();
        let byte = text
            .char_indices()
            .nth(caret)
            .map_or(text.len(), |(byte, _)| byte);
        // Highlight keys are IntelliJ text attribute keys
        // (`DEFAULT_LINE_COMMENT`, `DEFAULT_STRING`, ...).
        document
            .analysis
            .highlights
            .in_range(byte.saturating_sub(1)..byte + 1)
            .any(|span| {
                (span.key.contains("COMMENT") || span.key.contains("STRING"))
                    && span.start_byte < byte
                    && byte < span.end_byte
            })
    }

    /// Moves the popup selection by `delta` rows, wrapping around.
    pub fn move_completion_selection(&mut self, delta: isize) {
        if let Some(session) = self.completion.as_mut()
            && !session.ranked.is_empty()
        {
            let count = session.ranked.len() as isize;
            session.selected = (session.selected as isize + delta).rem_euclid(count) as usize;
        }
    }

    /// Selects a row; out-of-range indices are ignored.
    pub fn select_completion(&mut self, row: usize) {
        if let Some(session) = self.completion.as_mut()
            && row < session.ranked.len()
        {
            session.selected = row;
        }
    }

    /// Accepts the selected item. `replace` (Tab) also overwrites the rest of
    /// the identifier after the caret; otherwise (Enter) the text is inserted.
    pub fn accept_completion(&mut self, replace: bool) -> Result<(), String> {
        let session = self.completion.take().ok_or("no completion is showing")?;
        self.lsp.cancel_completion();
        let item = session
            .selected_item()
            .cloned()
            .ok_or("no completion item is selected")?;
        let index = self.active_document.ok_or("no active document")?;
        let document = self
            .documents
            .get_mut(index)
            .ok_or("active document does not exist")?;
        if document.path != session.path {
            return Err("the completed document is no longer active".into());
        }
        let caret = document.selections.primary().head;
        let snippet_format = item.insert_text_format == Some(InsertTextFormat::SNIPPET);
        let (start, new_text) = match &item.text_edit {
            Some(CompletionTextEdit::Edit(edit)) => (
                document.text.char_from_lsp_position(
                    edit.range.start.line as usize,
                    edit.range.start.character as usize,
                ),
                edit.new_text.clone(),
            ),
            Some(CompletionTextEdit::InsertAndReplace(edit)) => (
                document.text.char_from_lsp_position(
                    edit.insert.start.line as usize,
                    edit.insert.start.character as usize,
                ),
                edit.new_text.clone(),
            ),
            None => (session.anchor, ranking::insert_text(&item).to_owned()),
        };
        let start = start.min(caret);
        let file_name = document
            .path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        // Multi-line insertions follow the indentation of the current line.
        let new_text = if new_text.contains('\n') {
            let line = document.text.char_to_line(start);
            let indent: String = document
                .text
                .line(line)
                .unwrap_or_default()
                .chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect();
            adjust_indentation(&new_text, &indent)
        } else {
            new_text
        };
        let snippet = snippet_format.then(|| {
            let variables = |name: &str| match name {
                "TM_FILENAME" => file_name.clone(),
                _ => None,
            };
            snippet::parse(&new_text, &variables)
        });
        let new_text = snippet
            .as_ref()
            .map_or(new_text, |snippet| snippet.text.clone());
        let end = if replace {
            identifier_end(&document.text, caret)
        } else {
            caret
        };
        let mut changes = vec![TextChange {
            start_char: start,
            end_char: end,
            inserted: new_text.clone(),
        }];
        let mut shift: isize = 0;
        for edit in item.additional_text_edits.iter().flatten() {
            let change = lsp_change(&document.text, edit);
            if change.end_char <= start {
                shift += change.inserted.chars().count() as isize
                    - (change.end_char - change.start_char) as isize;
                changes.push(change);
            } else if change.start_char >= end {
                changes.push(change);
            }
        }
        if document.text.edit_many(changes.clone()).is_err() {
            // Overlapping auxiliary edits: apply the completion itself.
            shift = 0;
            document
                .text
                .edit(start, end, &new_text)
                .map_err(|error| error.to_string())?;
        }
        let inserted_at = start.saturating_add_signed(shift);
        let caret = inserted_at
            + snippet
                .as_ref()
                .map_or(new_text.chars().count(), Snippet::final_caret);
        document.selections.set_single(Selection::caret(caret));
        let source = document.text.text();
        document.refresh_analysis(&source);
        match snippet.filter(Snippet::is_interactive) {
            Some(snippet) => self.start_snippet(&snippet, inserted_at),
            // Completing inside a template variable keeps the template.
            None => self.snippet_after_change(),
        }
        self.sync_active_lsp();
        Ok(())
    }
}

/// Indents every line after the first with `indent`, and expands tabs to the
/// indentation unit of the line (a tab if it is tab-indented, else 4 spaces).
fn adjust_indentation(text: &str, indent: &str) -> String {
    let unit = if indent.starts_with('\t') {
        "\t"
    } else {
        "    "
    };
    let mut lines = text.split('\n');
    let mut out = lines.next().unwrap_or_default().replace('\t', unit);
    for line in lines {
        out.push('\n');
        out.push_str(indent);
        out.push_str(&line.replace('\t', unit));
    }
    out
}

fn lsp_change(text: &TextDocument, edit: &TextEdit) -> TextChange {
    let start = text.char_from_lsp_position(
        edit.range.start.line as usize,
        edit.range.start.character as usize,
    );
    let end = text
        .char_from_lsp_position(
            edit.range.end.line as usize,
            edit.range.end.character as usize,
        )
        .max(start);
    TextChange {
        start_char: start,
        end_char: end,
        inserted: edit.new_text.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::built_in_commands;
    use jetthing_lsp::lsp_types::{
        CompletionItem, CompletionTextEdit, InsertReplaceEdit, InsertTextFormat, Position, Range,
        TextEdit,
    };
    use serde_json::json;

    fn item(label: &str) -> CompletionItem {
        CompletionItem {
            label: label.into(),
            ..Default::default()
        }
    }

    fn range(line: u32, start: u32, end: u32) -> Range {
        Range::new(Position::new(line, start), Position::new(line, end))
    }

    /// A project with one open document and the caret at `caret`.
    fn state_with(text: &str, caret: usize) -> (IdeState, PathBuf) {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jetthing-completion-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("notes.txt"), text).unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("editor.openFile", &mut state, json!({"path": "notes.txt"}))
            .unwrap();
        state.documents[0]
            .selections
            .set_single(Selection::caret(caret));
        (state, root)
    }

    fn open_session(state: &mut IdeState, anchor: usize, explicit: bool) {
        state.completion = Some(CompletionSession {
            path: PathBuf::from("notes.txt"),
            anchor,
            request_caret: state.documents[0].selections.primary().head,
            items: Vec::new(),
            ranked: Vec::new(),
            selected: 0,
            prefix: String::new(),
            incomplete: false,
            waiting: true,
            explicit,
        });
    }

    fn visible_labels(state: &IdeState) -> Vec<String> {
        let session = state.completion.as_ref().unwrap();
        session
            .ranked
            .iter()
            .map(|row| session.items[row.index].label.clone())
            .collect()
    }

    fn type_text(state: &mut IdeState, text: &str) {
        let document = &mut state.documents[0];
        let caret = document.selections.primary().head;
        document.text.edit(caret, caret, text).unwrap();
        document
            .selections
            .set_single(Selection::caret(caret + text.chars().count()));
        state.completion_after_typing(text);
    }

    #[test]
    fn filters_as_you_type_and_closes_when_nothing_matches() {
        let (mut state, root) = state_with("app.", 4);
        open_session(&mut state, 4, false);
        state.receive_completion(CompletionResult {
            items: ["greeting", "getRequiredField", "getClass", "hashCode"]
                .into_iter()
                .map(item)
                .collect(),
            incomplete: false,
        });
        assert_eq!(visible_labels(&state).len(), 4);
        type_text(&mut state, "g");
        assert_eq!(
            visible_labels(&state),
            ["getClass", "getRequiredField", "greeting"]
        );
        type_text(&mut state, "RF");
        assert_eq!(visible_labels(&state), ["getRequiredField"]);
        assert_eq!(state.completion.as_ref().unwrap().prefix, "gRF");
        type_text(&mut state, "x");
        assert!(state.completion.is_none(), "no match closes the popup");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn leaving_the_identifier_closes_the_session() {
        let (mut state, root) = state_with("app.ge", 6);
        open_session(&mut state, 4, false);
        state.receive_completion(CompletionResult {
            items: vec![item("getClass")],
            incomplete: false,
        });
        assert!(state.completion.is_some());
        state.documents[0]
            .selections
            .set_single(Selection::caret(2));
        state.refresh_completion();
        assert!(state.completion.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn explicit_empty_results_report_no_suggestions() {
        let (mut state, root) = state_with("zz", 2);
        open_session(&mut state, 0, true);
        state.receive_completion(CompletionResult::default());
        assert_eq!(state.status, "No suggestions");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn enter_inserts_and_tab_replaces_the_identifier() {
        for (replace, expected) in [(false, "app.greetingName();"), (true, "app.greeting();")] {
            let (mut state, root) = state_with("app.gName();", 5);
            open_session(&mut state, 4, false);
            let mut greeting = item("greeting");
            greeting.text_edit = Some(CompletionTextEdit::InsertAndReplace(InsertReplaceEdit {
                new_text: "greeting".into(),
                insert: range(0, 4, 5),
                replace: range(0, 4, 9),
            }));
            state.receive_completion(CompletionResult {
                items: vec![greeting],
                incomplete: false,
            });
            state.accept_completion(replace).unwrap();
            let document = &state.documents[0];
            assert_eq!(document.text.text(), expected);
            assert_eq!(document.selections.primary().head, 12);
            assert!(state.completion.is_none());
            std::fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn additional_text_edits_add_imports_in_one_undo_step() {
        let (mut state, root) = state_with("class A {\n  Li\n}\n", 14);
        open_session(&mut state, 12, false);
        let mut list = item("List");
        list.text_edit = Some(CompletionTextEdit::Edit(TextEdit::new(
            range(1, 2, 4),
            "List".into(),
        )));
        list.additional_text_edits = Some(vec![TextEdit::new(
            range(0, 0, 0),
            "import java.util.List;\n\n".into(),
        )]);
        state.receive_completion(CompletionResult {
            items: vec![list],
            incomplete: false,
        });
        state.accept_completion(false).unwrap();
        let document = &mut state.documents[0];
        assert_eq!(
            document.text.text(),
            "import java.util.List;\n\nclass A {\n  List\n}\n"
        );
        let caret = document.selections.primary().head;
        assert_eq!(document.text.slice(caret - 4, caret).unwrap(), "List");
        assert!(document.text.undo());
        assert_eq!(document.text.text(), "class A {\n  Li\n}\n");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn plain_items_replace_the_typed_prefix_and_selection_wraps() {
        let (mut state, root) = state_with("x = has", 7);
        open_session(&mut state, 4, false);
        state.receive_completion(CompletionResult {
            items: vec![item("hashCode"), item("hasNext")],
            incomplete: false,
        });
        state.move_completion_selection(-1);
        assert_eq!(state.completion.as_ref().unwrap().selected, 1);
        state.move_completion_selection(1);
        assert_eq!(state.completion.as_ref().unwrap().selected, 0);
        state.accept_completion(false).unwrap();
        assert_eq!(state.documents[0].text.text(), "x = hashCode");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn snippet_items_expand_and_start_a_tab_stop_session() {
        let (mut state, root) = state_with("app.gr;", 6);
        open_session(&mut state, 4, false);
        let mut greeting = item("greeting");
        greeting.insert_text_format = Some(InsertTextFormat::SNIPPET);
        greeting.text_edit = Some(CompletionTextEdit::Edit(TextEdit::new(
            range(0, 4, 6),
            "greeting(${1:name})$0".into(),
        )));
        let mut plain_call = item("toString");
        plain_call.insert_text_format = Some(InsertTextFormat::SNIPPET);
        plain_call.insert_text = Some("toString()$0".into());
        state.receive_completion(CompletionResult {
            items: vec![greeting, plain_call],
            incomplete: false,
        });
        state.accept_completion(false).unwrap();
        assert_eq!(state.documents[0].text.text(), "app.greeting(name);");
        let selection = state.documents[0].selections.primary();
        assert_eq!((selection.start(), selection.end()), (13, 17));
        assert!(state.snippet.is_some());
        state.snippet_next();
        assert!(state.snippet.is_none());
        assert_eq!(state.documents[0].selections.primary().head, 18);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn multi_line_snippets_follow_the_line_indentation() {
        assert_eq!(
            adjust_indentation("for (;;) {\n\t$0\n}", "        "),
            "for (;;) {\n            $0\n        }"
        );
        assert_eq!(adjust_indentation("a\n\tb", "\t"), "a\n\t\tb");
        let (mut state, root) = state_with("    fo", 6);
        open_session(&mut state, 4, false);
        let mut fori = item("fori");
        fori.insert_text_format = Some(InsertTextFormat::SNIPPET);
        fori.insert_text = Some("for (${1:i}) {\n\t$0\n}".into());
        state.receive_completion(CompletionResult {
            items: vec![fori],
            incomplete: false,
        });
        state.accept_completion(false).unwrap();
        assert_eq!(
            state.documents[0].text.text(),
            "    for (i) {\n        \n    }"
        );
        state.snippet_next();
        assert_eq!(state.documents[0].selections.primary().head, 22);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn snippets_without_placeholders_just_place_the_caret() {
        let (mut state, root) = state_with("s.to", 4);
        open_session(&mut state, 2, false);
        let mut call = item("toString");
        call.insert_text_format = Some(InsertTextFormat::SNIPPET);
        call.insert_text = Some("toString($0)".into());
        state.receive_completion(CompletionResult {
            items: vec![call],
            incomplete: false,
        });
        state.accept_completion(false).unwrap();
        assert_eq!(state.documents[0].text.text(), "s.toString()");
        assert_eq!(state.documents[0].selections.primary().head, 11);
        assert!(state.snippet.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn no_auto_popup_inside_comments_and_strings() {
        let root =
            std::env::temp_dir().join(format!("jetthing-completion-rs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let text = "fn f() { let s = \"ab\"; // cd\n}\n";
        std::fs::write(root.join("lib.rs"), text).unwrap();
        let commands = built_in_commands();
        let mut state = IdeState::default();
        commands
            .execute("project.open", &mut state, json!({"path": root}))
            .unwrap();
        commands
            .execute("editor.openFile", &mut state, json!({"path": "lib.rs"}))
            .unwrap();
        let at = |needle: &str| text.find(needle).unwrap();
        assert!(state.caret_in_comment_or_string(at("ab") + 1));
        assert!(state.caret_in_comment_or_string(at("cd") + 1));
        assert!(!state.caret_in_comment_or_string(at("let") + 1));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn completion_commands_drive_the_session() {
        let (mut state, root) = state_with("has", 3);
        let commands = built_in_commands();
        assert!(!commands.is_enabled("completion.accept", &state));
        open_session(&mut state, 0, false);
        state.receive_completion(CompletionResult {
            items: vec![item("hashCode"), item("hasNext")],
            incomplete: false,
        });
        let moved = commands
            .execute("completion.move", &mut state, json!({"delta": 1}))
            .unwrap();
        assert_eq!(moved["selected"], 1);
        commands
            .execute("completion.accept", &mut state, json!({"row": 0}))
            .unwrap();
        assert_eq!(state.documents[0].text.text(), "hashCode");
        std::fs::remove_dir_all(root).unwrap();
    }
}
