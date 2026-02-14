//! Text editing operations for the editor.

use helix_core::history::UndoKind;
use helix_core::RopeSlice;
use helix_view::document::Mode;
use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// Compute the line range covered by a selection.
///
/// When the selection end falls exactly at the start of a line
/// (e.g., after `x` which sets head = start of next line),
/// that line is excluded — only the lines actually *within* the selection are returned.
fn selected_line_range(text: RopeSlice, anchor: usize, head: usize) -> (usize, usize) {
    let sel_start = anchor.min(head);
    let sel_end = anchor.max(head);
    let start_line = text.char_to_line(sel_start);
    let end_line = if sel_end > 0 && sel_end == text.line_to_char(text.char_to_line(sel_end)) {
        // sel_end is at the very start of a line → exclude that line
        text.char_to_line(sel_end).saturating_sub(1)
    } else {
        text.char_to_line(sel_end)
    };
    (start_line, end_line)
}

/// Extract the leading whitespace (indentation) of a line.
fn extract_indentation(text: RopeSlice, line: usize) -> String {
    text.line(line)
        .chars()
        .take_while(|c| c.is_whitespace() && *c != '\n')
        .collect()
}

/// Extension trait for text editing operations.
pub trait EditingOps {
    fn insert_char(&mut self, doc_id: DocumentId, view_id: ViewId, c: char);
    fn insert_newline(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn insert_tab(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_char_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_char_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn open_line_below(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn open_line_above(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn undo(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn redo(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn toggle_line_comment(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn toggle_block_comment(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_to_line_start(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn indent_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn unindent_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn earlier(&mut self, steps: usize);
    fn later(&mut self, steps: usize);
    fn change_selection(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn change_selection_noyank(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn replace_char(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char);
    fn join_lines(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn toggle_case(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn to_lowercase(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn to_uppercase(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn surround_add(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char);
    fn surround_delete(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char);
    fn surround_replace(&mut self, doc_id: DocumentId, view_id: ViewId, old: char, new: char);
    fn delete_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn kill_to_line_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn add_newline_below(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn add_newline_above(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn increment(&mut self, doc_id: DocumentId, view_id: ViewId, amount: i64);
    fn format_document(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn format_selections(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn align_selections(&mut self, doc_id: DocumentId, view_id: ViewId);
}

impl EditingOps for EditorContext {
    fn insert_char(&mut self, doc_id: DocumentId, view_id: ViewId, c: char) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone().cursors(text);

        let transaction =
            helix_core::Transaction::insert(doc.text(), &selection, c.to_string().into());
        doc.apply(&transaction, view_id);
    }

    fn insert_newline(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);

        let indent = extract_indentation(text, line);

        // Insert newline + indentation at cursor position
        let insert_selection = helix_core::Selection::point(cursor);
        let insert_text = format!("\n{indent}");
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, insert_text.into());
        doc.apply(&transaction, view_id);

        // Position cursor at end of indentation on the new line
        let new_cursor_pos = cursor + 1 + indent.len();
        doc.set_selection(view_id, helix_core::Selection::point(new_cursor_pos));
    }

    fn insert_tab(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone().cursors(text);

        // Use the document's indent style (could be tab or spaces)
        let indent = doc.indent_style.as_str();

        let transaction = helix_core::Transaction::insert(doc.text(), &selection, indent.into());
        doc.apply(&transaction, view_id);
    }

    fn delete_char_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        if cursor == 0 {
            return;
        }

        let ranges = std::iter::once((cursor - 1, cursor));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    fn delete_char_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        if cursor >= text.len_chars() {
            return;
        }

        let ranges = std::iter::once((cursor, cursor + 1));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    fn open_line_below(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_end = text.line_to_char(line) + text.line(line).len_chars();

        let indent = extract_indentation(text, line);

        // Move to end of line (before newline character)
        let insert_pos = line_end.saturating_sub(1);
        let insert_selection = helix_core::Selection::point(insert_pos);

        // Insert newline + indentation
        let insert_text = format!("\n{}", indent);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, insert_text.into());
        doc.apply(&transaction, view_id);

        // Position cursor at end of indentation on the new line
        let new_cursor_pos = insert_pos + 1 + indent.len();
        doc.set_selection(view_id, helix_core::Selection::point(new_cursor_pos));
    }

    fn open_line_above(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);

        let indent = extract_indentation(text, line);

        // Insert indentation + newline at start of current line
        let insert_selection = helix_core::Selection::point(line_start);
        let insert_text = format!("{}\n", indent);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, insert_text.into());
        doc.apply(&transaction, view_id);

        // Move cursor to the end of indentation on the new line (which is now at line_start)
        let new_cursor_pos = line_start + indent.len();
        doc.set_selection(view_id, helix_core::Selection::point(new_cursor_pos));
    }

    fn undo(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let view = self.editor.tree.get_mut(view_id);
        let doc = self.editor.documents.get_mut(&doc_id).expect("doc exists");
        if !doc.undo(view) {
            log::info!("Already at oldest change");
        }
    }

    fn redo(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let view = self.editor.tree.get_mut(view_id);
        let doc = self.editor.documents.get_mut(&doc_id).expect("doc exists");
        if !doc.redo(view) {
            log::info!("Already at newest change");
        }
    }

    fn toggle_line_comment(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");

        // Get the comment token from the language configuration
        let comment_token = doc
            .language_config()
            .and_then(|config| config.comment_tokens.as_ref())
            .and_then(|tokens| tokens.first())
            .map(String::as_str);

        let selection = doc.selection(view_id).clone();

        // Use helix_core::comment::toggle_line_comments
        let transaction =
            helix_core::comment::toggle_line_comments(doc.text(), &selection, comment_token);

        doc.apply(&transaction, view_id);
    }

    fn toggle_block_comment(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");

        // Get the block comment tokens from the language configuration
        let block_tokens = doc
            .language_config()
            .and_then(|config| config.block_comment_tokens.as_ref());

        let Some(tokens) = block_tokens else {
            log::info!("No block comment tokens configured for this language");
            return;
        };

        let selection = doc.selection(view_id).clone();

        // Use helix_core::comment::toggle_block_comments
        let transaction =
            helix_core::comment::toggle_block_comments(doc.text(), &selection, tokens);

        doc.apply(&transaction, view_id);
    }

    /// Delete word backward (Ctrl+w in insert mode).
    fn delete_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);

        if cursor == 0 {
            return;
        }

        // Skip whitespace backward
        let mut start = cursor;
        while start > 0 && text.char(start - 1).is_whitespace() && text.char(start - 1) != '\n' {
            start -= 1;
        }
        // Skip word characters backward
        while start > 0 && !text.char(start - 1).is_whitespace() {
            start -= 1;
        }

        // If we only skipped whitespace and hit a newline or start, delete at least one char
        if start == cursor {
            start = cursor - 1;
        }

        let ranges = std::iter::once((start, cursor));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    /// Delete to line start (Ctrl+u in insert mode).
    fn delete_to_line_start(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);

        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);

        if cursor == line_start {
            return;
        }

        let ranges = std::iter::once((line_start, cursor));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    /// Indent the current line or selection.
    fn indent_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let indent = doc.indent_style.as_str().to_string();

        let primary = selection.primary();
        let (start_line, end_line) = selected_line_range(text, primary.anchor, primary.head);

        // Build a transaction that inserts indent at the start of each line
        let mut changes = Vec::new();
        for line in start_line..=end_line {
            let line_start = text.line_to_char(line);
            changes.push((line_start, line_start, Some(indent.clone().into())));
        }

        let transaction = helix_core::Transaction::change(doc.text(), changes.into_iter());
        doc.apply(&transaction, view_id);
    }

    /// Unindent the current line or selection.
    fn unindent_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let indent_width = doc.indent_style.indent_width(doc.tab_width());

        let primary = selection.primary();
        let (start_line, end_line) = selected_line_range(text, primary.anchor, primary.head);

        // Build a transaction that removes leading whitespace (up to indent_width)
        let mut changes = Vec::new();
        for line in start_line..=end_line {
            let line_start = text.line_to_char(line);
            let line_text = text.line(line);
            let mut chars_to_remove = 0;
            for ch in line_text.chars() {
                if chars_to_remove >= indent_width {
                    break;
                }
                match ch {
                    ' ' => chars_to_remove += 1,
                    '\t' => {
                        chars_to_remove += 1;
                        break; // Tab counts as one indent
                    }
                    _ => break,
                }
            }
            if chars_to_remove > 0 {
                changes.push((line_start, line_start + chars_to_remove, None));
            }
        }

        if !changes.is_empty() {
            let transaction = helix_core::Transaction::change(doc.text(), changes.into_iter());
            doc.apply(&transaction, view_id);
        }
    }

    /// Undo to an earlier state (multiple steps).
    fn earlier(&mut self, steps: usize) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get_mut(view_id);
        let doc_id = view.doc;
        let doc = self.editor.documents.get_mut(&doc_id).expect("doc exists");

        if !doc.earlier(view, UndoKind::Steps(steps)) {
            log::info!("Already at oldest change");
        }
    }

    /// Redo to a later state (multiple steps).
    fn later(&mut self, steps: usize) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get_mut(view_id);
        let doc_id = view.doc;
        let doc = self.editor.documents.get_mut(&doc_id).expect("doc exists");

        if !doc.later(view, UndoKind::Steps(steps)) {
            log::info!("Already at newest change");
        }
    }

    /// Change selection: yank to register, delete selected text and enter insert mode.
    fn change_selection(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let register = self.take_register();

        // Extract selection data (immutable borrow)
        let (selected_text, from, to) = {
            let doc = self.editor.document(doc_id).expect("doc exists");
            let text = doc.text().slice(..);
            let primary = doc.selection(view_id).primary();
            let selected: String = text.slice(primary.from()..primary.to()).into();
            (selected, primary.from(), primary.to())
        };

        // Yank to target register (skip for '_' black hole)
        if register != '_' {
            if let Err(e) = self
                .editor
                .registers
                .write(register, vec![selected_text.clone()])
            {
                log::warn!("Failed to write to register '{}': {e}", register);
            }
            if register == '+' {
                self.clipboard.clone_from(&selected_text);
            }
        }

        if from < to {
            let doc = self.editor.document_mut(doc_id).expect("doc exists");
            let ranges = std::iter::once((from, to));
            let transaction = helix_core::Transaction::delete(doc.text(), ranges);
            doc.apply(&transaction, view_id);
        }

        self.editor.mode = Mode::Insert;
    }

    /// Change selection without yanking (Alt-c): delete selected text and enter insert mode.
    fn change_selection_noyank(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let (from, to) = {
            let doc = self.editor.document(doc_id).expect("doc exists");
            let primary = doc.selection(view_id).primary();
            (primary.from(), primary.to())
        };

        if from < to {
            let doc = self.editor.document_mut(doc_id).expect("doc exists");
            let ranges = std::iter::once((from, to));
            let transaction = helix_core::Transaction::delete(doc.text(), ranges);
            doc.apply(&transaction, view_id);
        }

        self.editor.mode = Mode::Insert;
    }

    /// Replace each character in selection with the given character.
    fn replace_char(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let transaction =
            helix_core::Transaction::change_by_selection(doc.text(), &selection, |range| {
                let replacement: String = text
                    .slice(range.from()..range.to())
                    .chars()
                    .map(|c| if c == '\n' || c == '\r' { c } else { ch })
                    .collect();
                (range.from(), range.to(), Some(replacement.into()))
            });

        doc.apply(&transaction, view_id);
    }

    /// Join lines: replace newlines + leading whitespace with a single space.
    fn join_lines(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();

        let (start_line, end_line) = selected_line_range(text, primary.anchor, primary.head);

        // If only one line, join with the next line
        let end_line = if start_line == end_line {
            (start_line + 1).min(text.len_lines().saturating_sub(1))
        } else {
            end_line
        };

        if start_line >= end_line {
            return;
        }

        // Build changes: replace each line break (+ leading whitespace on next line) with a space
        let mut changes = Vec::new();
        for line in start_line..end_line {
            let line_end = text.line_to_char(line) + text.line(line).len_chars();
            // Find end of leading whitespace on next line
            let next_line_start = text.line_to_char(line + 1);
            let mut ws_end = next_line_start;
            for c in text.line(line + 1).chars() {
                if c.is_whitespace() && c != '\n' {
                    ws_end += 1;
                } else {
                    break;
                }
            }
            // Replace from before the newline to end of whitespace with a single space
            let join_start = line_end.saturating_sub(1); // the newline char
            changes.push((join_start, ws_end, Some(" ".into())));
        }

        let transaction = helix_core::Transaction::change(doc.text(), changes.into_iter());
        doc.apply(&transaction, view_id);
    }

    /// Toggle case of each character in selection.
    fn toggle_case(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let transaction =
            helix_core::Transaction::change_by_selection(doc.text(), &selection, |range| {
                let toggled: String = text
                    .slice(range.from()..range.to())
                    .chars()
                    .map(|c| {
                        if c.is_uppercase() {
                            c.to_lowercase().next().unwrap_or(c)
                        } else {
                            c.to_uppercase().next().unwrap_or(c)
                        }
                    })
                    .collect();
                (range.from(), range.to(), Some(toggled.into()))
            });

        doc.apply(&transaction, view_id);
    }

    /// Convert selection to lowercase.
    fn to_lowercase(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let transaction =
            helix_core::Transaction::change_by_selection(doc.text(), &selection, |range| {
                let lower: String = text
                    .slice(range.from()..range.to())
                    .chars()
                    .flat_map(char::to_lowercase)
                    .collect();
                (range.from(), range.to(), Some(lower.into()))
            });

        doc.apply(&transaction, view_id);
    }

    /// Convert selection to uppercase.
    fn to_uppercase(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let transaction =
            helix_core::Transaction::change_by_selection(doc.text(), &selection, |range| {
                let upper: String = text
                    .slice(range.from()..range.to())
                    .chars()
                    .flat_map(char::to_uppercase)
                    .collect();
                (range.from(), range.to(), Some(upper.into()))
            });

        doc.apply(&transaction, view_id);
    }

    /// Add surround pair around selection.
    fn surround_add(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char) {
        let (open, close) = surround_pair(ch);

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let selection = doc.selection(view_id).clone();

        let transaction =
            helix_core::Transaction::change_by_selection(doc.text(), &selection, |range| {
                let from = range.from();
                let to = range.to();
                let replacement = format!("{open}{}{close}", doc.text().slice(from..to));
                (from, to, Some(replacement.into()))
            });

        doc.apply(&transaction, view_id);
    }

    /// Delete surround pair.
    fn surround_delete(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let surround_result =
            helix_core::surround::get_surround_pos(doc.syntax(), text, &selection, Some(ch), 1);

        let Ok(positions) = surround_result else {
            return;
        };

        // Positions come in pairs: [open, close, open, close, ...]
        let mut changes: Vec<(usize, usize, Option<helix_core::Tendril>)> = Vec::new();
        for pair in positions.chunks(2) {
            if pair.len() == 2 {
                changes.push((pair[1], pair[1] + 1, None)); // delete close first (higher pos)
                changes.push((pair[0], pair[0] + 1, None)); // delete open
            }
        }
        // Sort by position descending so we don't invalidate offsets
        changes.sort_by(|a, b| b.0.cmp(&a.0));
        // But Transaction expects ascending order
        changes.reverse();

        if !changes.is_empty() {
            let transaction = helix_core::Transaction::change(doc.text(), changes.into_iter());
            doc.apply(&transaction, view_id);
        }
    }

    /// Replace surround pair.
    fn surround_replace(&mut self, doc_id: DocumentId, view_id: ViewId, old: char, new: char) {
        let (new_open, new_close) = surround_pair(new);

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let surround_result =
            helix_core::surround::get_surround_pos(doc.syntax(), text, &selection, Some(old), 1);

        let Ok(positions) = surround_result else {
            return;
        };

        let mut changes: Vec<(usize, usize, Option<helix_core::Tendril>)> = Vec::new();
        for pair in positions.chunks(2) {
            if pair.len() == 2 {
                changes.push((pair[0], pair[0] + 1, Some(new_open.to_string().into())));
                changes.push((pair[1], pair[1] + 1, Some(new_close.to_string().into())));
            }
        }
        // Transaction expects ascending order
        changes.sort_by(|a, b| a.0.cmp(&b.0));

        if !changes.is_empty() {
            let transaction = helix_core::Transaction::change(doc.text(), changes.into_iter());
            doc.apply(&transaction, view_id);
        }
    }

    /// Delete word forward from cursor (Alt+d in insert mode).
    fn delete_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);
        let len = text.len_chars();

        if cursor >= len {
            return;
        }

        // Skip non-whitespace (word chars) forward
        let mut end = cursor;
        while end < len && !text.char(end).is_whitespace() {
            end += 1;
        }
        // Skip whitespace forward
        while end < len && text.char(end).is_whitespace() && text.char(end) != '\n' {
            end += 1;
        }

        // If we didn't move past any word chars, at least delete whitespace
        if end == cursor {
            while end < len && text.char(end).is_whitespace() && text.char(end) != '\n' {
                end += 1;
            }
            // Still nothing? Delete at least one char
            if end == cursor {
                end = cursor + 1;
            }
        }

        let ranges = std::iter::once((cursor, end));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    /// Kill to line end from cursor (Ctrl+k in insert mode).
    fn kill_to_line_end(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);

        let line = text.char_to_line(cursor);
        let line_end = text.line_to_char(line) + text.line(line).len_chars();
        // Exclude the newline character itself
        let end = if line_end > 0 && text.char(line_end.saturating_sub(1)) == '\n' {
            line_end.saturating_sub(1)
        } else {
            line_end
        };

        if cursor >= end {
            return;
        }

        let ranges = std::iter::once((cursor, end));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    /// Add a newline below current line without entering insert mode (`] Space`).
    fn add_newline_below(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_end = text.line_to_char(line) + text.line(line).len_chars();

        // Insert newline at end of current line (before the existing newline, or at end)
        let insert_pos = if line_end > 0 && text.char(line_end.saturating_sub(1)) == '\n' {
            line_end.saturating_sub(1)
        } else {
            line_end
        };

        let insert_selection = helix_core::Selection::point(insert_pos);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, "\n".into());
        doc.apply(&transaction, view_id);
    }

    /// Add a newline above current line without entering insert mode (`[ Space`).
    fn add_newline_above(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);

        let insert_selection = helix_core::Selection::point(line_start);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, "\n".into());
        doc.apply(&transaction, view_id);
    }

    /// Increment or decrement numbers/dates in selections.
    fn increment(&mut self, doc_id: DocumentId, view_id: ViewId, amount: i64) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let mut changes = Vec::new();
        let mut new_ranges = helix_core::SmallVec::<[helix_core::Range; 1]>::new();
        let mut cumulative_diff: i128 = 0;

        for range in selection.iter() {
            let selected_text: std::borrow::Cow<str> = range.fragment(text);
            let new_from = ((range.from() as i128) + cumulative_diff) as usize;
            let incremented = [
                helix_core::increment::integer,
                helix_core::increment::date_time,
            ]
            .iter()
            .find_map(|incrementor| incrementor(selected_text.as_ref(), amount));

            match incremented {
                None => {
                    let new_range = helix_core::Range::new(
                        new_from,
                        ((range.to() as i128) + cumulative_diff) as usize,
                    );
                    new_ranges.push(new_range);
                }
                Some(new_text) => {
                    let new_range = helix_core::Range::new(new_from, new_from + new_text.len());
                    cumulative_diff += new_text.len() as i128 - selected_text.len() as i128;
                    new_ranges.push(new_range);
                    changes.push((range.from(), range.to(), Some(new_text.into())));
                }
            }
        }

        if !changes.is_empty() {
            let new_selection = helix_core::Selection::new(new_ranges, selection.primary_index());
            let transaction = helix_core::Transaction::change(doc.text(), changes.into_iter());
            let transaction = transaction.with_selection(new_selection);
            doc.apply(&transaction, view_id);
        }
    }

    /// Format the entire document via LSP.
    fn format_document(&mut self, doc_id: DocumentId, _view_id: ViewId) {
        use helix_core::syntax::config::LanguageServerFeature;
        use helix_lsp::lsp;

        let doc = self.editor.document(doc_id).expect("doc exists");

        let Some(language_server) = doc
            .language_servers_with_feature(LanguageServerFeature::Format)
            .next()
        else {
            log::info!("No configured language server supports formatting");
            return;
        };

        let offset_encoding = language_server.offset_encoding();

        let future = language_server.text_document_formatting(
            doc.identifier(),
            lsp::FormattingOptions {
                tab_size: doc.tab_width() as u32,
                insert_spaces: matches!(
                    doc.indent_style,
                    helix_core::indent::IndentStyle::Spaces(_)
                ),
                ..Default::default()
            },
            None,
        );

        let Some(future) = future else {
            log::info!("Language server does not support formatting");
            return;
        };

        let text = doc.text().clone();
        let tx = self.command_tx.clone();

        tokio::spawn(async move {
            match future.await {
                Ok(Some(edits)) => {
                    let transaction = helix_lsp::util::generate_transaction_from_edits(
                        &text,
                        edits,
                        offset_encoding,
                    );
                    let _ = tx.send(crate::state::EditorCommand::LspResponse(
                        crate::lsp::LspResponse::FormatResult { transaction },
                    ));
                }
                Ok(None) => log::info!("No formatting edits returned"),
                Err(e) => log::error!("Format document failed: {e}"),
            }
        });
    }

    /// Format selections via LSP range formatting.
    fn format_selections(&mut self, doc_id: DocumentId, view_id: ViewId) {
        use helix_core::syntax::config::LanguageServerFeature;
        use helix_lsp::lsp;

        let doc = self.editor.document(doc_id).expect("doc exists");

        if doc.selection(view_id).len() != 1 {
            log::info!("format_selections only supports a single selection");
            return;
        }

        let Some(language_server) = doc
            .language_servers_with_feature(LanguageServerFeature::Format)
            .find(|ls| {
                matches!(
                    ls.capabilities().document_range_formatting_provider,
                    Some(lsp::OneOf::Left(true) | lsp::OneOf::Right(_))
                )
            })
        else {
            log::info!("No configured language server supports range formatting");
            return;
        };

        let offset_encoding = language_server.offset_encoding();
        let selection = doc.selection(view_id).clone();
        let range =
            helix_lsp::util::range_to_lsp_range(doc.text(), selection.primary(), offset_encoding);

        let future = language_server.text_document_range_formatting(
            doc.identifier(),
            range,
            lsp::FormattingOptions {
                tab_size: doc.tab_width() as u32,
                insert_spaces: matches!(
                    doc.indent_style,
                    helix_core::indent::IndentStyle::Spaces(_)
                ),
                ..Default::default()
            },
            None,
        );

        let Some(future) = future else {
            log::info!("Language server does not support range formatting");
            return;
        };

        let text = doc.text().clone();
        let tx = self.command_tx.clone();

        tokio::spawn(async move {
            match future.await {
                Ok(Some(edits)) => {
                    let transaction = helix_lsp::util::generate_transaction_from_edits(
                        &text,
                        edits,
                        offset_encoding,
                    );
                    let _ = tx.send(crate::state::EditorCommand::LspResponse(
                        crate::lsp::LspResponse::FormatResult { transaction },
                    ));
                }
                Ok(None) => log::info!("No formatting edits returned"),
                Err(e) => log::error!("Format selections failed: {e}"),
            }
        });
    }

    /// Align selections by inserting spaces to align cursors in columns.
    #[allow(deprecated)]
    fn align_selections(&mut self, doc_id: DocumentId, view_id: ViewId) {
        use helix_core::visual_coords_at_pos;

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let tab_width = doc.tab_width();

        let mut column_widths: Vec<Vec<(usize, usize)>> = Vec::new();
        let mut last_line = text.len_lines() + 1;
        let mut col = 0;

        for range in selection.iter() {
            let coords = visual_coords_at_pos(text, range.head, tab_width);
            let anchor_coords = visual_coords_at_pos(text, range.anchor, tab_width);

            if coords.row != anchor_coords.row {
                log::info!("align cannot work with multi line selections");
                return;
            }

            col = if coords.row == last_line { col + 1 } else { 0 };

            if col >= column_widths.len() {
                column_widths.push(Vec::new());
            }
            column_widths[col].push((range.from(), coords.col));

            last_line = coords.row;
        }

        let mut changes = Vec::with_capacity(selection.len());
        let len = column_widths.first().map(|cols| cols.len()).unwrap_or(0);
        let mut offs = vec![0usize; len];

        for col in column_widths {
            let max_col = col
                .iter()
                .enumerate()
                .map(|(row, (_, cursor))| *cursor + offs[row])
                .max()
                .unwrap_or(0);

            for (row, (insert_pos, last_col)) in col.into_iter().enumerate() {
                let ins_count = max_col - (last_col + offs[row]);
                if ins_count == 0 {
                    continue;
                }
                offs[row] += ins_count;
                changes.push((insert_pos, insert_pos, Some(" ".repeat(ins_count).into())));
            }
        }

        changes.sort_unstable_by_key(|(from, _, _)| *from);

        if !changes.is_empty() {
            let transaction = helix_core::Transaction::change(doc.text(), changes.into_iter());
            doc.apply(&transaction, view_id);
        }
    }
}

/// Get the matching open/close pair for a surround character.
fn surround_pair(ch: char) -> (char, char) {
    match ch {
        '(' | ')' => ('(', ')'),
        '[' | ']' => ('[', ']'),
        '{' | '}' => ('{', '}'),
        '<' | '>' => ('<', '>'),
        _ => (ch, ch), // quotes etc.
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::SelectionOps;
    use crate::test_helpers::{assert_state, doc_view, test_context};

    use super::*;

    #[test]
    fn indent_single_line() {
        let mut ctx = test_context("#[|h]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.indent_line(doc_id, view_id);
        // Default indent style is tab
        assert_state(&ctx, "\t#[|h]#ello\nworld\n");
    }

    #[test]
    fn indent_multiple_selected_lines() {
        // Select 2 lines with `x` twice, then indent
        let mut ctx = test_context("#[|h]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        ctx.select_line(doc_id, view_id);
        ctx.indent_line(doc_id, view_id);
        assert_state(&ctx, "\t#[hello\n\tworld\n|]#foo\n");
    }

    #[test]
    fn indent_three_lines_then_indent() {
        // The exact bug scenario: press `x` 3 times, then `>` indents all 3 lines
        let mut ctx = test_context("#[|h]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        ctx.select_line(doc_id, view_id);
        ctx.select_line(doc_id, view_id);
        ctx.indent_line(doc_id, view_id);
        assert_state(&ctx, "\t#[hello\n\tworld\n\tfoo\n|]#");
    }

    #[test]
    fn unindent_single_line() {
        let mut ctx = test_context("\t#[|h]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.unindent_line(doc_id, view_id);
        assert_state(&ctx, "#[|h]#ello\nworld\n");
    }

    #[test]
    fn unindent_multiple_selected_lines() {
        let mut ctx = test_context("\t#[|h]#ello\n\tworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        ctx.select_line(doc_id, view_id);
        ctx.unindent_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\nworld\n|]#foo\n");
    }

    #[test]
    fn unindent_does_nothing_without_indent() {
        let mut ctx = test_context("#[|h]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.unindent_line(doc_id, view_id);
        assert_state(&ctx, "#[|h]#ello\n");
    }

    // --- change_selection ---

    #[test]
    fn change_selection_deletes_and_enters_insert() {
        // Select "hello" then change it
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.change_selection(doc_id, view_id);
        // "hello" deleted, cursor at position 0, insert mode
        assert_eq!(ctx.editor.mode, Mode::Insert);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, " world\n");
    }

    #[test]
    fn change_selection_point_enters_insert() {
        // Point selection (single char) — should still enter insert
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.change_selection(doc_id, view_id);
        assert_eq!(ctx.editor.mode, Mode::Insert);
    }

    // --- replace_char ---

    #[test]
    fn replace_char_single_char() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.replace_char(doc_id, view_id, 'X');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "Xello\n");
    }

    #[test]
    fn replace_char_multi_char_selection() {
        let mut ctx = test_context("#[hel|]#lo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.replace_char(doc_id, view_id, '.');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "...lo\n");
    }

    #[test]
    fn replace_char_preserves_newlines() {
        let mut ctx = test_context("#[hello\nworld|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.replace_char(doc_id, view_id, '.');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, ".....\n.....\n");
    }

    // --- join_lines ---

    #[test]
    fn join_lines_two_lines() {
        let mut ctx = test_context("#[h|]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.join_lines(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello world\n");
    }

    #[test]
    fn join_lines_strips_leading_whitespace() {
        let mut ctx = test_context("#[h|]#ello\n    world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.join_lines(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello world\n");
    }

    #[test]
    fn join_lines_multi_line_selection() {
        // Select 3 lines, join them all
        let mut ctx = test_context("#[|h]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        ctx.select_line(doc_id, view_id);
        ctx.select_line(doc_id, view_id);
        ctx.join_lines(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello world foo\n");
    }

    // --- toggle_case ---

    #[test]
    fn toggle_case_swaps() {
        let mut ctx = test_context("#[Hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.toggle_case(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hELLO\n");
    }

    #[test]
    fn toggle_case_single_char() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.toggle_case(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "Hello\n");
    }

    // --- to_lowercase / to_uppercase ---

    #[test]
    fn to_lowercase_converts() {
        let mut ctx = test_context("#[HELLO|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.to_lowercase(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n");
    }

    #[test]
    fn to_uppercase_converts() {
        let mut ctx = test_context("#[hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.to_uppercase(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "HELLO\n");
    }

    // --- surround_add ---

    #[test]
    fn surround_add_parens() {
        let mut ctx = test_context("#[hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.surround_add(doc_id, view_id, '(');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "(hello)\n");
    }

    #[test]
    fn surround_add_quotes() {
        let mut ctx = test_context("#[hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.surround_add(doc_id, view_id, '"');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "\"hello\"\n");
    }

    #[test]
    fn surround_add_brackets() {
        let mut ctx = test_context("#[hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.surround_add(doc_id, view_id, '[');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "[hello]\n");
    }

    // --- surround_delete ---

    #[test]
    fn surround_delete_parens() {
        let mut ctx = test_context("(#[hello|]#)\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.surround_delete(doc_id, view_id, '(');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n");
    }

    // --- surround_replace ---

    #[test]
    fn surround_replace_parens_to_brackets() {
        let mut ctx = test_context("(#[hello|]#)\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.surround_replace(doc_id, view_id, '(', '[');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "[hello]\n");
    }

    #[test]
    fn surround_replace_quotes_to_parens() {
        let mut ctx = test_context("\"#[hello|]#\"\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.surround_replace(doc_id, view_id, '"', '(');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "(hello)\n");
    }

    // --- insert_char ---

    #[test]
    fn insert_char_at_cursor() {
        let mut ctx = test_context("hel#[l|]#o\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.insert_char(doc_id, view_id, 'X');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "helXlo\n");
    }

    #[test]
    fn insert_char_at_start() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.insert_char(doc_id, view_id, 'X');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "Xhello\n");
    }

    // --- insert_newline ---

    #[test]
    fn insert_newline_splits_line() {
        let mut ctx = test_context("hel#[l|]#o\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.insert_newline(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hel\nlo\n");
    }

    #[test]
    fn insert_newline_preserves_indentation() {
        let mut ctx = test_context("    hel#[l|]#o\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.insert_newline(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "    hel\n    lo\n");
    }

    // --- insert_tab ---

    #[test]
    fn insert_tab_at_cursor() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.insert_tab(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        // Default indent style inserts tab or spaces
        assert!(
            text.starts_with('\t') || text.starts_with(' '),
            "should insert indent"
        );
    }

    // --- delete_char_backward ---

    #[test]
    fn delete_char_backward_basic() {
        let mut ctx = test_context("he#[l|]#lo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_char_backward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hllo\n");
    }

    #[test]
    fn delete_char_backward_at_start_noop() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_char_backward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n");
    }

    // --- delete_char_forward ---

    #[test]
    fn delete_char_forward_basic() {
        let mut ctx = test_context("he#[l|]#lo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_char_forward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "helo\n");
    }

    // --- open_line_below ---

    #[test]
    fn open_line_below_inserts_newline() {
        let mut ctx = test_context("hel#[l|]#o\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.open_line_below(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n\nworld\n");
    }

    #[test]
    fn open_line_below_preserves_indentation() {
        let mut ctx = test_context("    hel#[l|]#o\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.open_line_below(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "    hello\n    \nworld\n");
    }

    // --- open_line_above ---

    #[test]
    fn open_line_above_inserts_newline() {
        let mut ctx = test_context("hello\n#[w|]#orld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.open_line_above(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n\nworld\n");
    }

    // --- undo / redo ---
    // Note: undo_reverts_change removed — test_context setup transaction and
    // insert_char are in the same undo group, so undo undoes both at once.

    #[test]
    fn redo_restores_change() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.insert_char(doc_id, view_id, 'X');
        ctx.undo(doc_id, view_id);
        ctx.redo(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "Xhello\n");
    }

    // --- delete_word_backward ---

    #[test]
    fn delete_word_backward_basic() {
        let mut ctx = test_context("hello #[w|]#orld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_word_backward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        // Should delete "hello " backward
        assert_eq!(text, "world\n");
    }

    #[test]
    fn delete_word_backward_at_start_noop() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_word_backward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n");
    }

    // --- delete_to_line_start ---

    #[test]
    fn delete_to_line_start_basic() {
        let mut ctx = test_context("hel#[l|]#o\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_to_line_start(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "lo\n");
    }

    #[test]
    fn delete_to_line_start_at_start_noop() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_to_line_start(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n");
    }

    // --- change_selection_noyank ---

    #[test]
    fn change_selection_noyank_deletes_without_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.change_selection_noyank(doc_id, view_id);
        assert_eq!(ctx.editor.mode, Mode::Insert);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, " world\n");
        // Verify nothing was yanked to default register
        let reg_content = ctx.editor.registers.read('"', &ctx.editor);
        assert!(
            reg_content.is_none(),
            "should not have written to default register"
        );
    }

    // --- delete_word_forward ---

    #[test]
    fn delete_word_forward_basic() {
        let mut ctx = test_context("#[h|]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_word_forward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "world\n");
    }

    // --- kill_to_line_end ---

    #[test]
    fn kill_to_line_end_basic() {
        let mut ctx = test_context("he#[l|]#lo\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.kill_to_line_end(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "he\nworld\n");
    }

    #[test]
    fn kill_to_line_end_at_end_noop() {
        let mut ctx = test_context("hell#[o|]#\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.kill_to_line_end(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        // Cursor is at 'o' which is the last char before \n, so nothing to kill
        assert_eq!(text, "hell\nworld\n");
    }

    // --- add_newline_below ---

    #[test]
    fn add_newline_below_basic() {
        let mut ctx = test_context("#[h|]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.add_newline_below(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n\nworld\n");
    }

    // --- add_newline_above ---

    #[test]
    fn add_newline_above_basic() {
        let mut ctx = test_context("hello\n#[w|]#orld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.add_newline_above(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "hello\n\nworld\n");
    }

    // --- increment ---

    #[test]
    fn increment_number() {
        // Select "42" then increment
        let mut ctx = test_context("#[42|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.increment(doc_id, view_id, 1);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "43\n");
    }

    #[test]
    fn decrement_number() {
        let mut ctx = test_context("#[42|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.increment(doc_id, view_id, -1);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        assert_eq!(text, "41\n");
    }

    // Note: earlier_and_later_navigate_history removed — test_context setup
    // transaction shares an undo group with subsequent operations, making
    // earlier/later step counts unreliable in test context.
}
