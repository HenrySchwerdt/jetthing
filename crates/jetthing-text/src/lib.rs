use ropey::Rope;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Edit {
    start_char: usize,
    deleted: String,
    inserted: String,
    before_state: u64,
    after_state: u64,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TextError {
    #[error("invalid character range {start}..{end} for document length {len}")]
    InvalidRange {
        start: usize,
        end: usize,
        len: usize,
    },
}

/// Revisioned UTF-8 text storage. All public offsets are Unicode scalar-value
/// (character) offsets, matching Ropey's native indexing and never splitting UTF-8.
#[derive(Debug, Clone)]
pub struct TextDocument {
    rope: Rope,
    revision: u64,
    current_state: u64,
    saved_state: u64,
    next_state: u64,
    undo: Vec<Edit>,
    redo: Vec<Edit>,
}

impl TextDocument {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
            revision: 0,
            current_state: 0,
            saved_state: 0,
            next_state: 1,
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn is_dirty(&self) -> bool {
        self.current_state != self.saved_state
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn mark_saved(&mut self) {
        self.saved_state = self.current_state;
    }

    /// Converts a rope character offset to the UTF-16 line/character position
    /// required by the Language Server Protocol.
    pub fn lsp_position(&self, char_index: usize) -> Result<(usize, usize), TextError> {
        self.validate_range(char_index, char_index)?;
        let line = self.rope.char_to_line(char_index);
        let line_start = self.rope.line_to_char(line);
        let utf16_character = self
            .rope
            .slice(line_start..char_index)
            .chars()
            .map(char::len_utf16)
            .sum();
        Ok((line, utf16_character))
    }

    pub fn edit(
        &mut self,
        start_char: usize,
        end_char: usize,
        inserted: &str,
    ) -> Result<(), TextError> {
        self.validate_range(start_char, end_char)?;
        let deleted = self.rope.slice(start_char..end_char).to_string();
        if deleted == inserted {
            return Ok(());
        }
        let before_state = self.current_state;
        let after_state = self.next_state;
        self.next_state = self.next_state.saturating_add(1);
        self.apply(start_char, end_char, inserted);
        self.undo.push(Edit {
            start_char,
            deleted,
            inserted: inserted.to_owned(),
            before_state,
            after_state,
        });
        self.redo.clear();
        self.revision = self.revision.saturating_add(1);
        self.current_state = after_state;
        Ok(())
    }

    /// Converts a UI-produced snapshot into one minimal contiguous edit. This keeps
    /// the engine transactional while the first UI shell still uses egui TextEdit.
    pub fn replace_from_snapshot(&mut self, next: &str) -> Result<(), TextError> {
        let current = self.text();
        if current == next {
            return Ok(());
        }
        let old_chars: Vec<char> = current.chars().collect();
        let new_chars: Vec<char> = next.chars().collect();
        let prefix = old_chars
            .iter()
            .zip(&new_chars)
            .take_while(|(a, b)| a == b)
            .count();
        let suffix = old_chars[prefix..]
            .iter()
            .rev()
            .zip(new_chars[prefix..].iter().rev())
            .take_while(|(a, b)| a == b)
            .count();
        let old_end = old_chars.len() - suffix;
        let new_end = new_chars.len() - suffix;
        let inserted: String = new_chars[prefix..new_end].iter().collect();
        self.edit(prefix, old_end, &inserted)
    }

    pub fn undo(&mut self) -> bool {
        let Some(edit) = self.undo.pop() else {
            return false;
        };
        let inserted_len = edit.inserted.chars().count();
        self.apply(
            edit.start_char,
            edit.start_char + inserted_len,
            &edit.deleted,
        );
        self.current_state = edit.before_state;
        self.redo.push(edit);
        self.revision = self.revision.saturating_add(1);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(edit) = self.redo.pop() else {
            return false;
        };
        let deleted_len = edit.deleted.chars().count();
        self.apply(
            edit.start_char,
            edit.start_char + deleted_len,
            &edit.inserted,
        );
        self.current_state = edit.after_state;
        self.undo.push(edit);
        self.revision = self.revision.saturating_add(1);
        true
    }

    fn validate_range(&self, start: usize, end: usize) -> Result<(), TextError> {
        let len = self.len_chars();
        if start > end || end > len {
            return Err(TextError::InvalidRange { start, end, len });
        }
        Ok(())
    }

    fn apply(&mut self, start: usize, end: usize, inserted: &str) {
        if start != end {
            self.rope.remove(start..end);
        }
        if !inserted.is_empty() {
            self.rope.insert(start, inserted);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn edits_unicode_without_splitting_and_tracks_history() {
        let mut doc = TextDocument::new("a🦀c\n");
        doc.edit(1, 2, "Rust").unwrap();
        assert_eq!(doc.text(), "aRustc\n");
        assert!(doc.is_dirty());
        assert!(doc.undo());
        assert_eq!(doc.text(), "a🦀c\n");
        assert!(doc.redo());
        assert_eq!(doc.text(), "aRustc\n");
    }

    #[test]
    fn snapshot_replacement_records_only_changed_span() {
        let mut doc = TextDocument::new("alpha beta gamma");
        doc.replace_from_snapshot("alpha delta gamma").unwrap();
        assert_eq!(doc.text(), "alpha delta gamma");
        assert!(doc.undo());
        assert_eq!(doc.text(), "alpha beta gamma");
    }

    #[test]
    fn undo_back_to_saved_state_clears_dirty_and_branching_stays_dirty() {
        let mut doc = TextDocument::new("one");
        doc.edit(3, 3, " two").unwrap();
        doc.mark_saved();
        doc.edit(7, 7, " three").unwrap();
        assert!(doc.is_dirty());
        assert!(doc.undo());
        assert!(!doc.is_dirty());
        assert!(doc.undo());
        assert!(doc.is_dirty());
        doc.edit(0, 3, "ONE").unwrap();
        assert!(doc.is_dirty());
    }

    #[test]
    fn converts_character_offsets_to_lsp_utf16_positions() {
        let doc = TextDocument::new("one\na🦀b\n");
        assert_eq!(doc.lsp_position(4).unwrap(), (1, 0));
        assert_eq!(doc.lsp_position(6).unwrap(), (1, 3));
        assert!(doc.lsp_position(99).is_err());
    }

    proptest! {
        #[test]
        fn edit_then_undo_restores_original(original in ".{0,200}", replacement in ".{0,80}") {
            let mut doc = TextDocument::new(&original);
            let len = original.chars().count();
            doc.edit(0, len, &replacement).unwrap();
            prop_assert!(doc.undo());
            prop_assert_eq!(doc.text(), original);
        }
    }
}
