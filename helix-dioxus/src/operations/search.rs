//! Search operations for the editor.

use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// Extension trait for search operations.
pub trait SearchOps {
    fn execute_search(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn search_next(&mut self, doc_id: DocumentId, view_id: ViewId, reverse: bool);
    fn do_search(&mut self, doc_id: DocumentId, view_id: ViewId, pattern: &str, backwards: bool);
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
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let cursor = selection.primary().cursor(text);

        // Simple substring search
        let text_str: String = text.into();

        let found_pos = if backwards {
            // Search backwards from cursor
            text_str[..cursor].rfind(pattern)
        } else {
            // Search forwards from cursor + 1
            let start = (cursor + 1).min(text_str.len());
            text_str[start..].find(pattern).map(|pos| pos + start)
        };

        if let Some(pos) = found_pos {
            // Move cursor to the found position
            let new_selection = helix_core::Selection::single(pos, pos + pattern.len());
            doc.set_selection(view_id, new_selection);
            log::info!("Found '{}' at position {}", pattern, pos);
        } else {
            // Wrap around search
            let wrap_pos = if backwards {
                text_str.rfind(pattern)
            } else {
                text_str.find(pattern)
            };

            if let Some(pos) = wrap_pos {
                let new_selection = helix_core::Selection::single(pos, pos + pattern.len());
                doc.set_selection(view_id, new_selection);
                log::info!("Wrapped: found '{}' at position {}", pattern, pos);
            } else {
                log::info!("Pattern '{}' not found", pattern);
            }
        }
    }
}
