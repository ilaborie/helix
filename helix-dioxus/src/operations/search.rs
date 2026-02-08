//! Search operations for the editor.

use helix_core::ropey::Rope;
use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// Collect all line numbers containing matches for the given pattern.
/// Used for scrollbar markers. Case-insensitive search.
///
/// Note: This function converts the Rope to a String for case-insensitive searching.
/// The byte-to-char conversion is O(n) per match. For large documents with many matches,
/// consider using a streaming approach or pre-computing a byte-to-char index.
pub fn collect_search_match_lines(text: &Rope, pattern: &str) -> Vec<usize> {
    if pattern.is_empty() {
        return Vec::new();
    }

    let text_str: String = text.slice(..).into();
    let pattern_lower = pattern.to_lowercase();
    let text_lower = text_str.to_lowercase();
    let mut match_lines = Vec::new();
    let mut last_line = usize::MAX;

    for (byte_idx, _) in text_lower.match_indices(&pattern_lower) {
        // Convert byte index in lowercase string to char index.
        // Note: lowercase preserves char count, so char index is same in both strings.
        // TODO: Consider using Rope's byte_to_char if we can avoid the lowercase conversion,
        // or build a byte-to-char lookup table for better performance with many matches.
        let char_idx = text_lower[..byte_idx].chars().count();
        let line = text.char_to_line(char_idx);
        // Deduplicate: only add each line once
        if line != last_line {
            match_lines.push(line);
            last_line = line;
        }
    }

    match_lines
}

