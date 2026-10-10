//! Editor actions on a document and its selections.
//!
//! Every mutating action is a single undo step (one `edit_many` transaction) and
//! leaves the selection set in the state the reference IDE shows after the action.

use crate::{Selection, SelectionSet, TextChange, TextDocument, TextError};

const INDENT: &str = "    ";

/// Character range of a line: (start, end of content, end including the newline).
pub fn line_bounds(doc: &TextDocument, line: usize) -> (usize, usize, usize) {
    let start = doc.line_to_char(line).unwrap_or(doc.len_chars());
    let full_end = doc.line_to_char(line + 1).unwrap_or(doc.len_chars());
    let mut content_end = full_end;
    while content_end > start && matches!(doc.char_at(content_end - 1), Some('\n') | Some('\r')) {
        content_end -= 1;
    }
    (start, content_end, full_end)
}

fn line_text(doc: &TextDocument, line: usize) -> String {
    let (start, end, _) = line_bounds(doc, line);
    doc.slice(start, end).unwrap_or_default()
}

/// Lines touched by a selection. A non-empty selection ending at column 0 does not
/// include that final line (matching line-based actions in the reference IDE).
pub fn selection_lines(doc: &TextDocument, selection: Selection) -> (usize, usize) {
    let first = doc.char_to_line(selection.start());
    let mut last = doc.char_to_line(selection.end());
    if !selection.is_caret() && last > first && doc.line_to_char(last) == Some(selection.end()) {
        last -= 1;
    }
    (first, last)
}

fn covered_lines(doc: &TextDocument, selections: &SelectionSet) -> Vec<usize> {
    let mut lines: Vec<usize> = selections
        .selections()
        .iter()
        .flat_map(|selection| {
            let (first, last) = selection_lines(doc, *selection);
            first..=last
        })
        .collect();
    lines.sort_unstable();
    lines.dedup();
    lines
}

fn indentation(text: &str) -> String {
    text.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect()
}

/// Text to place on the clipboard and whether it is a whole-line copy. Without a
/// selection the reference IDE copies the caret lines including their newline.
pub fn copy_text(doc: &TextDocument, selections: &SelectionSet) -> (String, bool) {
    if selections.selections().iter().all(|s| s.is_caret()) {
        let mut text = String::new();
        for line in covered_lines(doc, selections) {
            text.push_str(&line_text(doc, line));
            text.push('\n');
        }
        (text, true)
    } else {
        let parts: Vec<String> = selections
            .selections()
            .iter()
            .filter(|s| !s.is_caret())
            .map(|s| doc.slice(s.start(), s.end()).unwrap_or_default())
            .collect();
        (parts.join("\n"), false)
    }
}

/// Cuts the selection (or the caret lines) and returns the clipboard text.
pub fn cut(doc: &mut TextDocument, selections: &mut SelectionSet) -> Result<String, TextError> {
    let (text, whole_line) = copy_text(doc, selections);
    if whole_line {
        delete_lines(doc, selections)?;
    } else {
        selections.insert(doc, "")?;
    }
    Ok(text)
}

/// Pastes clipboard text. A whole-line clip pasted at a caret goes above the caret line.
pub fn paste(
    doc: &mut TextDocument,
    selections: &mut SelectionSet,
    text: &str,
    whole_line: bool,
) -> Result<(), TextError> {
    if whole_line && selections.selections().iter().all(|s| s.is_caret()) {
        let mut changes = Vec::new();
        let mut next = Vec::new();
        let inserted = text.chars().count();
        for (index, selection) in selections.selections().iter().enumerate() {
            let line = doc.char_to_line(selection.head);
            let (start, _, _) = line_bounds(doc, line);
            changes.push(TextChange {
                start_char: start,
                end_char: start,
                inserted: text.to_owned(),
            });
            next.push(Selection::caret(selection.head + inserted * (index + 1)));
        }
        doc.edit_many(changes)?;
        selections.set_all(next);
        return Ok(());
    }
    selections.insert(doc, text)
}

