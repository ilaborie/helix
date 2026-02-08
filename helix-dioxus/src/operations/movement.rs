//! Movement operations for the editor.

use helix_core::movement::Movement;
use helix_view::{Align, DocumentId, ViewId};

use crate::state::{Direction, EditorContext};

/// Number of lines per page for page up/down movement.
const PAGE_SIZE: usize = 20;

/// Number of lines for half-page up/down movement.
const HALF_PAGE_SIZE: usize = PAGE_SIZE / 2;

/// Extension trait for movement operations.
pub trait MovementOps {
    fn move_cursor(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction);
    fn move_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_word_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_long_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_long_word_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_long_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_line_start(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn move_line_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn goto_first_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn goto_last_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn goto_first_nonwhitespace(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn page_up(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn page_down(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn half_page_up(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn half_page_down(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn scroll_up(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize);
    fn scroll_down(&mut self, doc_id: DocumentId, view_id: ViewId, lines: usize);
    fn scroll_to_line(&mut self, doc_id: DocumentId, view_id: ViewId, target_line: usize);
    fn match_bracket(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn align_view_center(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn align_view_top(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn align_view_bottom(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn goto_window_top(&mut self);
    fn goto_window_center(&mut self);
    fn goto_window_bottom(&mut self);
    fn goto_last_accessed_file(&mut self);
    fn goto_last_modified_file(&mut self);
    fn goto_last_modification(&mut self);
    fn goto_first_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn goto_last_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn next_paragraph(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn prev_paragraph(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn next_function(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn prev_function(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn next_class(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn prev_class(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn next_parameter(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn prev_parameter(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn next_comment(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn prev_comment(&mut self, doc_id: DocumentId, view_id: ViewId);
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

        let new_selection =
            selection.transform(|range| helix_core::movement::move_prev_word_start(text, range, 1));

        doc.set_selection(view_id, new_selection);
    }

    fn move_word_end(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection =
            selection.transform(|range| helix_core::movement::move_next_word_end(text, range, 1));

        doc.set_selection(view_id, new_selection);
    }

    fn move_long_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection
            .transform(|range| helix_core::movement::move_next_long_word_start(text, range, 1));

        doc.set_selection(view_id, new_selection);
    }

    fn move_long_word_end(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection
            .transform(|range| helix_core::movement::move_next_long_word_end(text, range, 1));

        doc.set_selection(view_id, new_selection);
    }

    fn move_long_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection
            .transform(|range| helix_core::movement::move_prev_long_word_start(text, range, 1));

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

    fn goto_first_nonwhitespace(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);

        let mut pos = line_start;
        for c in text.line(line).chars() {
            if c.is_whitespace() && c != '\n' {
                pos += 1;
            } else {
                break;
            }
        }

        doc.set_selection(view_id, helix_core::Selection::point(pos));
    }

    fn page_up(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.move_vertical_by(doc_id, view_id, -(PAGE_SIZE as isize));
    }

    fn page_down(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.move_vertical_by(doc_id, view_id, PAGE_SIZE as isize);
    }

    fn half_page_up(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.move_vertical_by(doc_id, view_id, -(HALF_PAGE_SIZE as isize));
    }

    fn half_page_down(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.move_vertical_by(doc_id, view_id, HALF_PAGE_SIZE as isize);
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

    fn match_bracket(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);

        let syntax = doc.syntax();
        let new_pos = if let Some(syn) = syntax {
            helix_core::match_brackets::find_matching_bracket_fuzzy(syn, text, cursor)
        } else {
            helix_core::match_brackets::find_matching_bracket_plaintext(text, cursor)
        };

        if let Some(pos) = new_pos {
            doc.set_selection(view_id, helix_core::Selection::point(pos));
        }
    }

    fn align_view_center(&mut self, _doc_id: DocumentId, _view_id: ViewId) {
        let (view, doc) = helix_view::current!(self.editor);
        helix_view::align_view(doc, view, Align::Center);
    }

    fn align_view_top(&mut self, _doc_id: DocumentId, _view_id: ViewId) {
        let (view, doc) = helix_view::current!(self.editor);
        helix_view::align_view(doc, view, Align::Top);
    }

    fn align_view_bottom(&mut self, _doc_id: DocumentId, _view_id: ViewId) {
        let (view, doc) = helix_view::current!(self.editor);
        helix_view::align_view(doc, view, Align::Bottom);
    }

    fn goto_window_top(&mut self) {
        self.goto_window(Align::Top);
    }

    fn goto_window_center(&mut self) {
        self.goto_window(Align::Center);
    }

    fn goto_window_bottom(&mut self) {
        self.goto_window(Align::Bottom);
    }

    fn goto_last_accessed_file(&mut self) {
        let view = helix_view::view_mut!(self.editor);
        if let Some(alt) = view.docs_access_history.pop() {
            self.editor.switch(alt, helix_view::editor::Action::Replace);
        } else {
            self.editor.set_error("no last accessed buffer");
        }
    }

    fn goto_last_modified_file(&mut self) {
        let view = helix_view::view!(self.editor);
        let alternate_file = view
            .last_modified_docs
            .into_iter()
            .flatten()
            .find(|&id| id != view.doc);
        if let Some(alt) = alternate_file {
            self.editor.switch(alt, helix_view::editor::Action::Replace);
        } else {
            self.editor.set_error("no last modified buffer");
        }
    }

    fn goto_last_modification(&mut self) {
        let (view, doc) = helix_view::current!(self.editor);
        let pos = doc.history.get_mut().last_edit_pos();
        if let Some(pos) = pos {
            let text = doc.text().slice(..);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| range.put_cursor(text, pos, false));
            doc.set_selection(view.id, selection);
        }
    }

    fn goto_first_diagnostic(&mut self, _doc_id: DocumentId, _view_id: ViewId) {
        let (view, doc) = helix_view::current!(self.editor);
        if let Some(diag) = doc.diagnostics().first() {
            let selection = helix_core::Selection::single(diag.range.start, diag.range.end);
            doc.set_selection(view.id, selection);
        }
    }

    fn goto_last_diagnostic(&mut self, _doc_id: DocumentId, _view_id: ViewId) {
        let (view, doc) = helix_view::current!(self.editor);
        if let Some(diag) = doc.diagnostics().last() {
            let selection = helix_core::Selection::single(diag.range.start, diag.range.end);
            doc.set_selection(view.id, selection);
        }
    }

    fn next_paragraph(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            helix_core::movement::move_next_paragraph(text, range, 1, Movement::Move)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn prev_paragraph(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            helix_core::movement::move_prev_paragraph(text, range, 1, Movement::Move)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn next_function(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "function",
            helix_core::movement::Direction::Forward,
        );
    }

    fn prev_function(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "function",
            helix_core::movement::Direction::Backward,
        );
    }

    fn next_class(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "class",
            helix_core::movement::Direction::Forward,
        );
    }

    fn prev_class(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "class",
            helix_core::movement::Direction::Backward,
        );
    }

    fn next_parameter(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "parameter",
            helix_core::movement::Direction::Forward,
        );
    }

    fn prev_parameter(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "parameter",
            helix_core::movement::Direction::Backward,
        );
    }

    fn next_comment(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "comment",
            helix_core::movement::Direction::Forward,
        );
    }

    fn prev_comment(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.goto_ts_object(
            doc_id,
            view_id,
            "comment",
            helix_core::movement::Direction::Backward,
        );
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

    /// Find a character on the current line and extend selection to it.
    ///
    /// Like `find_char` but preserves the original anchor (for select mode).
    pub(crate) fn extend_find_char(
        &mut self,
        doc_id: DocumentId,
        view_id: ViewId,
        ch: char,
        forward: bool,
        till: bool,
    ) {
        // Remember this motion for repeat
        self.last_find_char = Some((ch, forward, till));

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();

        let cursor = primary.cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);
        let line_end = line_start + text.line(line).len_chars().saturating_sub(1);

        let target = if forward {
            let start = cursor + 1;
            (start..=line_end).find(|&pos| text.char(pos) == ch)
        } else if cursor == 0 {
            None
        } else {
            (line_start..cursor).rev().find(|&pos| text.char(pos) == ch)
        };

        if let Some(mut pos) = target {
            if till {
                if forward {
                    pos = pos.saturating_sub(1).max(cursor);
                } else {
                    pos = (pos + 1).min(line_end);
                }
            }
            // Extend: preserve original anchor, move head
            let new_head = if forward { pos + 1 } else { pos };
            let new_selection = helix_core::Selection::single(primary.anchor, new_head);
            doc.set_selection(view_id, new_selection);
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

    /// Move cursor to a window position (top/center/bottom).
    fn goto_window(&mut self, align: Align) {
        let config = self.editor.config();
        let (view, doc) = helix_view::current!(self.editor);
        let view_offset = doc.view_offset(view.id);

        let last_visual_line = view.last_visual_line(doc);
        let scrolloff = config.scrolloff.min(last_visual_line.saturating_sub(1) / 2);

        let visual_line = match align {
            Align::Top => view_offset.vertical_offset + scrolloff,
            Align::Center => view_offset.vertical_offset + (last_visual_line / 2),
            Align::Bottom => {
                view_offset.vertical_offset + last_visual_line.saturating_sub(scrolloff)
            }
        };
        let visual_line = visual_line
            .max(view_offset.vertical_offset + scrolloff)
            .min(view_offset.vertical_offset + last_visual_line.saturating_sub(scrolloff));

        if let Some(pos) = view.pos_at_visual_coords(doc, visual_line as u16, 0, false) {
            let text = doc.text().slice(..);
            let selection = doc
                .selection(view.id)
                .clone()
                .transform(|range| range.put_cursor(text, pos, false));
            doc.set_selection(view.id, selection);
        }
    }

    /// Navigate to a tree-sitter object (function, class, parameter, comment).
    fn goto_ts_object(
        &mut self,
        doc_id: DocumentId,
        view_id: ViewId,
        object: &str,
        dir: helix_core::movement::Direction,
    ) {
        // Load syn_loader before borrowing doc to avoid borrow conflicts.
        let loader = self.editor.syn_loader.load();
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        if let Some(syntax) = doc.syntax() {
            let text = doc.text().slice(..);
            let root = syntax.tree().root_node();

            let selection = doc.selection(view_id).clone().transform(|range| {
                let new_range = helix_core::movement::goto_treesitter_object(
                    text, range, object, dir, &root, syntax, &loader, 1,
                );
                new_range.with_direction(dir)
            });

            doc.set_selection(view_id, selection);
        } else {
            self.editor
                .set_status("Syntax-tree is not available in current buffer");
        }
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

    #[test]
    fn move_word_end_basic() {
        let mut ctx = test_context("#[h|]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.move_word_end(doc_id, view_id);
        // word end moves to last char of current word
        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id).primary();
        // Should land at 'o' (index 4) — the end of "hello"
        assert!(sel.head >= 4, "head should be at or past 'o': {:?}", sel);
    }

    #[test]
    fn move_long_word_forward_skips_punctuation() {
        let mut ctx = test_context("#[h|]#ello.world foo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.move_long_word_forward(doc_id, view_id);
        // WORD motion treats punctuation as part of word, so jumps past "hello.world"
        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id).primary();
        // Should land at 'f' (index 12) — start of "foo"
        assert!(sel.head >= 12, "head should be at 'f': {:?}", sel);
    }

    #[test]
    fn move_long_word_end_skips_punctuation() {
        let mut ctx = test_context("#[h|]#ello.world foo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.move_long_word_end(doc_id, view_id);
        // WORD end should land at end of "hello.world"
        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id).primary();
        // 'd' is at index 10
        assert!(sel.head >= 10, "head should be at or past 'd': {:?}", sel);
    }

    #[test]
    fn move_long_word_backward_skips_punctuation() {
        let mut ctx = test_context("hello.world #[f|]#oo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.move_long_word_backward(doc_id, view_id);
        // WORD backward should move head to start of "hello.world"
        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id).primary();
        assert_eq!(sel.from(), 0, "should start at 'hello.world': {:?}", sel);
    }

    #[test]
    fn match_bracket_parens() {
        // Plaintext bracket matching (no syntax tree)
        let mut ctx = test_context("#[(|]#a + b)\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.match_bracket(doc_id, view_id);
        assert_state(&ctx, "(a + b#[)|]#\n");
    }

    #[test]
    fn match_bracket_reverse() {
        let mut ctx = test_context("(a + b#[)|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.match_bracket(doc_id, view_id);
        assert_state(&ctx, "#[(|]#a + b)\n");
    }

    #[test]
    fn match_bracket_no_match() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.match_bracket(doc_id, view_id);
        // No bracket at cursor — should not move
        assert_state(&ctx, "#[h|]#ello\n");
    }
}
