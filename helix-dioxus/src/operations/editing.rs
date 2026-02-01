//! Text editing operations for the editor.

use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// Extension trait for text editing operations.
pub trait EditingOps {
    fn insert_char(&mut self, doc_id: DocumentId, view_id: ViewId, c: char);
    fn insert_newline(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn insert_tab(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_char_backward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_char_forward(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn open_line_below(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn open_line_above(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn undo(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn redo(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn toggle_line_comment(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn toggle_block_comment(&mut self, doc_id: DocumentId, view_id: ViewId);
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

    fn insert_newline(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);

        // Get the indentation of the current line
        let line_text = text.line(line);
        let indent: String = line_text
            .chars()
            .take_while(|c| c.is_whitespace() && *c != '\n')
            .collect();

        // Insert newline + indentation at cursor position
        let insert_selection = helix_core::Selection::point(cursor);
        let insert_text = format!("\n{indent}");
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, insert_text.into());
        doc.apply(&transaction, view_id);

        // Position cursor at end of indentation on the new line
        let new_cursor_pos = cursor + 1 + indent.len();
        doc.set_selection(view_id, helix_core::Selection::point(new_cursor_pos));
    }

    fn insert_tab(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone().cursors(text);

        // Use the document's indent style (could be tab or spaces)
        let indent = doc.indent_style.as_str();

        let transaction = helix_core::Transaction::insert(doc.text(), &selection, indent.into());
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

        // Get the indentation of the current line
        let line_text = text.line(line);
        let indent: String = line_text
            .chars()
            .take_while(|c| c.is_whitespace() && *c != '\n')
            .collect();

        // Move to end of line (before newline character)
        let insert_pos = line_end.saturating_sub(1);
        let insert_selection = helix_core::Selection::point(insert_pos);

        // Insert newline + indentation
        let insert_text = format!("\n{}", indent);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, insert_text.into());
        doc.apply(&transaction, view_id);

        // Position cursor at end of indentation on the new line
        let new_cursor_pos = insert_pos + 1 + indent.len();
        doc.set_selection(view_id, helix_core::Selection::point(new_cursor_pos));
    }

    fn open_line_above(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);

        // Get the indentation of the current line
        let line_text = text.line(line);
        let indent: String = line_text
            .chars()
            .take_while(|c| c.is_whitespace() && *c != '\n')
            .collect();

        // Insert indentation + newline at start of current line
        let insert_selection = helix_core::Selection::point(line_start);
        let insert_text = format!("{}\n", indent);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, insert_text.into());
        doc.apply(&transaction, view_id);

        // Move cursor to the end of indentation on the new line (which is now at line_start)
        let new_cursor_pos = line_start + indent.len();
        doc.set_selection(view_id, helix_core::Selection::point(new_cursor_pos));
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

    fn toggle_line_comment(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");

        // Get the comment token from the language configuration
        let comment_token = doc
            .language_config()
            .and_then(|config| config.comment_tokens.as_ref())
            .and_then(|tokens| tokens.first())
            .map(String::as_str);

        let selection = doc.selection(view_id).clone();

        // Use helix_core::comment::toggle_line_comments
        let transaction =
            helix_core::comment::toggle_line_comments(doc.text(), &selection, comment_token);

        doc.apply(&transaction, view_id);
    }

    fn toggle_block_comment(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let doc = self.editor.document_mut(doc_id).expect("doc exists");

        // Get the block comment tokens from the language configuration
        let block_tokens = doc
            .language_config()
            .and_then(|config| config.block_comment_tokens.as_ref());

        let Some(tokens) = block_tokens else {
            log::info!("No block comment tokens configured for this language");
            return;
        };

        let selection = doc.selection(view_id).clone();

        // Use helix_core::comment::toggle_block_comments
        let transaction =
            helix_core::comment::toggle_block_comments(doc.text(), &selection, tokens);

        doc.apply(&transaction, view_id);
    }
}