/// Deletes the lines covered by the selections (Ctrl+Y / ⌘⌫).
pub fn delete_lines(
    doc: &mut TextDocument,
    selections: &mut SelectionSet,
) -> Result<(), TextError> {
    let lines = covered_lines(doc, selections);
    let Some(&first) = lines.first() else {
        return Ok(());
    };
    let column = doc
        .char_to_line_column(selections.primary().head)
        .map(|(_, column)| column)
        .unwrap_or(0);
    let mut changes = Vec::new();
    for group in contiguous(&lines) {
        let (mut start, _, _) = line_bounds(doc, group.0);
        let (_, _, end) = line_bounds(doc, group.1);
        // Deleting a final line without newline also removes the newline before it.
        if start > 0 && !doc.slice(start, end)?.ends_with('\n') {
            start -= 1;
            if start > 0 && doc.char_at(start - 1) == Some('\r') {
                start -= 1;
            }
        }
        changes.push(TextChange {
            start_char: start,
            end_char: end,
            inserted: String::new(),
        });
    }
    doc.edit_many(changes)?;
    let line = first.min(doc.len_lines().saturating_sub(1));
    let (start, end, _) = line_bounds(doc, line);
    selections.set_single(Selection::caret((start + column).min(end)));
    Ok(())
}

fn contiguous(lines: &[usize]) -> Vec<(usize, usize)> {
    let mut groups: Vec<(usize, usize)> = Vec::new();
    for &line in lines {
        match groups.last_mut() {
            Some(group) if group.1 + 1 == line => group.1 = line,
            _ => groups.push((line, line)),
        }
    }
    groups
}

/// Duplicates the selection, or the caret line when nothing is selected (⌘D / Ctrl+D).
pub fn duplicate(doc: &mut TextDocument, selections: &mut SelectionSet) -> Result<(), TextError> {
    let mut changes = Vec::new();
    let mut next = Vec::new();
    let mut shift = 0usize;
    for selection in selections.selections().to_vec() {
        if selection.is_caret() {
            let line = doc.char_to_line(selection.head);
            let (start, content_end, full_end) = line_bounds(doc, line);
            let content = doc.slice(start, content_end)?;
            let (position, inserted) = if full_end > content_end {
                (full_end, format!("{content}\n"))
            } else {
                (content_end, format!("\n{content}"))
            };
            let added = inserted.chars().count();
            changes.push(TextChange {
                start_char: position,
                end_char: position,
                inserted,
            });
            next.push(Selection::caret(selection.head + shift + added));
            shift += added;
        } else {
            let text = doc.slice(selection.start(), selection.end())?;
            let added = text.chars().count();
            changes.push(TextChange {
                start_char: selection.end(),
                end_char: selection.end(),
                inserted: text,
            });
            next.push(Selection {
                anchor: selection.end() + shift,
                head: selection.end() + shift + added,
            });
            shift += added;
        }
    }
    doc.edit_many(changes)?;
    selections.set_all(next);
    Ok(())
}

