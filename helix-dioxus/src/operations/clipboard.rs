//! Clipboard operations for the editor.

use helix_view::document::Mode;
use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// Extension trait for clipboard operations.
pub trait ClipboardOps {
    fn yank(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn paste(&mut self, doc_id: DocumentId, view_id: ViewId, before: bool);
    fn delete_selection(&mut self, doc_id: DocumentId, view_id: ViewId);
}

impl ClipboardOps for EditorContext {
    /// Yank (copy) the current selection to clipboard.
    fn yank(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id);
        let primary = selection.primary();

        // Extract selected text
        let selected_text: String = text.slice(primary.from()..primary.to()).into();
        self.clipboard = selected_text;

        log::info!("Yanked {} characters", self.clipboard.len());
    }

    /// Paste from clipboard.
    fn paste(&mut self, doc_id: DocumentId, view_id: ViewId, before: bool) {
        if self.clipboard.is_empty() {
            return;
        }

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let pos = if before {
            selection.primary().from()
        } else {
            selection.primary().to()
        };

        // Check if clipboard ends with newline (line-wise paste)
        let is_linewise = self.clipboard.ends_with('\n');

        let insert_pos = if is_linewise && !before {
            // For line-wise paste after, move to start of next line
            let line = text.char_to_line(pos);
            if line + 1 < text.len_lines() {
                text.line_to_char(line + 1)
            } else {
                text.len_chars()
            }
        } else if is_linewise && before {
            // For line-wise paste before, move to start of current line
            let line = text.char_to_line(pos);
            text.line_to_char(line)
        } else {
            pos
        };

        let insert_selection = helix_core::Selection::point(insert_pos);
        let transaction = helix_core::Transaction::insert(
            doc.text(),
            &insert_selection,
            self.clipboard.clone().into(),
        );
        doc.apply(&transaction, view_id);

        log::info!("Pasted {} characters", self.clipboard.len());
    }

    /// Delete the current selection.
    fn delete_selection(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let primary = selection.primary();

        // First yank the selection
        let selected_text: String = text.slice(primary.from()..primary.to()).into();
        self.clipboard = selected_text;

        // Delete the selection
        let from = primary.from();
        let to = primary.to();

        if from < to {
            let ranges = std::iter::once((from, to));
            let transaction = helix_core::Transaction::delete(doc.text(), ranges);
            doc.apply(&transaction, view_id);
        }

        // Return to normal mode
        self.editor.mode = Mode::Normal;
    }
}
