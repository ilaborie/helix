//! Selection operations for the editor.

use helix_view::{DocumentId, ViewId};

use crate::state::{Direction, EditorContext};

/// Extension trait for selection operations.
pub trait SelectionOps {
    fn extend_selection(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction);
    fn extend_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_line_start(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_line_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn select_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_line(&mut self, doc_id: DocumentId, view_id: ViewId);
}

impl SelectionOps for EditorContext {
    fn extend_selection(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let head = range.head;
            let anchor = range.anchor;

            let new_head = match direction {
                Direction::Left => head.saturating_sub(1),
                Direction::Right => {
                    let max = text.len_chars().saturating_sub(1);
                    (head + 1).min(max)
                }
                Direction::Up => {
                    let line = text.char_to_line(head);
                    if line == 0 {
                        return range;
                    }
                    let col = head - text.line_to_char(line);
                    let new_line = line - 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
                Direction::Down => {
                    let line = text.char_to_line(head);
                    if line >= text.len_lines().saturating_sub(1) {
                        return range;
                    }
                    let col = head - text.line_to_char(line);
                    let new_line = line + 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
            };

            helix_core::Range::new(anchor, new_head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_next_word_start(text, range, 1);
            helix_core::Range::new(range.anchor, new_range.head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_prev_word_start(text, range, 1);
            helix_core::Range::new(range.anchor, new_range.head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_line_start(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let line = text.char_to_line(range.head);
            let line_start = text.line_to_char(line);
            helix_core::Range::new(range.anchor, line_start)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_line_end(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let line = text.char_to_line(range.head);
            let line_end = text.line_to_char(line) + text.line(line).len_chars().saturating_sub(1);
            helix_core::Range::new(range.anchor, line_end.max(text.line_to_char(line)))
        });

        doc.set_selection(view_id, new_selection);
    }

    /// Extend selection to line bounds (helix `x` command).
    ///
    /// Snaps anchor to its line start and head to its line end (start of next line).
    /// On subsequent presses, since head is already at the start of the next line,
    /// it extends to include that line too — growing the selection one line at a time.
    fn select_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            // Snap anchor to the start of its line
            let anchor_line = text.char_to_line(range.anchor);
            let new_anchor = text.line_to_char(anchor_line);

            // Snap head to the end of its line (= start of next line)
            let head_line = text.char_to_line(range.head);
            let new_head = if head_line + 1 < text.len_lines() {
                text.line_to_char(head_line + 1)
            } else {
                text.len_chars()
            };

            helix_core::Range::new(new_anchor, new_head)
        });

        doc.set_selection(view_id, new_selection);
    }

    /// Extend selection to include the next line (helix `X` command).
    fn extend_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let end_line = text.char_to_line(range.to());
            let new_end = if end_line + 1 < text.len_lines() {
                text.line_to_char(end_line + 1)
            } else {
                text.len_chars()
            };
            helix_core::Range::new(range.from(), new_end)
        });

        doc.set_selection(view_id, new_selection);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_state, doc_view, test_context};

    use super::*;

    #[test]
    fn select_line_first_press_selects_current_line() {
        let mut ctx = test_context("#[|h]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\n|]#world\nfoo\n");
    }

    #[test]
    fn select_line_extends_downward() {
        let mut ctx = test_context("#[|h]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\n|]#world\nfoo\n");
        ctx.select_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\nworld\n|]#foo\n");
        ctx.select_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\nworld\nfoo\n|]#");
    }

    #[test]
    fn select_line_from_middle_of_line() {
        let mut ctx = test_context("hel#[|l]#o\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\n|]#world\n");
    }

    #[test]
    fn select_line_on_last_line_without_trailing_newline() {
        let mut ctx = test_context("hello\n#[|w]#orld");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_line(doc_id, view_id);
        assert_state(&ctx, "hello\n#[world|]#");
    }

    #[test]
    fn extend_line_grows_from_current() {
        let mut ctx = test_context("#[|h]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\n|]#world\nfoo\n");
        ctx.extend_line(doc_id, view_id);
        assert_state(&ctx, "#[hello\nworld\n|]#foo\n");
    }
}
