//! Text editing operations for the editor.

use helix_core::history::UndoKind;
use helix_core::RopeSlice;
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
}
