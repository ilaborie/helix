//! Buffer management operations for the editor.

use std::path::PathBuf;

use helix_view::DocumentId;

use crate::state::{
    BufferInfo, ConfirmationAction, ConfirmationDialogSnapshot, EditorContext, NotificationSeverity,
};

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
    fn create_new_buffer(&mut self);
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
                    log::warn!("Buffer has unsaved changes - showing confirmation dialog");
                    let file_name = doc.display_name().to_string();
                    self.confirmation_dialog = ConfirmationDialogSnapshot {
                        title: "Close Buffer".to_string(),
                        message: format!(
                            "\"{file_name}\" has unsaved changes. Do you want to close it anyway?"
                        ),
                        confirm_label: "Close".to_string(),
                        deny_label: None, // Only confirm/cancel for close buffer
                        cancel_label: "Cancel".to_string(),
                        action: ConfirmationAction::CloseBuffer,
                    };
                    self.confirmation_dialog_visible = true;
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
                self.show_notification(
                    format!("Failed to open file: {}", e),
                    NotificationSeverity::Error,
                );
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
                    self.show_notification(
                        "No document to save".to_string(),
                        NotificationSeverity::Error,
                    );
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
                    self.show_notification(
                        format!("Failed to save: {}", e),
                        NotificationSeverity::Error,
                    );
                    return;
                }
            }
        };

        // Block on the async save operation
        match futures::executor::block_on(save_future) {
            Ok(event) => {
                log::info!("Saved to {:?}", event.path);
                // Update the document's path and modified state
                if let Some(doc) = self.editor.document_mut(doc_id) {
                    // Set the document path (important for Save As on scratch buffers)
                    doc.set_path(Some(&event.path));
                    doc.set_last_saved_revision(event.revision, event.save_time);
                }
                // Use relative path if shorter than absolute path
                let display_path = if let Ok(cwd) = std::env::current_dir() {
                    if let Ok(relative) = event.path.strip_prefix(&cwd) {
                        let relative_str = relative.to_string_lossy();
                        let absolute_str = event.path.to_string_lossy();
                        if relative_str.len() < absolute_str.len() {
                            relative_str.into_owned()
                        } else {
                            absolute_str.into_owned()
                        }
                    } else {
                        event.path.to_string_lossy().into_owned()
                    }
                } else {
                    event.path.to_string_lossy().into_owned()
                };
                self.show_notification(
                    format!("Saved to {}", display_path),
                    NotificationSeverity::Success,
                );
            }
            Err(e) => {
                log::error!("Save failed: {}", e);
                self.show_notification(format!("Save failed: {}", e), NotificationSeverity::Error);
            }
        }
    }

    /// Try to quit the editor.
    /// If force is false and there are unsaved changes, shows a confirmation dialog.
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
            log::warn!("Unsaved changes - showing confirmation dialog");
            let file_name = doc.display_name().to_string();
            self.confirmation_dialog = ConfirmationDialogSnapshot {
                title: "Unsaved Changes".to_string(),
                message: format!("\"{file_name}\" has unsaved changes. What would you like to do?"),
                confirm_label: "Save & Quit".to_string(),
                deny_label: Some("Don't Save".to_string()),
                cancel_label: "Cancel".to_string(),
                action: ConfirmationAction::SaveAndQuit,
            };
            self.confirmation_dialog_visible = true;
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

    /// Create a new scratch buffer.
    fn create_new_buffer(&mut self) {
        self.editor.new_file(helix_view::editor::Action::Replace);
    }
}
