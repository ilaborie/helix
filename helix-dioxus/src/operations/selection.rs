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
    fn trim_selections(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn rotate_selections_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn rotate_selections_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn copy_selection_on_line(&mut self, doc_id: DocumentId, view_id: ViewId, forward: bool);
    fn split_selection_on_newline(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn select_regex(&mut self, doc_id: DocumentId, view_id: ViewId, pattern: &str);
    fn split_selection(&mut self, doc_id: DocumentId, view_id: ViewId, pattern: &str);
    fn extend_to_first_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_to_last_line(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_goto_first_nonwhitespace(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_goto_column(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn expand_selection(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn shrink_selection(&mut self, doc_id: DocumentId, view_id: ViewId);
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
    /// Rotate selections forward (`)` in Helix): move primary index forward.
    fn rotate_selections_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let mut selection = doc.selection(view_id).clone();
        let len = selection.len();
        if len > 1 {
            let idx = selection.primary_index();
            selection.set_primary_index((idx + 1) % len);
            doc.set_selection(view_id, selection);
        }
    }

    /// Rotate selections backward (`(` in Helix): move primary index backward.
    fn rotate_selections_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let mut selection = doc.selection(view_id).clone();
        let len = selection.len();
        if len > 1 {
            let idx = selection.primary_index();
            selection.set_primary_index((idx + len - 1) % len);
            doc.set_selection(view_id, selection);
        }
    }

    /// Copy selection to next/previous line (`C`/`A-C` in Helix).
    #[allow(deprecated)]
    fn copy_selection_on_line(&mut self, doc_id: DocumentId, view_id: ViewId, forward: bool) {
        use helix_core::{pos_at_visual_coords, visual_coords_at_pos, Position, Range, Selection};

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let tab_width = doc.tab_width();

        let mut ranges = helix_core::SmallVec::with_capacity(selection.len() * 2);
        ranges.extend_from_slice(selection.ranges());
        let mut primary_index = 0;

        for range in selection.iter() {
            let is_primary = *range == selection.primary();

            // The range is always head exclusive
            let (head, anchor) = if range.anchor < range.head {
                (range.head - 1, range.anchor)
            } else {
                (range.head, range.anchor.saturating_sub(1))
            };

            let head_pos = visual_coords_at_pos(text, head, tab_width);
            let anchor_pos = visual_coords_at_pos(text, anchor, tab_width);

            let height = std::cmp::max(head_pos.row, anchor_pos.row)
                - std::cmp::min(head_pos.row, anchor_pos.row)
                + 1;

            if is_primary {
                primary_index = ranges.len();
            }
            ranges.push(*range);

            let offset = height;

            let anchor_row = if forward {
                anchor_pos.row + offset
            } else {
                anchor_pos.row.saturating_sub(offset)
            };

            let head_row = if forward {
                head_pos.row + offset
            } else {
                head_pos.row.saturating_sub(offset)
            };

            if anchor_row >= text.len_lines() || head_row >= text.len_lines() {
                continue;
            }

            let new_anchor =
                pos_at_visual_coords(text, Position::new(anchor_row, anchor_pos.col), tab_width);
            let new_head =
                pos_at_visual_coords(text, Position::new(head_row, head_pos.col), tab_width);

            // Skip lines that are too short
            if visual_coords_at_pos(text, new_anchor, tab_width).col == anchor_pos.col
                && visual_coords_at_pos(text, new_head, tab_width).col == head_pos.col
            {
                if is_primary {
                    primary_index = ranges.len();
                }
                ranges.push(Range::point(new_anchor).put_cursor(text, new_head, true));
            }
        }

        let new_selection = Selection::new(ranges, primary_index);
        doc.set_selection(view_id, new_selection);
    }

    /// Split selection on newlines (`A-s` in Helix).
    fn split_selection_on_newline(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let new_selection = helix_core::selection::split_on_newline(text, &selection);
        doc.set_selection(view_id, new_selection);
    }

    /// Select regex matches within current selection (`s` in Helix).
    fn select_regex(&mut self, doc_id: DocumentId, view_id: ViewId, pattern: &str) {
        let regex = match helix_stdx::rope::Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Invalid regex pattern '{}': {}", pattern, e);
                return;
            }
        };
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        if let Some(new_selection) =
            helix_core::selection::select_on_matches(text, &selection, &regex)
        {
            doc.set_selection(view_id, new_selection);
        }
    }

    /// Split selection on regex matches (`S` in Helix).
    fn split_selection(&mut self, doc_id: DocumentId, view_id: ViewId, pattern: &str) {
        let regex = match helix_stdx::rope::Regex::new(pattern) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Invalid regex pattern '{}': {}", pattern, e);
                return;
            }
        };
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let new_selection = helix_core::selection::split_on_matches(text, &selection, &regex);
        doc.set_selection(view_id, new_selection);
    }

    /// Extend selection to first line (`gg` in select mode).
    fn extend_to_first_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| helix_core::Range::new(range.anchor, 0));

        doc.set_selection(view_id, new_selection);
    }

    /// Extend selection to last line (`ge` in select mode).
    fn extend_to_last_line(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let last_line = text.len_lines().saturating_sub(1);
        let last_line_start = text.line_to_char(last_line);

        let new_selection = selection
            .transform(|range| helix_core::Range::new(range.anchor, last_line_start));

        doc.set_selection(view_id, new_selection);
    }

    /// Extend selection to first non-whitespace on line (`gs` in select mode).
    fn extend_goto_first_nonwhitespace(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let line = text.char_to_line(range.head);
            let line_start = text.line_to_char(line);
            let line_text = text.line(line);
            let offset = line_text
                .chars()
                .take_while(|ch| ch.is_whitespace() && *ch != '\n')
                .count();
            helix_core::Range::new(range.anchor, line_start + offset)
        });

        doc.set_selection(view_id, new_selection);
    }

    /// Extend selection to column 1 (`g|` in select mode).
    fn extend_goto_column(&mut self, doc_id: DocumentId, view_id: ViewId) {
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

    /// Trim whitespace from selection edges (`_` in Helix).
    fn trim_selections(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let ranges: helix_core::SmallVec<[helix_core::Range; 1]> = selection
            .iter()
            .filter_map(|range| {
                if range.is_empty() || range.slice(text).chars().all(|ch| ch.is_whitespace()) {
                    return None;
                }
                let mut start = range.from();
                let mut end = range.to();
                start = helix_core::movement::skip_while(text, start, |x| x.is_whitespace())
                    .unwrap_or(start);
                end = helix_core::movement::backwards_skip_while(text, end, |x| x.is_whitespace())
                    .unwrap_or(end);
                Some(helix_core::Range::new(start, end).with_direction(range.direction()))
            })
            .collect();

        if !ranges.is_empty() {
            let primary = selection.primary();
            let idx = ranges
                .iter()
                .position(|range| range.overlaps(&primary))
                .unwrap_or(ranges.len() - 1);
            doc.set_selection(view_id, helix_core::Selection::new(ranges, idx));
        }
    }

    /// Expand selection to parent syntax node (`Alt-o` in Helix).
    fn expand_selection(&mut self, _doc_id: DocumentId, _view_id: ViewId) {
        let (view, doc) = helix_view::current!(self.editor);
        if let Some(syntax) = doc.syntax() {
            let text = doc.text().slice(..);
            let current_selection = doc.selection(view.id);
            let selection =
                helix_core::object::expand_selection(syntax, text, current_selection.clone());
            if *current_selection != selection {
                view.object_selections.push(current_selection.clone());
                doc.set_selection(view.id, selection);
            }
        }
    }

    /// Shrink selection to child syntax node (`Alt-i` in Helix).
    fn shrink_selection(&mut self, _doc_id: DocumentId, _view_id: ViewId) {
        let (view, doc) = helix_view::current!(self.editor);
        let current_selection = doc.selection(view.id);
        if let Some(prev_selection) = view.object_selections.pop() {
            if current_selection.contains(&prev_selection) {
                doc.set_selection(view.id, prev_selection);
                return;
            }
            view.object_selections.clear();
        }
        if let Some(syntax) = doc.syntax() {
            let text = doc.text().slice(..);
            let selection =
                helix_core::object::shrink_selection(syntax, text, current_selection.clone());
            doc.set_selection(view.id, selection);
        }
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

    // --- extend_selection ---

    #[test]
    fn extend_selection_right() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_selection(doc_id, view_id, Direction::Right);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 0, "anchor should stay at 0");
        assert_eq!(sel.head, 2, "head should move right to 2");
    }

    #[test]
    fn extend_selection_left() {
        let mut ctx = test_context("he#[l|]#lo\n");
        let (doc_id, view_id) = doc_view(&ctx);
        // First set an explicit selection so anchor != head
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(3, 2));
        ctx.extend_selection(doc_id, view_id, Direction::Left);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 3, "anchor should stay at 3");
        assert_eq!(sel.head, 1, "head should move left to 1");
    }

    #[test]
    fn extend_selection_down() {
        let mut ctx = test_context("#[h|]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_selection(doc_id, view_id, Direction::Down);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 0, "anchor should stay at 0");
        // Head should move to second line, same column
        let text = doc.text().slice(..);
        let head_line = text.char_to_line(sel.head);
        assert_eq!(head_line, 1, "head should be on second line");
    }

    #[test]
    fn extend_selection_up_at_first_line_noop() {
        let mut ctx = test_context("hel#[l|]#o\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(0, 3));
        ctx.extend_selection(doc_id, view_id, Direction::Up);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        // At first line, range returned unchanged
        assert_eq!(sel.anchor, 0, "anchor should stay");
        assert_eq!(sel.head, 3, "head should stay (first line, can't go up)");
    }

    // --- extend_word_forward ---

    #[test]
    fn extend_word_forward_basic() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_word_forward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 1, "anchor should stay at 1");
        assert!(sel.head >= 6, "head should move to 'w' or further");
    }

    // --- extend_word_backward ---

    #[test]
    fn extend_word_backward_basic() {
        let mut ctx = test_context("hello #[|w]#orld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_word_backward(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 7, "anchor should stay at 7");
        assert_eq!(sel.head, 0, "head should move to start of 'hello'");
    }

    // --- extend_line_start ---

    #[test]
    fn extend_line_start_basic() {
        let mut ctx = test_context("hel#[l|]#o\n");
        let (doc_id, view_id) = doc_view(&ctx);
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(4, 3));
        ctx.extend_line_start(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 4, "anchor should stay at 4");
        assert_eq!(sel.head, 0, "head should move to line start");
    }

    // --- extend_line_end ---

    #[test]
    fn extend_line_end_basic() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(0, 1));
        ctx.extend_line_end(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 0, "anchor should stay at 0");
        assert_eq!(sel.head, 5, "head should move to line end (newline char)");
    }

    // --- select_all ---

    #[test]
    fn select_all_selects_entire_buffer() {
        let mut ctx = test_context("#[h|]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_all(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.from(), 0, "should start at 0");
        assert_eq!(sel.to(), doc.text().len_chars(), "should end at buffer end");
    }

    // --- flip_selections ---

    #[test]
    fn flip_selections_swaps_anchor_and_head() {
        let mut ctx = test_context("#[hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        let doc = ctx.editor.document(doc_id).expect("doc");
        let before = doc.selection(view_id).primary();
        let (before_anchor, before_head) = (before.anchor, before.head);
        ctx.flip_selections(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, before_head, "anchor should become old head");
        assert_eq!(sel.head, before_anchor, "head should become old anchor");
    }

    // --- extend_to_line_bounds ---

    #[test]
    fn extend_to_line_bounds_single_line() {
        let mut ctx = test_context("he#[l|]#lo\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.extend_to_line_bounds(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.from(), 0, "should extend to line start");
        // Should include the newline (start of next line)
        assert_eq!(sel.to(), 6, "should extend to start of next line");
    }

    // --- shrink_to_line_bounds ---

    #[test]
    fn shrink_to_line_bounds_single_line_noop() {
        let mut ctx = test_context("he#[l|]#lo\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.shrink_to_line_bounds(doc_id, view_id);
        // Single line selection should not change
        assert_state(&ctx, "he#[l|]#lo\nworld\n");
    }

    // --- trim_selections ---

    #[test]
    fn trim_selections_removes_whitespace() {
        let mut ctx = test_context("#[  hello  |]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.trim_selections(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        let text = doc.text().slice(..);
        let selected: String = text.slice(sel.from()..sel.to()).into();
        assert_eq!(selected, "hello", "whitespace should be trimmed");
    }

    // --- select_regex ---

    #[test]
    fn select_regex_finds_matches() {
        let mut ctx = test_context("#[hello world hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_regex(doc_id, view_id, "hello");
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id);
        assert_eq!(sel.len(), 2, "should find 2 matches for 'hello'");
    }

    #[test]
    fn select_regex_invalid_pattern_noop() {
        let mut ctx = test_context("#[hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.select_regex(doc_id, view_id, "[invalid");
        // Should not crash or change selection
        assert_state(&ctx, "#[hello|]#\n");
    }

    // --- split_selection ---

    #[test]
    fn split_selection_on_pattern() {
        let mut ctx = test_context("#[hello world hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.split_selection(doc_id, view_id, " ");
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id);
        assert_eq!(sel.len(), 3, "should split into 3 parts on spaces");
    }

    // --- split_selection_on_newline ---

    // --- extend_to_first_line ---

    #[test]
    fn extend_to_first_line_preserves_anchor() {
        let mut ctx = test_context("hello\nwor#[l|]#d\n");
        let (doc_id, view_id) = doc_view(&ctx);
        // Set anchor at position 9, head at 10
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(9, 10));
        ctx.extend_to_first_line(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 9, "anchor should be preserved");
        assert_eq!(sel.head, 0, "head should move to position 0");
    }

    // --- extend_to_last_line ---

    #[test]
    fn extend_to_last_line_preserves_anchor() {
        let mut ctx = test_context("#[h|]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(0, 1));
        ctx.extend_to_last_line(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 0, "anchor should be preserved");
        let text = doc.text().slice(..);
        let last_line = text.len_lines().saturating_sub(1);
        let last_line_start = text.line_to_char(last_line);
        assert_eq!(sel.head, last_line_start, "head should move to last line start");
    }

    // --- extend_goto_first_nonwhitespace ---

    #[test]
    fn extend_goto_first_nonwhitespace_preserves_anchor() {
        let mut ctx = test_context("  #[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(4, 3));
        ctx.extend_goto_first_nonwhitespace(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 4, "anchor should be preserved");
        assert_eq!(sel.head, 2, "head should move to first non-whitespace");
    }

    // --- extend_goto_column ---

    #[test]
    fn extend_goto_column_preserves_anchor() {
        let mut ctx = test_context("hel#[l|]#o\n");
        let (doc_id, view_id) = doc_view(&ctx);
        let doc = ctx.editor.document_mut(doc_id).expect("doc");
        doc.set_selection(view_id, helix_core::Selection::single(4, 3));
        ctx.extend_goto_column(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        assert_eq!(sel.anchor, 4, "anchor should be preserved");
        assert_eq!(sel.head, 0, "head should move to column 1 (line start)");
    }

    // --- split_selection_on_newline ---

    #[test]
    fn split_selection_on_newline_basic() {
        let mut ctx = test_context("#[hello\nworld\nfoo|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.split_selection_on_newline(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id);
        assert!(sel.len() >= 3, "should split on newlines into at least 3 parts");
    }

    // --- expand_selection / shrink_selection ---

    #[test]
    fn expand_selection_without_syntax_noop() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.expand_selection(doc_id, view_id);
        // No syntax tree → selection unchanged
        assert_state(&ctx, "#[h|]#ello\n");
    }

    #[test]
    fn shrink_selection_without_syntax_noop() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.shrink_selection(doc_id, view_id);
        // No syntax tree, no history → selection unchanged
        assert_state(&ctx, "#[h|]#ello\n");
    }

    #[test]
    fn shrink_selection_pops_from_history() {
        let mut ctx = test_context("#[hello|]#\n");
        let (doc_id, view_id) = doc_view(&ctx);

        // Manually push a previous selection into the object_selections history
        let prev_sel = helix_core::Selection::single(1, 4);
        let (view, _doc) = helix_view::current!(ctx.editor);
        view.object_selections.push(prev_sel.clone());

        ctx.shrink_selection(doc_id, view_id);
        let (_view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view_id).primary();
        // Should have restored the pushed selection
        assert_eq!(sel.anchor, 1, "anchor should be from history");
        assert_eq!(sel.head, 4, "head should be from history");
    }

    #[test]
    fn shrink_selection_clears_stale_history() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);

        // Push a selection that does NOT contain the current one (stale)
        let stale_sel = helix_core::Selection::single(3, 5);
        let (view, _doc) = helix_view::current!(ctx.editor);
        view.object_selections.push(stale_sel);

        ctx.shrink_selection(doc_id, view_id);

        // History should be cleared after encountering stale entry
        let (view, _doc) = helix_view::current_ref!(ctx.editor);
        assert!(
            view.object_selections.is_empty(),
            "stale history should be cleared"
        );
    }
}
