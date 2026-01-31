//! Text editing operations for the editor.

use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// Extension trait for text editing operations.
pub trait EditingOps {
    fn insert_char(&mut self, doc_id: DocumentId, view_id: ViewId, c: char);
    fn delete_char_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_char_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn open_line_below(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn open_line_above(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn undo(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn redo(&mut self, doc_id: DocumentId, view_id: ViewId);
}

impl EditingOps for EditorContext {
    fn insert_char(&mut self, doc_id: DocumentId, view_id: ViewId, c: char) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone().cursors(text);

        let transaction =
            helix_core::Transaction::insert(doc.text(), &selection, c.to_string().into());
        doc.apply(&transaction, view_id);
    }

    fn delete_char_backward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        if cursor == 0 {
            return;
        }

        let ranges = std::iter::once((cursor - 1, cursor));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    fn delete_char_forward(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        if cursor >= text.len_chars() {
            return;
        }

        let ranges = std::iter::once((cursor, cursor + 1));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    fn open_line_below(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_end = text.line_to_char(line) + text.line(line).len_chars();

        // Move to end of line
        let new_selection = helix_core::Selection::point(line_end.saturating_sub(1));
        doc.set_selection(view_id, new_selection.clone());

        // Insert newline
        let transaction = helix_core::Transaction::insert(doc.text(), &new_selection, "\n".into());
        doc.apply(&transaction, view_id);
    }

    fn open_line_above(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);

        // Insert newline at start of current line
        let insert_selection = helix_core::Selection::point(line_start);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, "\n".into());
        doc.apply(&transaction, view_id);

        // Move cursor to the new empty line
        doc.set_selection(view_id, helix_core::Selection::point(line_start));
    }

    fn undo(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let view = self.editor.tree.get_mut(view_id);
        let doc = self.editor.documents.get_mut(&doc_id).expect("doc exists");
        if !doc.undo(view) {
            log::info!("Already at oldest change");
        }
    }

    fn redo(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let view = self.editor.tree.get_mut(view_id);
        let doc = self.editor.documents.get_mut(&doc_id).expect("doc exists");
        if !doc.redo(view) {
            log::info!("Already at newest change");
        }
    }
}
