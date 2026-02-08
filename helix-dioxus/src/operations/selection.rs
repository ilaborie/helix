//! Selection operations for the editor.

use helix_view::{DocumentId, ViewId};

use crate::state::{Direction, EditorContext};

/// Extension trait for selection operations.
pub trait SelectionOps {
    fn extend_selection(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction);
    fn extend_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_word_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_long_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_long_word_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_long_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_line_start(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_line_end(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn select_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn collapse_selection(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn keep_primary_selection(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn select_inside_pair(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char);
    fn select_around_pair(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char);
    fn select_all(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn flip_selections(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_to_line_bounds(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn shrink_to_line_bounds(&mut self, doc_id: DocumentId, view_id: ViewId);
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

    fn extend_word_end(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_next_word_end(text, range, 1);
            helix_core::Range::new(range.anchor, new_range.head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_long_word_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_next_long_word_start(text, range, 1);
            helix_core::Range::new(range.anchor, new_range.head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_long_word_end(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_next_long_word_end(text, range, 1);
            helix_core::Range::new(range.anchor, new_range.head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_long_word_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_prev_long_word_start(text, range, 1);
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

    /// Collapse selection to cursor position (`;` in Helix).
    fn collapse_selection(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            helix_core::Range::point(cursor)
        });

        doc.set_selection(view_id, new_selection);
    }

    /// Keep only primary selection, remove others (`,` in Helix).
    fn keep_primary_selection(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();
        doc.set_selection(
            view_id,
            helix_core::Selection::single(primary.anchor, primary.head),
        );
    }

    /// Select inside a bracket/quote pair (`mi` in Helix).
    fn select_inside_pair(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();

        if let Ok((open, close)) = helix_core::surround::find_nth_pairs_pos(text, ch, primary, 1) {
            // Inside: exclude the delimiters
            let new_anchor = open + 1;
            let new_head = close;
            if new_anchor < new_head {
                doc.set_selection(view_id, helix_core::Selection::single(new_anchor, new_head));
            }
        }
    }

    /// Select around a bracket/quote pair (`ma` in Helix).
    fn select_around_pair(&mut self, doc_id: DocumentId, view_id: ViewId, ch: char) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();

        if let Ok((open, close)) = helix_core::surround::find_nth_pairs_pos(text, ch, primary, 1) {
            // Around: include the delimiters
            doc.set_selection(view_id, helix_core::Selection::single(open, close + 1));
        }
    }

    /// Select entire buffer (`%` in Helix).
    fn select_all(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let len = text.len_chars();
        doc.set_selection(view_id, helix_core::Selection::single(0, len));
    }

    /// Flip selections: swap anchor and head (`Alt+;` in Helix).
    fn flip_selections(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let selection = doc.selection(view_id).clone();
        let new_selection =
            selection.transform(|range| helix_core::Range::new(range.head, range.anchor));
        doc.set_selection(view_id, new_selection);
    }

    /// Extend selection to full line bounds (`X` in Helix).
    fn extend_to_line_bounds(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().clone();
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let (start_line, end_line) = range.line_range(text.slice(..));
            let start = text.line_to_char(start_line);
            let end = text.line_to_char((end_line + 1).min(text.len_lines()));
            helix_core::Range::new(start, end).with_direction(range.direction())
        });

        doc.set_selection(view_id, new_selection);
    }

    /// Shrink selection to line bounds (`Alt-x` in Helix).
    fn shrink_to_line_bounds(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().clone();
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let (start_line, end_line) = range.line_range(text.slice(..));

            // Do nothing if the selection is within one line
            if start_line == end_line {
                return range;
            }

            let mut start = text.line_to_char(start_line);
            let mut end = text.line_to_char((end_line + 1).min(text.len_lines()));

            if start != range.from() {
                start = text.line_to_char((start_line + 1).min(text.len_lines()));
            }

            if end != range.to() {
                end = text.line_to_char(end_line);
            }

            helix_core::Range::new(start, end).with_direction(range.direction())
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

    // --- collapse_selection ---

    #[test]
    fn collapse_selection_multi_char() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.collapse_selection(doc_id, view_id);
        // Collapse to cursor position (point selection = 1 char)
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.to() - sel.from(), 1, "should be a point selection");
    }

    #[test]
    fn collapse_selection_already_point() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.collapse_selection(doc_id, view_id);
        assert_state(&ctx, "#[h|]#ello\n");
    }

    // --- keep_primary_selection ---

    #[test]
    fn keep_primary_selection_single() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.keep_primary_selection(doc_id, view_id);
        assert_state(&ctx, "#[h|]#ello\n");
    }

    // --- extend_word_end ---

    #[test]
    fn extend_word_end_basic() {
        // #[|h]# means head=0, anchor=1
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_word_end(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        // Anchor preserved from original range (1), head moves to word end
        assert_eq!(sel.anchor, 1, "anchor should stay at 1");
        assert!(sel.head >= 4, "head should be at or past 'o': {:?}", sel);
    }

    // --- extend_long_word_forward ---

    #[test]
    fn extend_long_word_forward_basic() {
        let mut ctx = test_context("#[|h]#ello.world foo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_long_word_forward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 1, "anchor should stay at 1");
        assert!(sel.head >= 12, "head should be at 'f': {:?}", sel);
    }

    // --- extend_long_word_end ---

    #[test]
    fn extend_long_word_end_basic() {
        let mut ctx = test_context("#[|h]#ello.world foo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_long_word_end(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 1, "anchor should stay at 1");
        assert!(sel.head >= 10, "head should be at or past 'd': {:?}", sel);
    }

    // --- extend_long_word_backward ---

    #[test]
    fn extend_long_word_backward_basic() {
        // #[|f]# means head=12, anchor=13
        let mut ctx = test_context("hello.world #[|f]#oo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_long_word_backward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 13, "anchor should stay at 13");
        assert_eq!(sel.head, 0, "head should be at start of 'hello.world'");
    }

    // --- select_inside_pair ---

    #[test]
    fn select_inside_pair_parens() {
        let mut ctx = test_context("(he#[l|]#lo)\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_inside_pair(doc_id, view_id, '(');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        // Inside parens: anchor=1, head=6 → text.slice(1..6) = "hello"
        assert_eq!(sel.from(), 1, "should start after '('");
        assert_eq!(sel.to(), 6, "head at closing delimiter position");
    }

    #[test]
    fn select_inside_pair_quotes() {
        let mut ctx = test_context("\"he#[l|]#lo\"\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_inside_pair(doc_id, view_id, '"');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.from(), 1, "should start after '\"'");
        assert_eq!(sel.to(), 6, "head at closing delimiter position");
    }

    // --- select_around_pair ---

    #[test]
    fn select_around_pair_parens() {
        let mut ctx = test_context("(he#[l|]#lo)\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_around_pair(doc_id, view_id, '(');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        // Around parens: anchor=0, head=close+1=7 → text.slice(0..7) = "(hello)"
        assert_eq!(sel.from(), 0, "should include '('");
        assert_eq!(sel.to(), 7, "should include past ')'");
    }

    #[test]
    fn select_around_pair_brackets() {
        let mut ctx = test_context("[he#[l|]#lo]\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_around_pair(doc_id, view_id, '[');
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.from(), 0, "should include '['");
        assert_eq!(sel.to(), 7, "should include past ']'");
    }
}
