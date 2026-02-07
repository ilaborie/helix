//! LSP operations for the editor.
//!
//! This module provides LSP-related operations as an extension trait on `EditorContext`.
//! Operations are designed to work with the async/sync bridge - they spawn async tasks
//! that send results back via the command channel.

use helix_core::movement::Direction;
use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// LSP operations for the editor.
pub trait LspOps {
    /// Jump to the next diagnostic in the document.
    fn next_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId);

    /// Jump to the previous diagnostic in the document.
    fn prev_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId);
}

impl LspOps for EditorContext {
    fn next_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.navigate_diagnostic(doc_id, view_id, Direction::Forward);
    }

    fn prev_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId) {
        self.navigate_diagnostic(doc_id, view_id, Direction::Backward);
    }
}

impl EditorContext {
    /// Navigate to a diagnostic in the given direction.
    fn navigate_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction) {
        // First pass: find the target position and message (immutable borrow)
        let target_info = {
            let Some(doc) = self.editor.document(doc_id) else {
                return;
            };

            let text = doc.text();
            let selection = doc.selection(view_id);
            let cursor = selection.primary().cursor(text.slice(..));
            let cursor_line = text.char_to_line(cursor);

            let target = match direction {
                Direction::Forward => {
                    let next = doc.diagnostics().iter().find(|d| {
                        d.line > cursor_line || (d.line == cursor_line && d.range.start > cursor)
                    });
                    next.or_else(|| doc.diagnostics().first())
                }
                Direction::Backward => {
                    let prev = doc.diagnostics().iter().rev().find(|d| {
                        d.line < cursor_line || (d.line == cursor_line && d.range.start < cursor)
                    });
                    prev.or_else(|| doc.diagnostics().last())
                }
            };

            target.map(|diag| (diag.range.start, diag.line + 1, diag.message.clone()))
        };

        // Second pass: move cursor (mutable borrow)
        if let Some((target_pos, line, message)) = target_info {
            self.goto_char(doc_id, view_id, target_pos);
            log::info!("Jumped to diagnostic at line {}: {}", line, message);
        } else {
            log::info!("No diagnostics in document");
        }
    }

    /// Move cursor to a specific character position.
    fn goto_char(&mut self, doc_id: DocumentId, view_id: ViewId, pos: usize) {
        let doc = match self.editor.document_mut(doc_id) {
            Some(d) => d,
            None => return,
        };

        let text = doc.text();
        let pos = pos.min(text.len_chars().saturating_sub(1));

        let selection = helix_core::Selection::point(pos);
        doc.set_selection(view_id, selection);
    }
}
