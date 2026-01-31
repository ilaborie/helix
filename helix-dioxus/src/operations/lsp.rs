//! LSP operations for the editor.
//!
//! This module provides LSP-related operations as an extension trait on `EditorContext`.
//! Operations are designed to work with the async/sync bridge - they spawn async tasks
//! that send results back via the command channel.

use helix_view::{DocumentId, ViewId};

use crate::lsp::{DiagnosticSeverity, DiagnosticSnapshot};

// Note: These imports will be used when full LSP client integration is added:
// CompletionItemKind, CompletionItemSnapshot, LocationSnapshot, LspResponse
use crate::state::EditorContext;

/// LSP operations for the editor.
pub trait LspOps {
    /// Jump to the next diagnostic in the document.
    fn next_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId);

    /// Jump to the previous diagnostic in the document.
    fn prev_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId);

    /// Get all diagnostics for the current document.
    #[allow(dead_code)]
    fn get_diagnostics(&self, doc_id: DocumentId) -> Vec<DiagnosticSnapshot>;
}

impl LspOps for EditorContext {
    fn next_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId) {
        // First pass: find the target position and message (immutable borrow)
        let target_info = {
            let Some(doc) = self.editor.document(doc_id) else {
                return;
            };

            let text = doc.text();
            let selection = doc.selection(view_id);
            let cursor = selection.primary().cursor(text.slice(..));
            let cursor_line = text.char_to_line(cursor);

            // Find the next diagnostic after the current cursor position
            let next = doc.diagnostics().iter().find(|d| {
                d.line > cursor_line || (d.line == cursor_line && d.range.start > cursor)
            });

            // If no diagnostic found after cursor, wrap to first diagnostic
            let target = next.or_else(|| doc.diagnostics().first());

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

    fn prev_diagnostic(&mut self, doc_id: DocumentId, view_id: ViewId) {
        // First pass: find the target position and message (immutable borrow)
        let target_info = {
            let Some(doc) = self.editor.document(doc_id) else {
                return;
            };

            let text = doc.text();
            let selection = doc.selection(view_id);
            let cursor = selection.primary().cursor(text.slice(..));
            let cursor_line = text.char_to_line(cursor);

            // Find the previous diagnostic before the current cursor position
            let prev = doc.diagnostics().iter().rev().find(|d| {
                d.line < cursor_line || (d.line == cursor_line && d.range.start < cursor)
            });

            // If no diagnostic found before cursor, wrap to last diagnostic
            let target = prev.or_else(|| doc.diagnostics().last());

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

    fn get_diagnostics(&self, doc_id: DocumentId) -> Vec<DiagnosticSnapshot> {
        let Some(doc) = self.editor.document(doc_id) else {
            return Vec::new();
        };

        let text = doc.text();

        doc.diagnostics()
            .iter()
            .map(|diag| {
                let line = diag.line;
                let line_start = text.line_to_char(line);
                let start_col = diag.range.start.saturating_sub(line_start);
                let end_col = diag.range.end.saturating_sub(line_start);

                DiagnosticSnapshot {
                    line: line + 1, // 1-indexed for display
                    start_col,
                    end_col,
                    message: diag.message.clone(),
                    severity: diag
                        .severity
                        .map(DiagnosticSeverity::from)
                        .unwrap_or_default(),
                    source: diag.source.clone(),
                    code: diag.code.as_ref().map(|c| match c {
                        helix_core::diagnostic::NumberOrString::Number(n) => n.to_string(),
                        helix_core::diagnostic::NumberOrString::String(s) => s.clone(),
                    }),
                }
            })
            .collect()
    }
}

impl EditorContext {
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

// Note: The actual LSP client integration (calling language servers) would require
// additional infrastructure:
//
// 1. Access to the LSP Registry from helix-lsp
// 2. A way to spawn async tasks and send results back (tokio channel)
// 3. Integration with helix-view's language server handling
//
// For now, the operations module provides the UI state management and basic
// diagnostic navigation. Full LSP integration would involve:
//
// - trigger_completion: Request completions from LSP, populate completion_items
// - trigger_hover: Request hover info from LSP, populate hover_content
// - goto_definition: Request definition locations from LSP
// - etc.
//
// These would use patterns like:
//
// ```rust
// let client = self.editor.language_servers.get(&doc)?;
// let future = client.text_document_completion(...);
// tokio::spawn(async move {
//     let result = future.await;
//     command_tx.send(EditorCommand::LspResponse(LspResponse::Completions(result)));
// });
// ```
