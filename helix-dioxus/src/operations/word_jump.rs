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
    fn compute_word_jump_labels(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = match self.editor.document(doc_id) {
            Some(doc) => doc,
            None => return,
        };
        let text = doc.text();
        let total_chars = text.len_chars();
        if total_chars == 0 {
            return;
        }

        // Get visible range from view offset
        let view_offset = doc.view_offset(view_id);
        let visible_start_char = view_offset.anchor.min(total_chars);
        let visible_start_line = text.char_to_line(visible_start_char);
        // Estimate visible lines from editor area (use 50 as reasonable default)
        let visible_end_line = (visible_start_line + 50).min(text.len_lines());
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
            // No match — cancel
            self.cancel_word_jump();
        }
    }

    fn execute_word_jump(&mut self, ch: char) {
        let ch_lower = ch.to_ascii_lowercase();
        let first = match self.word_jump_first_idx {
            Some(f) => f,
            None => {
                self.cancel_word_jump();
                return;
            }
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
                    let new_range =
                        helix_core::Range::new(primary.anchor, char_pos.min(text.len_chars()));
                    let new_selection = selection.clone().transform(|range| {
                        if range == primary {
                            new_range
                        } else {
                            range
                        }
                    });
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
}
