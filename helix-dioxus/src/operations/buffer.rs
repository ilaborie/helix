//! Buffer management operations for the editor.

use std::path::{Path, PathBuf};

use helix_view::document::DocumentSavedEvent;
use helix_view::DocumentId;

use crate::state::{BufferInfo, ConfirmationAction, ConfirmationDialogSnapshot, EditorContext, NotificationSeverity};

/// Build a `ConfirmationDialogSnapshot` with common structure.
fn build_confirmation_dialog(
    title: &str,
    message: &str,
    confirm_label: &str,
    deny_label: Option<&str>,
    action: ConfirmationAction,
) -> ConfirmationDialogSnapshot {
    ConfirmationDialogSnapshot {
        title: title.to_string(),
        message: message.to_string(),
        confirm_label: confirm_label.to_string(),
        deny_label: deny_label.map(str::to_string),
        cancel_label: "Cancel".to_string(),
        action,
    }
}

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

    // Additional buffer management operations
    fn reload_document(&mut self);
    fn write_all(&mut self);
    fn quit_all(&mut self, force: bool);
    fn buffer_close_all(&mut self, force: bool);
    fn buffer_close_others(&mut self);
    fn change_directory(&mut self, path: &Path);
    fn print_working_directory(&mut self);
}

impl BufferOps for EditorContext {
    /// Switch to a specific buffer.
    fn switch_to_buffer(&mut self, doc_id: DocumentId) {
        self.editor.switch(doc_id, helix_view::editor::Action::Replace);
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
                    self.confirmation_dialog = build_confirmation_dialog(
                        "Close Buffer",
                        &format!("\"{file_name}\" has unsaved changes. Do you want to close it anyway?"),
                        "Close",
                        None,
                        ConfirmationAction::CloseBuffer,
                    );
                    self.confirmation_dialog_visible = true;
                    return;
                }
            }
        }

        let _ = self.editor.close_document(doc_id, force);
    }

    /// Cycle through buffers.
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)] // Buffer count is small; index arithmetic is safe
    fn cycle_buffer(&mut self, direction: i32) {
        let doc_ids: Vec<DocumentId> = self.editor.documents.keys().copied().collect();
        if doc_ids.is_empty() {
            return;
        }

        let current_doc_id = self.editor.tree.get(self.editor.tree.focus).doc;

        let current_idx = doc_ids.iter().position(|&id| id == current_doc_id).unwrap_or(0);

        let len = doc_ids.len() as i32;
        let new_idx = ((current_idx as i32 + direction).rem_euclid(len)) as usize;

        if let Some(&new_doc_id) = doc_ids.get(new_idx) {
            self.switch_to_buffer(new_doc_id);
        }
    }

    /// Get buffer bar snapshot for rendering.
    fn buffer_bar_snapshot(&mut self) -> (Vec<BufferInfo>, usize) {
        const MAX_VISIBLE_TABS: usize = 8;

        let current_doc_id = self.editor.tree.get(self.editor.tree.focus).doc;

        let buffers: Vec<BufferInfo> = self
            .editor
            .documents
            .iter()
            .map(|(&id, doc)| BufferInfo {
                id,
                name: doc.display_name().into_owned(),
                path: doc.path().map(|p| p.to_string_lossy().into_owned()),
                is_modified: doc.is_modified(),
                is_current: id == current_doc_id,
            })
            .collect();

        // Auto-scroll to make current buffer visible
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
                log::info!("Opened file: {}", path.display());
            }
            Err(e) => {
                log::error!("Failed to open file {}: {e}", path.display());
                self.show_notification(format!("Failed to open file: {e}"), NotificationSeverity::Error);
            }
        }
    }

    /// Save the current document.
    /// If path is None, saves to the document's existing path.
    fn save_document(&mut self, path: Option<PathBuf>, force: bool) {
        let view_id = self.editor.tree.focus;
        let doc_id = self.editor.tree.get(view_id).doc;

        match self.save_doc_inner(doc_id, path, force) {
            Ok(event) => {
                // Set the document path (important for Save As on scratch buffers)
                if let Some(doc) = self.editor.document_mut(doc_id) {
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
                self.show_notification(format!("Saved to {display_path}"), NotificationSeverity::Success);
            }
            Err(e) => {
                self.show_notification(format!("Save failed: {e}"), NotificationSeverity::Error);
            }
        }
    }

    /// Try to quit the editor.
    /// If force is false and there are unsaved changes, shows a confirmation dialog.
    fn try_quit(&mut self, force: bool) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let Some(doc) = self.editor.document(view.doc) else {
            self.should_quit = true;
            return;
        };

        if doc.is_modified() && !force {
            log::warn!("Unsaved changes - showing confirmation dialog");
            let file_name = doc.display_name().to_string();
            self.confirmation_dialog = build_confirmation_dialog(
                "Unsaved Changes",
                &format!("\"{file_name}\" has unsaved changes. What would you like to do?"),
                "Save & Quit",
                Some("Don't Save"),
                ConfirmationAction::SaveAndQuit,
            );
            self.confirmation_dialog_visible = true;
            return;
        }

        self.should_quit = true;
        log::info!("Quit command executed");
    }

    /// Parse a document ID from its display string and find the matching document.
    fn parse_document_id(&self, id_str: &str) -> Option<DocumentId> {
        self.editor
            .documents
            .keys()
            .find(|&&doc_id| doc_id.to_string() == id_str)
            .copied()
    }

    /// Create a new scratch buffer.
    fn create_new_buffer(&mut self) {
        self.editor.new_file(helix_view::editor::Action::Replace);
    }

    /// Reload the current document from disk.
    fn reload_document(&mut self) {
        let view_id = self.editor.tree.focus;
        let doc_id = self.editor.tree.get(view_id).doc;

        if let Some(doc) = self.editor.document(doc_id) {
            if let Some(path) = doc.path().cloned() {
                match self.editor.open(&path, helix_view::editor::Action::Replace) {
                    Ok(_) => {
                        log::info!("Reloaded document from {}", path.display());
                        self.show_notification("File reloaded".to_string(), NotificationSeverity::Info);
                    }
                    Err(e) => {
                        log::error!("Failed to reload document: {e}");
                        self.show_notification(format!("Failed to reload: {e}"), NotificationSeverity::Error);
                    }
                }
            } else {
                self.show_notification(
                    "Cannot reload: buffer has no file path".to_string(),
                    NotificationSeverity::Warning,
                );
            }
        }
    }

    /// Save all modified buffers.
    fn write_all(&mut self) {
        let doc_ids: Vec<_> = self
            .editor
            .documents()
            .filter(|d| d.is_modified() && d.path().is_some())
            .map(helix_view::Document::id)
            .collect();

        if doc_ids.is_empty() {
            self.show_notification("No modified buffers to save".to_string(), NotificationSeverity::Info);
            return;
        }

        let mut saved_count = 0;
        let mut error_count = 0;

        for doc_id in doc_ids {
            match self.save_doc_inner(doc_id, None, false) {
                Ok(event) => {
                    if let Some(doc) = self.editor.document_mut(doc_id) {
                        doc.set_last_saved_revision(event.revision, event.save_time);
                    }
                    saved_count += 1;
                }
                Err(e) => {
                    log::error!("Save failed for {doc_id:?}: {e}");
                    error_count += 1;
                }
            }
        }

        if error_count > 0 {
            self.show_notification(
                format!("Saved {saved_count} files, {error_count} errors"),
                NotificationSeverity::Warning,
            );
        } else {
            self.show_notification(format!("Saved {saved_count} files"), NotificationSeverity::Success);
        }
    }

    /// Quit all buffers and exit.
    fn quit_all(&mut self, force: bool) {
        // Check for unsaved changes in any buffer
        if !force {
            let has_unsaved = self.editor.documents().any(helix_view::Document::is_modified);
            if has_unsaved {
                // Count modified buffers
                let modified_count = self.editor.documents().filter(|d| d.is_modified()).count();
                self.confirmation_dialog = build_confirmation_dialog(
                    "Unsaved Changes",
                    &format!("{modified_count} buffer(s) have unsaved changes. What would you like to do?"),
                    "Save All & Quit",
                    Some("Discard All"),
                    ConfirmationAction::SaveAndQuit,
                );
                self.confirmation_dialog_visible = true;
                return;
            }
        }

        self.should_quit = true;
        log::info!("Quit all command executed");
    }

    /// Close all buffers.
    fn buffer_close_all(&mut self, force: bool) {
        // Check for unsaved changes if not forcing
        if !force {
            let has_unsaved = self.editor.documents().any(helix_view::Document::is_modified);
            if has_unsaved {
                self.show_notification(
                    "Some buffers have unsaved changes. Use :bca! to force close.".to_string(),
                    NotificationSeverity::Warning,
                );
                return;
            }
        }

        let doc_ids: Vec<_> = self.editor.documents().map(helix_view::Document::id).collect();
        for doc_id in doc_ids {
            let _ = self.editor.close_document(doc_id, force);
        }

        // Create a new scratch buffer if all were closed
        if self.editor.documents.is_empty() {
            self.editor.new_file(helix_view::editor::Action::Replace);
        }
    }

    /// Close all buffers except the current one.
    fn buffer_close_others(&mut self) {
        let view_id = self.editor.tree.focus;
        let current_doc_id = self.editor.tree.get(view_id).doc;

        let other_ids: Vec<_> = self
            .editor
            .documents()
            .map(helix_view::Document::id)
            .filter(|&id| id != current_doc_id)
            .collect();

        let mut closed_count = 0;
        for doc_id in other_ids {
            if let Ok(()) = self.editor.close_document(doc_id, false) {
                closed_count += 1;
            }
        }

        if closed_count > 0 {
            self.show_notification(format!("Closed {closed_count} buffer(s)"), NotificationSeverity::Info);
        }
    }

    /// Change the current working directory.
    fn change_directory(&mut self, path: &Path) {
        match std::env::set_current_dir(path) {
            Ok(()) => {
                if let Ok(cwd) = std::env::current_dir() {
                    self.show_notification(format!("Changed to {}", cwd.display()), NotificationSeverity::Info);
                }
            }
            Err(e) => {
                self.show_notification(format!("Failed to cd: {e}"), NotificationSeverity::Error);
            }
        }
    }

    /// Print the current working directory.
    fn print_working_directory(&mut self) {
        match std::env::current_dir() {
            Ok(cwd) => {
                self.show_notification(cwd.display().to_string(), NotificationSeverity::Info);
            }
            Err(e) => {
                self.show_notification(format!("Failed to get cwd: {e}"), NotificationSeverity::Error);
            }
        }
    }
}

impl EditorContext {
    /// Flush history and save a single document. Returns the save event on success.
    fn save_doc_inner(
        &mut self,
        doc_id: DocumentId,
        path: Option<PathBuf>,
        force: bool,
    ) -> anyhow::Result<DocumentSavedEvent> {
        // Flush pending changes to history
        let view_id = self.editor.tree.focus;
        {
            let view = self.editor.tree.get_mut(view_id);
            if let Some(doc) = self.editor.documents.get_mut(&doc_id) {
                doc.append_changes_to_history(view);
            }
        }

        // Initiate save
        let save_future = {
            let doc = self
                .editor
                .document_mut(doc_id)
                .ok_or_else(|| anyhow::anyhow!("No document to save"))?;
            doc.save::<PathBuf>(path, force)?
        };

        // Block on the async save
        let event = futures::executor::block_on(save_future)?;
        log::info!("Saved to {}", event.path.display());
        Ok(event)
    }
}
