//! Word jump (EasyMotion-style) operations.
//!
//! Provides `gw` command that shows labels on words in the viewport for
//! quick navigation by typing a two-character label.

use helix_core::Selection;
use helix_view::{DocumentId, ViewId};

use crate::state::{EditorContext, WordJumpLabel};

/// Extension trait for word jump operations.
pub trait WordJumpOps {
    /// Compute word jump labels for visible words in the viewport.
    fn compute_word_jump_labels(&mut self, doc_id: DocumentId, view_id: ViewId);

    /// Filter labels by first character — dim non-matching labels.
    fn filter_word_jump_first_char(&mut self, ch: char);

    /// Execute word jump — find the label matching the two-char sequence, jump to that word.
    fn execute_word_jump(&mut self, ch: char);

    /// Cancel word jump and clear all state.
    fn cancel_word_jump(&mut self);
}

impl WordJumpOps for EditorContext {
    #[allow(clippy::indexing_slicing)] // indices bounded by max_labels = alphabet.len()^2
    fn compute_word_jump_labels(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let Some(doc) = self.editor.document(doc_id) else {
            return;
        };
        let text = doc.text();
        let total_chars = text.len_chars();
        if total_chars == 0 {
            return;
        }

        // Get visible range from view offset (same calculation as snapshot())
        let view_offset = doc.view_offset(view_id);
        let visible_start_char = view_offset.anchor.min(total_chars);
        let visible_start_line = text.char_to_line(visible_start_char);
        // Use 40 lines to match the viewport_lines used in snapshot()
        let visible_end_line = (visible_start_line + 40).min(text.len_lines());
        let visible_end_char = if visible_end_line < text.len_lines() {
            text.line_to_char(visible_end_line)
        } else {
            total_chars
        };

        // Scan for word starts in the visible range
        let label_alphabet: Vec<char> = "abcdefghijklmnopqrstuvwxyz".chars().collect();
        let max_labels = label_alphabet.len() * label_alphabet.len(); // 676

        let mut word_positions: Vec<(usize, usize, usize)> = Vec::new(); // (line_1indexed, col, char_pos)

        let slice = text.slice(..);
        let mut pos = visible_start_char;

        while pos < visible_end_char && word_positions.len() < max_labels {
            let ch = slice.char(pos);

            // Check if we're at a word start
            if is_word_char(ch) {
                let is_word_start = if pos == 0 {
                    true
                } else {
                    let prev = slice.char(pos - 1);
                    !is_word_char(prev)
                };

                if is_word_start {
                    // Check word is at least 2 chars (worth labeling)
                    let mut end = pos + 1;
                    while end < visible_end_char && is_word_char(slice.char(end)) {
                        end += 1;
                    }
                    if end - pos >= 2 {
                        let line = text.char_to_line(pos);
                        let line_start = text.line_to_char(line);
                        let col = pos - line_start;
                        word_positions.push((line + 1, col, pos)); // 1-indexed line
                    }
                }
            }
            pos += 1;
        }

        // Generate two-character labels
        let mut labels = Vec::with_capacity(word_positions.len());
        let mut ranges = Vec::with_capacity(word_positions.len());

        for (i, (line, col, char_pos)) in word_positions.iter().enumerate() {
            if i >= max_labels {
                break;
            }
            let first = label_alphabet[i / label_alphabet.len()];
            let second = label_alphabet[i % label_alphabet.len()];
            labels.push(WordJumpLabel {
                line: *line,
                col: *col,
                label: [first, second],
                dimmed: false,
            });
            ranges.push(*char_pos);
        }

        self.word_jump_labels = labels;
        self.word_jump_ranges = ranges;
        self.word_jump_active = true;
        self.word_jump_first_idx = None;
    }

    fn filter_word_jump_first_char(&mut self, ch: char) {
        let ch_lower = ch.to_ascii_lowercase();

        // Find all labels whose first char matches
        let mut any_match = false;
        for label in &mut self.word_jump_labels {
            if label.label[0] == ch_lower {
                label.dimmed = false;
                any_match = true;
            } else {
                label.dimmed = true;
            }
        }

        if any_match {
            self.word_jump_first_idx = Some(ch_lower);
        } else {
            self.cancel_word_jump();
        }
    }