/// Toggles line comments with `prefix` (⌘/ / Ctrl+/). A single caret moves down a line.
pub fn toggle_line_comment(
    doc: &mut TextDocument,
    selections: &mut SelectionSet,
    prefix: &str,
) -> Result<(), TextError> {
    let lines = covered_lines(doc, selections);
    let texts: Vec<(usize, String)> = lines.iter().map(|&l| (l, line_text(doc, l))).collect();
    let non_blank: Vec<&(usize, String)> =
        texts.iter().filter(|(_, t)| !t.trim().is_empty()).collect();
    if non_blank.is_empty() {
        return Ok(());
    }
    let uncomment = non_blank
        .iter()
        .all(|(_, text)| text.trim_start().starts_with(prefix));
    let min_indent = non_blank
        .iter()
        .map(|(_, text)| indentation(text).chars().count())
        .min()
        .unwrap_or(0);
    let mut changes = Vec::new();
    for (line, text) in &non_blank {
        let (start, _, _) = line_bounds(doc, *line);
        if uncomment {
            let indent = indentation(text).chars().count();
            let after = &text[indentation(text).len() + prefix.len()..];
            let remove = prefix.chars().count() + usize::from(after.starts_with(' '));
            changes.push(TextChange {
                start_char: start + indent,
                end_char: start + indent + remove,
                inserted: String::new(),
            });
        } else {
            changes.push(TextChange {
                start_char: start + min_indent,
                end_char: start + min_indent,
                inserted: format!("{prefix} "),
            });
        }
    }
    let single_caret = selections.selections().len() == 1 && selections.primary().is_caret();
    let primary = selections.primary();
    doc.edit_many(changes)?;
    if single_caret {
        let line = doc.char_to_line(primary.head.min(doc.len_chars()));
        let column = doc
            .char_to_line_column(primary.head.min(doc.len_chars()))
            .map(|(_, c)| c)
            .unwrap_or(0);
        let target = (line + 1).min(doc.len_lines().saturating_sub(1));
        let (start, end, _) = line_bounds(doc, target);
        selections.set_single(Selection::caret((start + column).min(end)));
    } else {
        let first = line_bounds(doc, lines[0]).0;
        let last = line_bounds(doc, *lines.last().unwrap_or(&lines[0])).1;
        selections.set_single(Selection {
            anchor: first,
            head: last,
        });
    }
    Ok(())
}

/// Moves the covered lines one line up or down (⌥⇧↑/↓ / Alt+Shift+↑/↓).
pub fn move_lines(
    doc: &mut TextDocument,
    selections: &mut SelectionSet,
    up: bool,
) -> Result<(), TextError> {
    let lines = covered_lines(doc, selections);
    let (Some(&first), Some(&last)) = (lines.first(), lines.last()) else {
        return Ok(());
    };
    let line_count = doc.len_lines();
    // Treat a trailing empty line produced by a final newline as nonexistent.
    let real_lines = if doc.len_chars() > 0 && line_text(doc, line_count - 1).is_empty() {
        line_count - 1
    } else {
        line_count
    };
    if (up && first == 0) || (!up && last + 1 >= real_lines) {
        return Ok(());
    }
    let (region_first, region_last) = if up {
        (first - 1, last)
    } else {
        (first, last + 1)
    };
    let region_start = line_bounds(doc, region_first).0;
    let region_end = line_bounds(doc, region_last).1;
    let region = doc.slice(region_start, region_end)?;
    let trailing_newline = region.ends_with('\n');
    let mut parts: Vec<String> = (region_first..=region_last)
        .map(|l| line_text(doc, l))
        .collect();
    let neighbor_len;
    if up {
        let moved = parts.remove(0);
        neighbor_len = moved.chars().count() + 1;
        parts.push(moved);
    } else {
        let moved = parts.pop().unwrap_or_default();
        neighbor_len = moved.chars().count() + 1;
        parts.insert(0, moved);
    }
    let mut replacement = parts.join("\n");
    if trailing_newline {
        replacement.push('\n');
    }
    doc.edit(region_start, region_end, &replacement)?;
    let moved: Vec<Selection> = selections
        .selections()
        .iter()
        .map(|s| {
            let shift = |offset: usize| {
                if up {
                    offset.saturating_sub(neighbor_len)
                } else {
                    (offset + neighbor_len).min(doc.len_chars())
                }
            };
            Selection {
                anchor: shift(s.anchor),
                head: shift(s.head),
            }
        })
        .collect();
    selections.set_all(moved);
    Ok(())
}

/// Indents covered lines by one level (Tab with a multi-line selection).
pub fn indent_lines(
    doc: &mut TextDocument,
    selections: &mut SelectionSet,
) -> Result<(), TextError> {
    let lines = covered_lines(doc, selections);
    let changes: Vec<TextChange> = lines
        .iter()
        .filter(|&&line| !line_text(doc, line).is_empty())
        .map(|&line| {
            let start = line_bounds(doc, line).0;
            TextChange {
                start_char: start,
                end_char: start,
                inserted: INDENT.into(),
            }
        })
        .collect();
    let before: Vec<Selection> = selections.selections().to_vec();
    doc.edit_many(changes.clone())?;
    selections.set_all(shift_selections(&before, &changes));
    Ok(())
}

