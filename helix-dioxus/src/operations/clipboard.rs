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
        self.clipboard.clone_from(&selected_text);

        // Write to system clipboard via '+' register
        if let Err(e) = self.editor.registers.write('+', vec![selected_text]) {
            log::warn!("Failed to write to system clipboard: {e}");
        }

        log::info!("Yanked {} characters", self.clipboard.len());
    }

    /// Paste from clipboard.
    fn paste(&mut self, doc_id: DocumentId, view_id: ViewId, before: bool) {
        // Read from system clipboard via '+' register, fall back to internal
        let clipboard_text = self
            .editor
            .registers
            .read('+', &self.editor)
            .and_then(|mut values| values.next().map(|v| v.into_owned()))
            .unwrap_or_else(|| self.clipboard.clone());

        if clipboard_text.is_empty() {
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
        let is_linewise = clipboard_text.ends_with('\n');

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
            clipboard_text.clone().into(),
        );
        doc.apply(&transaction, view_id);

        log::info!("Pasted {} characters", clipboard_text.len());
    }

    /// Delete the current selection.
    fn delete_selection(&mut self, doc_id: DocumentId, view_id: ViewId) {
        // Extract selected text and range (immutable borrow)
        let (selected_text, from, to) = {
            let doc = self.editor.document(doc_id).expect("doc exists");
            let text = doc.text().slice(..);
            let primary = doc.selection(view_id).primary();
            let selected: String = text.slice(primary.from()..primary.to()).into();
            (selected, primary.from(), primary.to())
        };

        // Yank to internal and system clipboard
        self.clipboard.clone_from(&selected_text);
        if let Err(e) = self.editor.registers.write('+', vec![selected_text]) {
            log::warn!("Failed to write to system clipboard: {e}");
        }

        // Delete the selection
        if from < to {
            let doc = self.editor.document_mut(doc_id).expect("doc exists");
            let ranges = std::iter::once((from, to));
            let transaction = helix_core::Transaction::delete(doc.text(), ranges);
            doc.apply(&transaction, view_id);
        }

        // Return to normal mode
        self.editor.mode = Mode::Normal;
    }
}