    fn execute_word_jump(&mut self, ch: char) {
        let ch_lower = ch.to_ascii_lowercase();
        let Some(first) = self.word_jump_first_idx else {
            self.cancel_word_jump();
            return;
        };

        // Find the label with matching [first, second]
        let target_idx = self
            .word_jump_labels
            .iter()
            .position(|l| l.label[0] == first && l.label[1] == ch_lower);

        if let Some(idx) = target_idx {
            if let Some(&char_pos) = self.word_jump_ranges.get(idx) {
                let extend = self.word_jump_extend;
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;

                let doc = self.editor.document_mut(doc_id).expect("doc exists");
                let text = doc.text().slice(..);

                if extend {
                    // Extend selection to the word
                    let selection = doc.selection(view_id).clone();
                    let primary = selection.primary();
                    let new_range = helix_core::Range::new(primary.anchor, char_pos.min(text.len_chars()));
                    let new_selection = selection
                        .clone()
                        .transform(|range| if range == primary { new_range } else { range });
                    doc.set_selection(view_id, new_selection);
                } else {
                    doc.set_selection(view_id, Selection::point(char_pos));
                }
            }
        }

        self.cancel_word_jump();
    }

    fn cancel_word_jump(&mut self) {
        self.word_jump_active = false;
        self.word_jump_labels.clear();
        self.word_jump_ranges.clear();
        self.word_jump_extend = false;
        self.word_jump_first_idx = None;
    }
}