/// Removes one indentation level from covered lines (⇧Tab).
pub fn unindent_lines(
    doc: &mut TextDocument,
    selections: &mut SelectionSet,
) -> Result<(), TextError> {
    let lines = covered_lines(doc, selections);
    let changes: Vec<TextChange> = lines
        .iter()
        .filter_map(|&line| {
            let text = line_text(doc, line);
            let remove = if text.starts_with('\t') {
                1
            } else {
                text.chars()
                    .take(INDENT.len())
                    .take_while(|c| *c == ' ')
                    .count()
            };
            (remove > 0).then(|| {
                let start = line_bounds(doc, line).0;
                TextChange {
                    start_char: start,
                    end_char: start + remove,
                    inserted: String::new(),
                }
            })
        })
        .collect();
    let before: Vec<Selection> = selections.selections().to_vec();
    doc.edit_many(changes.clone())?;
    selections.set_all(shift_selections(&before, &changes));
    Ok(())
}

/// Maps offsets through a set of pre-transaction changes.
fn map_offset(offset: usize, changes: &[TextChange]) -> usize {
    let mut result = offset as isize;
    for change in changes {
        let removed = (change.end_char - change.start_char) as isize;
        let added = change.inserted.chars().count() as isize;
        if change.end_char <= offset {
            result += added - removed;
        } else if change.start_char < offset {
            result -= (offset - change.start_char) as isize;
            result += added.min((offset - change.start_char) as isize);
        }
    }
    result.max(0) as usize
}

fn shift_selections(before: &[Selection], changes: &[TextChange]) -> Vec<Selection> {
    before
        .iter()
        .map(|s| Selection {
            anchor: map_offset(s.anchor, changes),
            head: map_offset(s.head, changes),
        })
        .collect()
}

