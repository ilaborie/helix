//! Movement operations for the editor.

use helix_view::{DocumentId, ViewId};

use crate::state::{Direction, EditorContext};

/// Number of lines per page for page up/down movement.
const PAGE_SIZE: usize = 20;

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
    fn scroll_to_line(&mut self, doc_id: DocumentId, view_id: ViewId, target_line: usize);
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
        self.move_vertical_by(doc_id, view_id, -(PAGE_SIZE as isize));
    }

    fn page_down(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.move_vertical_by(doc_id, view_id, PAGE_SIZE as isize);
    }

    fn scroll_up(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize) {
        self.scroll_by(doc_id, view_id, -(lines as isize));
    }

    fn scroll_down(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize) {
        self.scroll_by(doc_id, view_id, lines as isize);
    }

    fn scroll_to_line(&mut self, doc_id: DocumentId, view_id: ViewId, target_line: usize) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);

        // Clamp target to valid range
        let last_line = text.len_lines().saturating_sub(1);
        let target = target_line.min(last_line);

        // Get current view offset and update anchor
        let mut offset = doc.view_offset(view_id);
        offset.anchor = text.line_to_char(target);
        doc.set_view_offset(view_id, offset);
    }
}

impl EditorContext {
    /// Find a character on the current line and move the cursor to it.
    ///
    /// - `forward`: search direction (true = right, false = left)
    /// - `till`: if true, stop one position before the character
    pub(crate) fn find_char(
        &mut self,
        doc_id: DocumentId,
        view_id: ViewId,
        ch: char,
        forward: bool,
        till: bool,
    ) {
        // Remember this motion for repeat (;) and reverse (,)
        self.last_find_char = Some((ch, forward, till));

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);
        let line_end = line_start + text.line(line).len_chars().saturating_sub(1); // exclude newline

        let target = if forward {
            // Search forward from cursor+1 to line end
            let start = cursor + 1;
            (start..=line_end).find(|&pos| text.char(pos) == ch)
        } else {
            // Search backward from cursor-1 to line start
            if cursor == 0 {
                None
            } else {
                (line_start..cursor).rev().find(|&pos| text.char(pos) == ch)
            }
        };

        if let Some(mut pos) = target {
            if till {
                // Stop one position before the target
                if forward {
                    pos = pos.saturating_sub(1).max(cursor);
                } else {
                    pos = (pos + 1).min(line_end);
                }
            }
            doc.set_selection(view_id, helix_core::Selection::point(pos));
        }
    }

    /// Move cursor vertically by `delta` lines (negative = up, positive = down).
    fn move_vertical_by(&mut self, doc_id: DocumentId, view_id: ViewId, delta: isize) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let last_line = text.len_lines().saturating_sub(1);

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let line = text.char_to_line(cursor);
            let col = cursor - text.line_to_char(line);
            let new_line = if delta < 0 {
                line.saturating_sub(delta.unsigned_abs())
            } else {
                (line + delta as usize).min(last_line)
            };
            let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
            let new_cursor = text.line_to_char(new_line) + col.min(new_line_len);
            helix_core::Range::point(new_cursor)
        });

        doc.set_selection(view_id, new_selection);
    }

    /// Scroll the view by `delta` lines (negative = up, positive = down).
    fn scroll_by(&mut self, doc_id: DocumentId, view_id: ViewId, delta: isize) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);

        let mut offset = doc.view_offset(view_id);
        let current_line = text.char_to_line(offset.anchor.min(text.len_chars()));
        let last_line = text.len_lines().saturating_sub(1);

        let new_line = if delta < 0 {
            current_line.saturating_sub(delta.unsigned_abs())
        } else {
            (current_line + delta as usize).min(last_line)
        };

        offset.anchor = text.line_to_char(new_line);
        doc.set_view_offset(view_id, offset);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_state, doc_view, test_context};

    use super::*;

    #[test]
    fn find_char_forward() {
        let mut ctx = test_context("#[h|]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.find_char(doc_id, view_id, 'o', true, false);
        assert_state(&ctx, "hell#[o|]# world\n");
    }

    #[test]
    fn find_char_backward() {
        let mut ctx = test_context("hello w#[o|]#rld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.find_char(doc_id, view_id, 'l', false, false);
        assert_state(&ctx, "hel#[l|]#o world\n");
    }

    #[test]
    fn till_char_forward() {
        let mut ctx = test_context("#[h|]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.find_char(doc_id, view_id, 'o', true, true);
        assert_state(&ctx, "hel#[l|]#o world\n");
    }

    #[test]
    fn till_char_backward() {
        let mut ctx = test_context("hello w#[o|]#rld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.find_char(doc_id, view_id, 'l', false, true);
        assert_state(&ctx, "hell#[o|]# world\n");
    }

    #[test]
    fn find_char_not_found() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.find_char(doc_id, view_id, 'z', true, false);
        // Cursor should not move when char is not found
        assert_state(&ctx, "#[h|]#ello\n");
    }

    #[test]
    fn find_char_remembers_last_motion() {
        let mut ctx = test_context("#[h|]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.find_char(doc_id, view_id, 'l', true, false);
        assert!(ctx.last_find_char.is_some());
        let (ch, forward, till) = ctx.last_find_char.expect("last_find_char should be set");
        assert_eq!(ch, 'l');
        assert!(forward);
        assert!(!till);
    }

    #[test]
    fn move_cursor_left_right() {
        use crate::state::Direction;
        let mut ctx = test_context("he#[l|]#lo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.move_cursor(doc_id, view_id, Direction::Right);
        assert_state(&ctx, "hel#[l|]#o\n");
        ctx.move_cursor(doc_id, view_id, Direction::Left);
        assert_state(&ctx, "he#[l|]#lo\n");
    }

    #[test]
    fn move_cursor_up_down() {
        use crate::state::Direction;
        let mut ctx = test_context("he#[l|]#lo\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.move_cursor(doc_id, view_id, Direction::Down);
        assert_state(&ctx, "hello\nwo#[r|]#ld\n");
        ctx.move_cursor(doc_id, view_id, Direction::Up);
        assert_state(&ctx, "he#[l|]#lo\nworld\n");
    }
}