/// Check if a character is a "word character" (alphanumeric or underscore).
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{assert_state, doc_view, test_context};

    #[test]
    fn is_word_char_alpha() {
        assert!(is_word_char('a'));
        assert!(is_word_char('Z'));
        assert!(is_word_char('_'));
        assert!(is_word_char('0'));
    }

    #[test]
    fn is_word_char_non_word() {
        assert!(!is_word_char(' '));
        assert!(!is_word_char('.'));
        assert!(!is_word_char('('));
        assert!(!is_word_char('\n'));
    }

    // --- compute_word_jump_labels ---

    #[test]
    fn compute_labels_activates_word_jump() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);

        ctx.compute_word_jump_labels(doc_id, view_id);

        assert!(ctx.word_jump_active);
        assert!(ctx.word_jump_first_idx.is_none());
    }

    #[test]
    fn compute_labels_finds_words() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);

        ctx.compute_word_jump_labels(doc_id, view_id);

        assert_eq!(ctx.word_jump_labels.len(), 2);
        assert_eq!(ctx.word_jump_ranges.len(), 2);
    }

    #[test]
    fn compute_labels_generates_two_char_labels() {
        let mut ctx = test_context("#[|h]#ello world foo\n");
        let (doc_id, view_id) = doc_view(&ctx);

        ctx.compute_word_jump_labels(doc_id, view_id);

        assert_eq!(ctx.word_jump_labels[0].label, ['a', 'a']);
        assert_eq!(ctx.word_jump_labels[1].label, ['a', 'b']);
        assert_eq!(ctx.word_jump_labels[2].label, ['a', 'c']);
    }

    #[test]
    fn compute_labels_skips_single_char_words() {
        let mut ctx = test_context("#[|a]# bb cc\n");
        let (doc_id, view_id) = doc_view(&ctx);

        ctx.compute_word_jump_labels(doc_id, view_id);

        // "a" is 1 char → skipped, "bb" and "cc" are 2+ chars
        assert_eq!(ctx.word_jump_labels.len(), 2);
    }

    #[test]
    fn compute_labels_empty_doc() {
        let mut ctx = test_context("#[| ]#");
        let (doc_id, view_id) = doc_view(&ctx);

        ctx.compute_word_jump_labels(doc_id, view_id);

        // A space-only doc has no words, but word_jump_active is still set
        assert!(ctx.word_jump_labels.is_empty());
    }

    #[test]
    fn compute_labels_records_line_and_col() {
        let mut ctx = test_context("#[|h]#ello\n  world\n");
        let (doc_id, view_id) = doc_view(&ctx);

        ctx.compute_word_jump_labels(doc_id, view_id);

        assert_eq!(ctx.word_jump_labels.len(), 2);
        // "hello" at line 1, col 0
        assert_eq!(ctx.word_jump_labels[0].line, 1);
        assert_eq!(ctx.word_jump_labels[0].col, 0);
        // "world" at line 2, col 2
        assert_eq!(ctx.word_jump_labels[1].line, 2);
        assert_eq!(ctx.word_jump_labels[1].col, 2);
    }

    #[test]
    fn compute_labels_not_dimmed_initially() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);

        ctx.compute_word_jump_labels(doc_id, view_id);

        for label in &ctx.word_jump_labels {
            assert!(!label.dimmed);
        }
    }

    // --- filter_word_jump_first_char ---

    #[test]
    fn filter_first_char_sets_first_idx() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);

        ctx.filter_word_jump_first_char('a');

        assert_eq!(ctx.word_jump_first_idx, Some('a'));
        assert!(ctx.word_jump_active);
    }

    #[test]
    fn filter_first_char_dims_non_matching() {
        // Need 27+ words so labels span 'a_' and 'b_' prefixes
        let text = "aa bb cc dd ee ff gg hh ii jj kk ll mm nn oo pp qq rr ss tt uu vv ww xx yy zz aaa";
        let mut ctx = test_context(&format!("#[|{text}]#\n"));
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);
        assert!(ctx.word_jump_labels.len() > 26, "need >26 labels for multi-prefix");

        ctx.filter_word_jump_first_char('b');

        // Labels starting with 'b' are undimmed, others dimmed
        for label in &ctx.word_jump_labels {
            if label.label[0] == 'b' {
                assert!(!label.dimmed, "label {:?} should not be dimmed", label.label);
            } else {
                assert!(label.dimmed, "label {:?} should be dimmed", label.label);
            }
        }
    }

    #[test]
    fn filter_first_char_no_match_cancels() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);

        // Only 2 labels, both start with 'a'; 'z' won't match
        ctx.filter_word_jump_first_char('z');

        assert!(!ctx.word_jump_active);
        assert!(ctx.word_jump_labels.is_empty());
    }

    #[test]
    fn filter_first_char_case_insensitive() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);

        ctx.filter_word_jump_first_char('A');

        assert_eq!(ctx.word_jump_first_idx, Some('a'));
    }

    // --- Regression: few labels all share first char ---

    #[test]
    fn filter_first_char_few_labels_all_same_prefix() {
        // Regression: with <27 words, ALL labels start with 'a'.
        // After filtering by 'a', all labels are undimmed but first_idx must be set.
        let mut ctx = test_context("#[|h]#ello world foo bar\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);

        // All labels start with 'a'
        assert!(ctx.word_jump_labels.iter().all(|l| l.label[0] == 'a'));

        ctx.filter_word_jump_first_char('a');

        // first_idx MUST be set even though all labels remain undimmed
        assert_eq!(ctx.word_jump_first_idx, Some('a'));
        assert!(ctx.word_jump_active);
        assert!(ctx.word_jump_labels.iter().all(|l| !l.dimmed));
    }

    // --- execute_word_jump ---

    #[test]
    fn execute_jump_moves_cursor() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);
        ctx.filter_word_jump_first_char('a');

        // label 'ab' = index 1 = "world" at char_pos 6
        ctx.execute_word_jump('b');

        assert_state(&ctx, "hello #[w|]#orld\n");
    }

    #[test]
    fn execute_jump_first_label() {
        let mut ctx = test_context("hello #[|w]#orld\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);
        ctx.filter_word_jump_first_char('a');

        // label 'aa' = index 0 = "hello" at char_pos 0
        ctx.execute_word_jump('a');

        assert_state(&ctx, "#[h|]#ello world\n");
    }

    #[test]
    fn execute_jump_clears_state() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);
        ctx.filter_word_jump_first_char('a');
        ctx.execute_word_jump('a');

        assert!(!ctx.word_jump_active);
        assert!(ctx.word_jump_labels.is_empty());
        assert!(ctx.word_jump_first_idx.is_none());
    }

    #[test]
    fn execute_jump_no_first_idx_cancels() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);
        // Skip filter_word_jump_first_char — first_idx is None

        ctx.execute_word_jump('a');

        // Should cancel without moving — selection unchanged
        assert!(!ctx.word_jump_active);
        assert_state(&ctx, "#[|h]#ello world\n");
    }

    #[test]
    fn execute_jump_invalid_second_char_no_move() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);
        ctx.filter_word_jump_first_char('a');

        // Only 2 labels: aa, ab. 'z' doesn't match any.
        ctx.execute_word_jump('z');

        // Cursor stays at original position, state cleared
        assert!(!ctx.word_jump_active);
        assert_state(&ctx, "#[|h]#ello world\n");
    }

    // --- Regression: full two-char sequence with few labels ---

    #[test]
    fn full_sequence_few_labels_jumps_correctly() {
        // Regression test for the bug: with <27 words, all labels start with 'a'.
        // The full sequence (compute → filter 'a' → execute second char) must work.
        let mut ctx = test_context("#[|f]#n main() {\n    let foo = bar;\n}\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);

        let label_count = ctx.word_jump_labels.len();
        assert!(label_count < 27, "must have <27 labels for this regression test");
        assert!(label_count >= 2, "need at least 2 labels");

        // All labels start with 'a'
        assert!(ctx.word_jump_labels.iter().all(|l| l.label[0] == 'a'));

        // Step 1: filter by first char 'a'
        ctx.filter_word_jump_first_char('a');
        assert_eq!(ctx.word_jump_first_idx, Some('a'));
        assert!(ctx.word_jump_active);

        // Step 2: type second char — pick the last label to jump far
        let last_label = ctx.word_jump_labels.last().expect("has labels");
        let second_char = last_label.label[1];
        let target_pos = *ctx.word_jump_ranges.last().expect("has ranges");

        ctx.execute_word_jump(second_char);

        // Cursor must have moved to the target position
        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id);
        let head = sel.primary().cursor(doc.text().slice(..));
        assert_eq!(head, target_pos, "cursor should jump to target word");
    }

    // --- extend mode ---

    #[test]
    fn execute_jump_extend_preserves_anchor() {
        // Use #[h|]# so anchor=0, head=1
        let mut ctx = test_context("#[h|]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.word_jump_extend = true;
        ctx.compute_word_jump_labels(doc_id, view_id);
        ctx.filter_word_jump_first_char('a');

        // Jump to "world" at pos 6 — should extend from anchor 0
        ctx.execute_word_jump('b');

        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id);
        let primary = sel.primary();
        assert_eq!(primary.anchor, 0);
        assert_eq!(primary.head, 6);
    }

    // --- cancel ---

    #[test]
    fn cancel_clears_all_state() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);
        ctx.filter_word_jump_first_char('a');

        ctx.cancel_word_jump();

        assert!(!ctx.word_jump_active);
        assert!(ctx.word_jump_labels.is_empty());
        assert!(ctx.word_jump_ranges.is_empty());
        assert!(!ctx.word_jump_extend);
        assert!(ctx.word_jump_first_idx.is_none());
    }

    // --- snapshot integration ---

    #[test]
    fn snapshot_reflects_first_char_state() {
        let mut ctx = test_context("#[|h]#ello world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.compute_word_jump_labels(doc_id, view_id);

        let snap1 = ctx.snapshot(40);
        assert!(snap1.word_jump_active);
        assert!(snap1.word_jump_first_char.is_none());

        ctx.filter_word_jump_first_char('a');

        let snap2 = ctx.snapshot(40);
        assert!(snap2.word_jump_active);
        assert_eq!(snap2.word_jump_first_char, Some('a'));
    }
}
