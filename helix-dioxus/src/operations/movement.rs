//! Movement operations for the editor.

use helix_view::{DocumentId, ViewId};

use crate::state::{Direction, EditorContext};

/// Extension trait for movement operations.
pub trait MovementOps {
    fn move_cursor(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction);
    fn move_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_line_start(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_line_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn goto_first_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn goto_last_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn page_up(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn page_down(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn scroll_up(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize);
    fn scroll_down(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize);
}

impl MovementOps for EditorContext {
    fn move_cursor(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let new_cursor = match direction {
                Direction::Left => cursor.saturating_sub(1),
                Direction::Right => {
                    let max = text.len_chars().saturating_sub(1);
                    (cursor + 1).min(max)
                }
                Direction::Up => {
                    let line = text.char_to_line(cursor);
                    if line == 0 {
                        // At first line, collapse to point at current cursor
                        return helix_core::Range::point(cursor);
                    }
                    let col = cursor - text.line_to_char(line);
                    let new_line = line - 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
                Direction::Down => {
                    let line = text.char_to_line(cursor);
                    if line >= text.len_lines().saturating_sub(1) {
                        // At last line, collapse to point at current cursor
                        return helix_core::Range::point(cursor);
                    }
                    let col = cursor - text.line_to_char(line);
                    let new_line = line + 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
            };
            helix_core::Range::point(new_cursor)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn move_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        // Helix's selection-first model: movements create selections
        let new_selection =
            selection.transform(|range| helix_core::movement::move_next_word_start(text, range, 1));

        doc.set_selection(view_id, new_selection);
    }

    fn move_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        // Helix's selection-first model: movements create selections
        let new_selection =
            selection.transform(|range| helix_core::movement::move_prev_word_start(text, range, 1));

        doc.set_selection(view_id, new_selection);
    }

    fn move_line_start(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let line = text.char_to_line(cursor);
            let line_start = text.line_to_char(line);
            helix_core::Range::point(line_start)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn move_line_end(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let line = text.char_to_line(cursor);
            let line_end = text.line_to_char(line) + text.line(line).len_chars().saturating_sub(1);
            helix_core::Range::point(line_end.max(text.line_to_char(line)))
        });

        doc.set_selection(view_id, new_selection);
    }

    fn goto_first_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        doc.set_selection(view_id, helix_core::Selection::point(0));
    }

    fn goto_last_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);

        let last_line = text.len_lines().saturating_sub(1);
        let line_start = text.line_to_char(last_line);

        doc.set_selection(view_id, helix_core::Selection::point(line_start));
    }

    fn page_up(&mut self, doc_id: DocumentId, view_id: ViewId) {
        // Move cursor up by approximately half the viewport (20 lines)
        let page_size = 20;
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let line = text.char_to_line(cursor);
            let col = cursor - text.line_to_char(line);
            let new_line = line.saturating_sub(page_size);
            let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
            let new_cursor = text.line_to_char(new_line) + col.min(new_line_len);
            helix_core::Range::point(new_cursor)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn page_down(&mut self, doc_id: DocumentId, view_id: ViewId) {
        // Move cursor down by approximately half the viewport (20 lines)
        let page_size = 20;
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let line = text.char_to_line(cursor);
            let col = cursor - text.line_to_char(line);
            let last_line = text.len_lines().saturating_sub(1);
            let new_line = (line + page_size).min(last_line);
            let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
            let new_cursor = text.line_to_char(new_line) + col.min(new_line_len);
            helix_core::Range::point(new_cursor)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn scroll_up(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);

        // Get the current view offset
        let mut offset = doc.view_offset(view_id);
        let current_line = text.char_to_line(offset.anchor.min(text.len_chars()));

        // Calculate the new line (scroll up = decrease line number)
        let new_line = current_line.saturating_sub(lines);

        // Set the new anchor to the start of the new line
        offset.anchor = text.line_to_char(new_line);
        doc.set_view_offset(view_id, offset);
    }

    fn scroll_down(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);

        // Get the current view offset
        let mut offset = doc.view_offset(view_id);
        let current_line = text.char_to_line(offset.anchor.min(text.len_chars()));

        // Calculate the new line (scroll down = increase line number)
        let last_line = text.len_lines().saturating_sub(1);
        let new_line = (current_line + lines).min(last_line);

        // Set the new anchor to the start of the new line
        offset.anchor = text.line_to_char(new_line);
        doc.set_view_offset(view_id, offset);
    }
}