fn closing_for(open: char) -> Option<char> {
    match open {
        '(' => Some(')'),
        '[' => Some(']'),
        '{' => Some('}'),
        '"' => Some('"'),
        '\'' => Some('\''),
        '`' => Some('`'),
        _ => None,
    }
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Types one character with bracket/quote pairing and selection surrounding.
/// Returns `Ok(false)` when the character needs no special handling.
pub fn type_char(
    doc: &mut TextDocument,
    selections: &mut SelectionSet,
    typed: char,
) -> Result<bool, TextError> {
    let all_carets = selections.selections().iter().all(|s| s.is_caret());
    // Overtype a closing character that was auto-inserted.
    if all_carets
        && matches!(typed, ')' | ']' | '}' | '"' | '\'' | '`')
        && selections
            .selections()
            .iter()
            .all(|s| doc.char_at(s.head) == Some(typed))
    {
        let next: Vec<Selection> = selections
            .selections()
            .iter()
            .map(|s| Selection::caret(s.head + 1))
            .collect();
        selections.set_all(next);
        return Ok(true);
    }
    let Some(close) = closing_for(typed) else {
        return Ok(false);
    };
    if !all_carets {
        // Surround each selection.
        let mut changes = Vec::new();
        let mut next = Vec::new();
        let mut shift = 0;
        for s in selections.selections() {
            changes.push(TextChange {
                start_char: s.start(),
                end_char: s.start(),
                inserted: typed.to_string(),
            });
            changes.push(TextChange {
                start_char: s.end(),
                end_char: s.end(),
                inserted: close.to_string(),
            });
            next.push(Selection {
                anchor: s.start() + shift + 1,
                head: s.end() + shift + 1,
            });
            shift += 2;
        }
        doc.edit_many(changes)?;
        selections.set_all(next);
        return Ok(true);
    }
    let pair_allowed = selections.selections().iter().all(|s| {
        let next = doc.char_at(s.head);
        let next_ok =
            next.is_none_or(|c| c.is_whitespace() || matches!(c, ')' | ']' | '}' | ',' | ';'));
        if typed == '"' || typed == '\'' || typed == '`' {
            let previous = s.head.checked_sub(1).and_then(|i| doc.char_at(i));
            // Lifetimes like `'a` and contractions should not pair.
            next_ok && !previous.is_some_and(|c| is_word(c) || c == '&' || c == '<')
        } else {
            next_ok
        }
    });
    if !pair_allowed {
        return Ok(false);
    }
    let mut changes = Vec::new();
    let mut next = Vec::new();
    for (index, s) in selections.selections().iter().enumerate() {
        changes.push(TextChange {
            start_char: s.head,
            end_char: s.head,
            inserted: format!("{typed}{close}"),
        });
        next.push(Selection::caret(s.head + 1 + index * 2));
    }
    doc.edit_many(changes)?;
    selections.set_all(next);
    Ok(true)
}

/// Backspace that removes an empty auto-inserted pair such as `()`.
pub fn backspace(doc: &mut TextDocument, selections: &mut SelectionSet) -> Result<(), TextError> {
    if selections.selections().iter().all(|s| {
        s.is_caret()
            && s.head > 0
            && doc
                .char_at(s.head - 1)
                .and_then(closing_for)
                .is_some_and(|close| doc.char_at(s.head) == Some(close))
    }) {
        let changes: Vec<TextChange> = selections
            .selections()
            .iter()
            .map(|s| TextChange {
                start_char: s.head - 1,
                end_char: s.head + 1,
                inserted: String::new(),
            })
            .collect();
        let next: Vec<Selection> = selections
            .selections()
            .iter()
            .enumerate()
            .map(|(index, s)| Selection::caret(s.head - 1 - index * 2))
            .collect();
        doc.edit_many(changes)?;
        selections.set_all(next);
        return Ok(());
    }
    selections.backspace(doc)
}

/// Enter with indentation: keeps the line indent, indents after an opening
/// bracket, and splits an empty `{}` pair onto three lines.
pub fn newline(doc: &mut TextDocument, selections: &mut SelectionSet) -> Result<(), TextError> {
    let mut changes = Vec::new();
    let mut next = Vec::new();
    let mut shift: isize = 0;
    for s in selections.selections() {
        let line = doc.char_to_line(s.start());
        let indent = indentation(&line_text(doc, line));
        let before = s.start().checked_sub(1).and_then(|i| doc.char_at(i));
        let after = doc.char_at(s.end());
        let opens = before.is_some_and(|c| matches!(c, '{' | '(' | '['));
        let (inserted, caret) = if opens {
            let inner = format!("\n{indent}{INDENT}");
            let caret = inner.chars().count();
            if after.is_some_and(|c| before.and_then(closing_for) == Some(c)) {
                (format!("{inner}\n{indent}"), caret)
            } else {
                (inner, caret)
            }
        } else {
            let text = format!("\n{indent}");
            let caret = text.chars().count();
            (text, caret)
        };
        let removed = (s.end() - s.start()) as isize;
        next.push(Selection::caret(
            s.start().saturating_add_signed(shift) + caret,
        ));
        shift += inserted.chars().count() as isize - removed;
        changes.push(TextChange {
            start_char: s.start(),
            end_char: s.end(),
            inserted,
        });
    }
    doc.edit_many(changes)?;
    selections.set_all(next);
    Ok(())
}

/// Start of the previous word (⌥← / Ctrl+←).
pub fn word_left(doc: &TextDocument, position: usize) -> usize {
    let mut position = position;
    while position > 0 && doc.char_at(position - 1).is_some_and(|c| c.is_whitespace()) {
        position -= 1;
    }
    if position > 0 && doc.char_at(position - 1).is_some_and(is_word) {
        while position > 0 && doc.char_at(position - 1).is_some_and(is_word) {
            position -= 1;
        }
    } else {
        position = position.saturating_sub(1);
    }
    position
}

/// Start of the next word (Ctrl+→ on Windows/Linux; ⌥→ ends at the word end on macOS).
pub fn word_right(doc: &TextDocument, position: usize) -> usize {
    let len = doc.len_chars();
    let mut position = position;
    if position < len && doc.char_at(position).is_some_and(is_word) {
        while position < len && doc.char_at(position).is_some_and(is_word) {
            position += 1;
        }
    } else if position < len && !doc.char_at(position).is_some_and(|c| c.is_whitespace()) {
        position += 1;
    }
    while position < len
        && doc
            .char_at(position)
            .is_some_and(|c| c.is_whitespace() && c != '\n')
    {
        position += 1;
    }
    position
}

/// Word range around `position` (double-click selection).
pub fn word_at(doc: &TextDocument, position: usize) -> (usize, usize) {
    let len = doc.len_chars();
    let probe = if doc.char_at(position).is_some_and(is_word) {
        position
    } else if position > 0 && doc.char_at(position - 1).is_some_and(is_word) {
        position - 1
    } else {
        return (position, (position + 1).min(len));
    };
    let mut start = probe;
    while start > 0 && doc.char_at(start - 1).is_some_and(is_word) {
        start -= 1;
    }
    let mut end = probe;
    while end < len && doc.char_at(end).is_some_and(is_word) {
        end += 1;
    }
    (start, end)
}

/// If `position` holds a bracket, returns it with its matching partner.
/// The scan is bounded so huge files stay responsive.
pub fn matching_bracket(doc: &TextDocument, position: usize) -> Option<(usize, usize)> {
    const LIMIT: usize = 200_000;
    let c = doc.char_at(position)?;
    let (open, close, forward) = match c {
        '(' => ('(', ')', true),
        '[' => ('[', ']', true),
        '{' => ('{', '}', true),
        ')' => ('(', ')', false),
        ']' => ('[', ']', false),
        '}' => ('{', '}', false),
        _ => return None,
    };
    let mut depth = 0usize;
    let mut index = position;
    for _ in 0..LIMIT {
        let current = doc.char_at(index)?;
        if current == open {
            if forward {
                depth += 1;
            } else {
                depth = depth.checked_sub(1)?;
            }
        } else if current == close {
            if forward {
                depth = depth.checked_sub(1)?;
            } else {
                depth += 1;
            }
        }
        if depth == 0 {
            return Some((position.min(index), position.max(index)));
        }
        if forward {
            index += 1;
        } else {
            index = index.checked_sub(1)?;
        }
    }
    None
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FindOptions {
    pub match_case: bool,
    pub whole_words: bool,
}

/// All non-overlapping matches of `query` as character ranges.
pub fn find_all(text: &str, query: &str, options: FindOptions) -> Vec<(usize, usize)> {
    if query.is_empty() {
        return Vec::new();
    }
    let fold = |s: &str| -> Vec<char> {
        if options.match_case {
            s.chars().collect()
        } else {
            // Per-char lowercase keeps offsets aligned with the original text.
            s.chars()
                .map(|c| c.to_lowercase().next().unwrap_or(c))
                .collect()
        }
    };
    let haystack = fold(text);
    let needle = fold(query);
    let mut matches = Vec::new();
    let mut index = 0;
    while index + needle.len() <= haystack.len() {
        if haystack[index..index + needle.len()] == needle[..] {
            let end = index + needle.len();
            let bounded = !options.whole_words
                || ((index == 0 || !is_word(haystack[index - 1]))
                    && (end == haystack.len() || !is_word(haystack[end])));
            if bounded {
                matches.push((index, end));
                index = end;
                continue;
            }
        }
        index += 1;
    }
    matches
}

/// Replaces the given match ranges with `replacement` in one undo step.
pub fn replace_ranges(
    doc: &mut TextDocument,
    ranges: &[(usize, usize)],
    replacement: &str,
) -> Result<(), TextError> {
    doc.edit_many(ranges.iter().map(|&(start, end)| TextChange {
        start_char: start,
        end_char: end,
        inserted: replacement.to_owned(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(text: &str, selection: Selection) -> (TextDocument, SelectionSet) {
        let doc = TextDocument::new(text);
        let mut selections = SelectionSet::default();
        selections.set_single(selection);
        (doc, selections)
    }

    #[test]
    fn copy_without_selection_copies_the_line() {
        let (doc, selections) = setup("one\ntwo\n", Selection::caret(5));
        assert_eq!(copy_text(&doc, &selections), ("two\n".into(), true));
        let (doc, selections) = setup("one\ntwo\n", Selection { anchor: 1, head: 6 });
        assert_eq!(copy_text(&doc, &selections), ("ne\ntw".into(), false));
    }

    #[test]
    fn cut_line_and_paste_it_above() {
        let (mut doc, mut selections) = setup("one\ntwo\nthree", Selection::caret(1));
        let (text, whole) = copy_text(&doc, &selections);
        assert_eq!(cut(&mut doc, &mut selections).unwrap(), "one\n");
        assert_eq!(doc.text(), "two\nthree");
        selections.set_single(Selection::caret(6));
        paste(&mut doc, &mut selections, &text, whole).unwrap();
        assert_eq!(doc.text(), "two\none\nthree");
        assert!(doc.undo());
        assert_eq!(doc.text(), "two\nthree");
    }

    #[test]
    fn delete_last_line_removes_preceding_newline() {
        let (mut doc, mut selections) = setup("a\nb", Selection::caret(3));
        delete_lines(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "a");
        let (mut doc, mut selections) = setup("a\nb\nc\n", Selection::caret(2));
        delete_lines(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "a\nc\n");
        assert_eq!(selections.primary().head, 2);
    }

    #[test]
    fn duplicates_lines_and_selections() {
        let (mut doc, mut selections) = setup("let x = 1;\nend", Selection::caret(4));
        duplicate(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "let x = 1;\nlet x = 1;\nend");
        assert_eq!(selections.primary().head, 15);
        let (mut doc, mut selections) = setup("last", Selection::caret(0));
        duplicate(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "last\nlast");
        let (mut doc, mut selections) = setup("ab", Selection { anchor: 0, head: 1 });
        duplicate(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "aab");
        assert_eq!(selections.primary(), Selection { anchor: 1, head: 2 });
    }

    #[test]
    fn toggles_line_comments() {
        let (mut doc, mut selections) = setup("    let a = 1;\nlet b = 2;", Selection::caret(4));
        toggle_line_comment(&mut doc, &mut selections, "//").unwrap();
        assert_eq!(doc.text(), "    // let a = 1;\nlet b = 2;");
        assert_eq!(doc.char_to_line(selections.primary().head), 1);
        selections.set_single(Selection::caret(6));
        toggle_line_comment(&mut doc, &mut selections, "//").unwrap();
        assert_eq!(doc.text(), "    let a = 1;\nlet b = 2;");
        let (mut doc, mut selections) = setup("a\n  b\n", Selection { anchor: 0, head: 5 });
        toggle_line_comment(&mut doc, &mut selections, "//").unwrap();
        assert_eq!(doc.text(), "// a\n//   b\n");
    }

    #[test]
    fn moves_lines_up_and_down() {
        let (mut doc, mut selections) = setup("a\nbb\nc\n", Selection::caret(3));
        move_lines(&mut doc, &mut selections, true).unwrap();
        assert_eq!(doc.text(), "bb\na\nc\n");
        assert_eq!(selections.primary().head, 1);
        move_lines(&mut doc, &mut selections, false).unwrap();
        move_lines(&mut doc, &mut selections, false).unwrap();
        assert_eq!(doc.text(), "a\nc\nbb\n");
        // Already at the bottom: no-op.
        move_lines(&mut doc, &mut selections, false).unwrap();
        assert_eq!(doc.text(), "a\nc\nbb\n");
        let (mut doc, mut selections) = setup("x\ny", Selection::caret(0));
        move_lines(&mut doc, &mut selections, false).unwrap();
        assert_eq!(doc.text(), "y\nx");
    }

    #[test]
    fn indents_and_unindents_selected_lines() {
        let (mut doc, mut selections) = setup("a\n  b\n", Selection { anchor: 0, head: 5 });
        indent_lines(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "    a\n      b\n");
        unindent_lines(&mut doc, &mut selections).unwrap();
        unindent_lines(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "a\nb\n");
    }

    #[test]
    fn pairs_brackets_and_overtypes_closers() {
        let (mut doc, mut selections) = setup("f", Selection::caret(1));
        assert!(type_char(&mut doc, &mut selections, '(').unwrap());
        assert_eq!(doc.text(), "f()");
        assert_eq!(selections.primary().head, 2);
        assert!(type_char(&mut doc, &mut selections, ')').unwrap());
        assert_eq!(selections.primary().head, 3);
        let (mut doc, mut selections) = setup("()", Selection::caret(1));
        backspace(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "");
        let (mut doc, mut selections) = setup("word", Selection { anchor: 0, head: 4 });
        type_char(&mut doc, &mut selections, '"').unwrap();
        assert_eq!(doc.text(), "\"word\"");
        // No pairing before a word character or for lifetimes.
        let (mut doc, mut selections) = setup("x", Selection::caret(0));
        assert!(!type_char(&mut doc, &mut selections, '(').unwrap());
        let (mut doc, mut selections) = setup("&", Selection::caret(1));
        assert!(!type_char(&mut doc, &mut selections, '\'').unwrap());
    }

    #[test]
    fn smart_enter_indents_inside_braces() {
        let (mut doc, mut selections) = setup("    fn a() {}", Selection::caret(12));
        newline(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "    fn a() {\n        \n    }");
        assert_eq!(selections.primary().head, 21);
        let (mut doc, mut selections) = setup("  x", Selection::caret(3));
        newline(&mut doc, &mut selections).unwrap();
        assert_eq!(doc.text(), "  x\n  ");
    }

    #[test]
    fn word_navigation_and_selection() {
        let doc = TextDocument::new("let foo_bar = baz(1);");
        assert_eq!(word_right(&doc, 0), 4);
        assert_eq!(word_right(&doc, 4), 12);
        assert_eq!(word_left(&doc, 12), 4);
        assert_eq!(word_left(&doc, 4), 0);
        assert_eq!(word_at(&doc, 6), (4, 11));
        assert_eq!(word_at(&doc, 11), (4, 11));
    }

    #[test]
    fn matches_brackets_in_both_directions() {
        let doc = TextDocument::new("f(a[1], (b)) }");
        assert_eq!(matching_bracket(&doc, 1), Some((1, 11)));
        assert_eq!(matching_bracket(&doc, 11), Some((1, 11)));
        assert_eq!(matching_bracket(&doc, 3), Some((3, 5)));
        assert_eq!(matching_bracket(&doc, 13), None);
        assert_eq!(matching_bracket(&doc, 0), None);
    }

    #[test]
    fn finds_with_case_and_word_options() {
        let text = "Foo foo food ÄÖ äö";
        assert_eq!(find_all(text, "foo", FindOptions::default()).len(), 3);
        let case = FindOptions {
            match_case: true,
            whole_words: false,
        };
        assert_eq!(find_all(text, "foo", case), vec![(4, 7), (8, 11)]);
        let words = FindOptions {
            match_case: false,
            whole_words: true,
        };
        assert_eq!(find_all(text, "foo", words), vec![(0, 3), (4, 7)]);
        assert_eq!(
            find_all(text, "äö", FindOptions::default()),
            vec![(13, 15), (16, 18)]
        );
        let mut doc = TextDocument::new(text);
        let ranges = find_all(text, "foo", words);
        replace_ranges(&mut doc, &ranges, "x").unwrap();
        assert_eq!(doc.text(), "x x food ÄÖ äö");
    }
}