/// Extension trait for search operations.
pub trait SearchOps {
    fn execute_search(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn search_next(&mut self, doc_id: DocumentId, view_id: ViewId, reverse: bool);
    fn do_search(&mut self, doc_id: DocumentId, view_id: ViewId, pattern: &str, backwards: bool);
    fn extend_search_next(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn extend_search_prev(&mut self, doc_id: DocumentId, view_id: ViewId);
}

impl EditorContext {
    /// Search for the word under the cursor and jump to the next occurrence.
    pub(crate) fn search_word_under_cursor(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let word = {
            let doc = self.editor.document(doc_id).expect("doc exists");
            let text = doc.text().slice(..);
            let selection = doc.selection(view_id);
            let cursor = selection.primary().cursor(text);

            // Find word boundaries around cursor
            let mut start = cursor;
            while start > 0 && is_word_char(text.char(start.saturating_sub(1))) {
                start -= 1;
            }
            let len = text.len_chars();
            let mut end = cursor;
            while end < len && is_word_char(text.char(end)) {
                end += 1;
            }

            if start == end {
                return;
            }

            text.slice(start..end).to_string()
        };

        self.last_search = word.clone();
        self.search_backwards = false;
        self.do_search(doc_id, view_id, &word, false);
    }
}

/// Check if a character is part of a word (alphanumeric or underscore).
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

impl SearchOps for EditorContext {
    /// Execute search with current search input.
    fn execute_search(&mut self, doc_id: DocumentId, view_id: ViewId) {
        if self.search_input.is_empty() {
            self.search_mode = false;
            return;
        }

        // Save search pattern for n/N
        self.last_search = self.search_input.clone();

        // Perform the search
        self.do_search(
            doc_id,
            view_id,
            &self.last_search.clone(),
            self.search_backwards,
        );

        self.search_mode = false;
        self.search_input.clear();
    }

    /// Search for next/previous occurrence.
    fn search_next(&mut self, doc_id: DocumentId, view_id: ViewId, reverse: bool) {
        if self.last_search.is_empty() {
            log::info!("No previous search");
            return;
        }

        let backwards = if reverse {
            !self.search_backwards
        } else {
            self.search_backwards
        };

        self.do_search(doc_id, view_id, &self.last_search.clone(), backwards);
    }

    /// Perform the actual search.
    fn do_search(&mut self, doc_id: DocumentId, view_id: ViewId, pattern: &str, backwards: bool) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text();
        let text_slice = text.slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor_char = selection.primary().cursor(text_slice);

        // Convert rope to string for searching
        let text_str: String = text_slice.into();

        // Use Rope's char_to_byte for efficient conversion
        let cursor_byte = text.char_to_byte(cursor_char);
        let pattern_char_len = pattern.chars().count();

        let found_byte_pos = if backwards {
            // Search backwards from cursor
            text_str[..cursor_byte].rfind(pattern)
        } else {
            // Search forwards from cursor + 1 char
            let start_char = (cursor_char + 1).min(text.len_chars());
            let start_byte = text.char_to_byte(start_char);
            text_str[start_byte..]
                .find(pattern)
                .map(|pos| pos + start_byte)
        };

        // Determine final byte position (with wrap-around if needed)
        let final_byte_pos = found_byte_pos.or_else(|| {
            // Wrap around search
            if backwards {
                text_str.rfind(pattern)
            } else {
                text_str.find(pattern)
            }
        });

        if let Some(byte_pos) = final_byte_pos {
            // Convert byte position to char position using Rope
            let char_pos = text.byte_to_char(byte_pos);
            let char_end = char_pos + pattern_char_len;
            let new_selection = helix_core::Selection::single(char_pos, char_end);
            doc.set_selection(view_id, new_selection);
            let wrapped = found_byte_pos.is_none();
            if wrapped {
                log::info!("Wrapped: found '{}' at char position {}", pattern, char_pos);
            } else {
                log::info!("Found '{}' at char position {}", pattern, char_pos);
            }
        } else {
            log::info!("Pattern '{}' not found", pattern);
        }
    }

    /// Extend selection to next search match (n in select mode).
    fn extend_search_next(&mut self, doc_id: DocumentId, view_id: ViewId) {
        if self.last_search.is_empty() {
            log::info!("No previous search");
            return;
        }

        let pattern = self.last_search.clone();
        let backwards = self.search_backwards;

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text();
        let text_slice = text.slice(..);
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();
        let cursor_char = primary.cursor(text_slice);
        let pattern_char_len = pattern.chars().count();

        let text_str: String = text_slice.into();
        let cursor_byte = text.char_to_byte(cursor_char);

        let found_byte_pos = if backwards {
            text_str[..cursor_byte].rfind(&pattern)
        } else {
            let start_char = (cursor_char + 1).min(text.len_chars());
            let start_byte = text.char_to_byte(start_char);
            text_str[start_byte..]
                .find(&pattern)
                .map(|pos| pos + start_byte)
        };

        let final_byte_pos = found_byte_pos.or_else(|| {
            if backwards {
                text_str.rfind(&pattern)
            } else {
                text_str.find(&pattern)
            }
        });

        if let Some(byte_pos) = final_byte_pos {
            let char_pos = text.byte_to_char(byte_pos);
            let char_end = char_pos + pattern_char_len;
            // Extend: preserve original anchor, move head to match end
            let new_head = if backwards { char_pos } else { char_end };
            let new_selection = helix_core::Selection::single(primary.anchor, new_head);
            doc.set_selection(view_id, new_selection);
        }
    }

    /// Extend selection to previous search match (N in select mode).
    fn extend_search_prev(&mut self, doc_id: DocumentId, view_id: ViewId) {
        if self.last_search.is_empty() {
            log::info!("No previous search");
            return;
        }

        let pattern = self.last_search.clone();
        let backwards = !self.search_backwards; // Reverse direction

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text();
        let text_slice = text.slice(..);
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();
        let cursor_char = primary.cursor(text_slice);
        let pattern_char_len = pattern.chars().count();

        let text_str: String = text_slice.into();
        let cursor_byte = text.char_to_byte(cursor_char);

        let found_byte_pos = if backwards {
            text_str[..cursor_byte].rfind(&pattern)
        } else {
            let start_char = (cursor_char + 1).min(text.len_chars());
            let start_byte = text.char_to_byte(start_char);
            text_str[start_byte..]
                .find(&pattern)
                .map(|pos| pos + start_byte)
        };

        let final_byte_pos = found_byte_pos.or_else(|| {
            if backwards {
                text_str.rfind(&pattern)
            } else {
                text_str.find(&pattern)
            }
        });

        if let Some(byte_pos) = final_byte_pos {
            let char_pos = text.byte_to_char(byte_pos);
            let char_end = char_pos + pattern_char_len;
            // Extend: preserve original anchor, move head
            let new_head = if backwards { char_pos } else { char_end };
            let new_selection = helix_core::Selection::single(primary.anchor, new_head);
            doc.set_selection(view_id, new_selection);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_state, doc_view, test_context};

    #[test]
    fn search_word_under_cursor_finds_next_occurrence() {
        let mut ctx = test_context("#[|f]#oo bar foo baz\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.search_word_under_cursor(doc_id, view_id);
        assert_state(&ctx, "foo bar #[foo|]# baz\n");
    }

    #[test]
    fn search_word_under_cursor_wraps_around() {
        let mut ctx = test_context("foo bar #[|f]#oo baz\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.search_word_under_cursor(doc_id, view_id);
        // Should wrap around to the first "foo"
        assert_state(&ctx, "#[foo|]# bar foo baz\n");
    }

    #[test]
    fn search_word_under_cursor_sets_last_search() {
        let mut ctx = test_context("#[|h]#ello world hello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.search_word_under_cursor(doc_id, view_id);
        assert_eq!(ctx.last_search, "hello");
    }
}
