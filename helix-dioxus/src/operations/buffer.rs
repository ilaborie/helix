//! Buffer management operations for the editor.

use std::path::PathBuf;

use helix_view::DocumentId;

use crate::state::{BufferInfo, EditorContext};

/// Extension trait for buffer management operations.
pub trait BufferOps {
    fn switch_to_buffer(&mut self, doc_id: DocumentId);
    fn close_buffer(&mut self, doc_id: DocumentId);
    fn close_current_buffer(&mut self, force: bool);
    fn cycle_buffer(&mut self, direction: i32);
    fn buffer_bar_snapshot(&mut self) -> (Vec<BufferInfo>, usize);
    fn open_file(&mut self, path: &std::path::Path);
    fn save_document(&mut self, path: Option<PathBuf>, force: bool);
    fn try_quit(&mut self, force: bool);
    fn parse_document_id(&self, id_str: &str) -> Option<DocumentId>;
}

impl BufferOps for EditorContext {
    /// Switch to a specific buffer.
    fn switch_to_buffer(&mut self, doc_id: DocumentId) {
        self.editor
            .switch(doc_id, helix_view::editor::Action::Replace);
    }

    /// Close a buffer.
    fn close_buffer(&mut self, doc_id: DocumentId) {
        let _ = self.editor.close_document(doc_id, false);
    }

    /// Close the current buffer.
    fn close_current_buffer(&mut self, force: bool) {
        let view_id = self.editor.tree.focus;
        let doc_id = self.editor.tree.get(view_id).doc;

        if !force {
            if let Some(doc) = self.editor.document(doc_id) {
                if doc.is_modified() {
                    log::warn!("Buffer has unsaved changes. Use :bd! to force close.");
                    return;
                }
            }
        }

        let _ = self.editor.close_document(doc_id, force);
    }

    /// Cycle through buffers.
    fn cycle_buffer(&mut self, direction: i32) {
        let doc_ids: Vec<DocumentId> = self.editor.documents.keys().copied().collect();
        if doc_ids.is_empty() {
            return;
        }

        let current_doc_id = self.editor.tree.get(self.editor.tree.focus).doc;

        let current_idx = doc_ids
            .iter()
            .position(|&id| id == current_doc_id)
            .unwrap_or(0);

        let len = doc_ids.len() as i32;
        let new_idx = ((current_idx as i32 + direction).rem_euclid(len)) as usize;

        if let Some(&new_doc_id) = doc_ids.get(new_idx) {
            self.switch_to_buffer(new_doc_id);
        }
    }

    /// Get buffer bar snapshot for rendering.
    fn buffer_bar_snapshot(&mut self) -> (Vec<BufferInfo>, usize) {
        let current_doc_id = self.editor.tree.get(self.editor.tree.focus).doc;

        let buffers: Vec<BufferInfo> = self
            .editor
            .documents
            .iter()
            .map(|(&id, doc)| BufferInfo {
                id,
                name: doc.display_name().into_owned(),
                is_modified: doc.is_modified(),
                is_current: id == current_doc_id,
            })
            .collect();

        // Auto-scroll to make current buffer visible (max 8 visible tabs)
        const MAX_VISIBLE_TABS: usize = 8;
        if let Some(current_idx) = buffers.iter().position(|b| b.is_current) {
            if current_idx < self.buffer_bar_scroll {
                // Current buffer is to the left of visible area
                self.buffer_bar_scroll = current_idx;
            } else if current_idx >= self.buffer_bar_scroll + MAX_VISIBLE_TABS {
                // Current buffer is to the right of visible area
                self.buffer_bar_scroll = current_idx.saturating_sub(MAX_VISIBLE_TABS - 1);
            }
        }

        (buffers, self.buffer_bar_scroll)
    }

    /// Open a file in the editor.
    fn open_file(&mut self, path: &std::path::Path) {
        let path = helix_stdx::path::canonicalize(path);
        match self.editor.open(&path, helix_view::editor::Action::Replace) {
            Ok(_) => {
                log::info!("Opened file: {:?}", path);
            }
            Err(e) => {
                log::error!("Failed to open file {:?}: {}", path, e);
            }
        }
    }

    /// Save the current document.
    /// If path is None, saves to the document's existing path.
    fn save_document(&mut self, path: Option<PathBuf>, force: bool) {
        let view_id = self.editor.tree.focus;
        let doc_id = self.editor.tree.get(view_id).doc;

        // Flush pending changes to history before saving
        // This ensures is_modified() returns false after save
        {
            let view = self.editor.tree.get_mut(view_id);
            let doc = match self.editor.documents.get_mut(&doc_id) {
                Some(doc) => doc,
                None => {
                    log::error!("No document to save");
                    return;
                }
            };
            doc.append_changes_to_history(view);
        }

        // Get the save future in a separate scope to release the borrow
        let save_future = {
            let doc = match self.editor.document_mut(doc_id) {
                Some(doc) => doc,
                None => {
                    log::error!("No document to save");
                    return;
                }
            };

            match doc.save::<PathBuf>(path, force) {
                Ok(future) => future,
                Err(e) => {
                    log::error!("Failed to initiate save: {}", e);
                    return;
                }
            }
        };

        // Block on the async save operation
        match futures::executor::block_on(save_future) {
            Ok(event) => {
                log::info!("Saved to {:?}", event.path);
                // Update the document's modified state
                if let Some(doc) = self.editor.document_mut(doc_id) {
                    doc.set_last_saved_revision(event.revision, event.save_time);
                }
            }
            Err(e) => {
                log::error!("Save failed: {}", e);
            }
        }
    }

    /// Try to quit the editor.
    /// If force is false and there are unsaved changes, logs a warning and does not quit.
    fn try_quit(&mut self, force: bool) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc = match self.editor.document(view.doc) {
            Some(doc) => doc,
            None => {
                self.should_quit = true;
                return;
            }
        };

        if doc.is_modified() && !force {
            log::warn!("Unsaved changes. Use :q! to force quit.");
            return;
        }

        self.should_quit = true;
        log::info!("Quit command executed");
    }

    /// Parse a document ID from its debug string representation.
    fn parse_document_id(&self, id_str: &str) -> Option<DocumentId> {
        // The id is stored directly, we need to find the matching document
        for (&doc_id, _) in &self.editor.documents {
            if format!("{:?}", doc_id) == id_str {
                return Some(doc_id);
            }
        }
        None
    }
}
