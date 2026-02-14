//! Clipboard operations for the editor.

use helix_view::document::Mode;
use helix_view::{DocumentId, ViewId};

use crate::state::EditorContext;

/// Extension trait for clipboard operations.
pub trait ClipboardOps {
    fn yank(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn yank_main_selection_to_clipboard(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn paste(&mut self, doc_id: DocumentId, view_id: ViewId, before: bool);
    fn delete_selection(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn delete_selection_noyank(&mut self, doc_id: DocumentId, view_id: ViewId);
    fn replace_with_yanked(&mut self, doc_id: DocumentId, view_id: ViewId);
}

impl ClipboardOps for EditorContext {
    /// Yank (copy) the current selection to the selected register.
    fn yank(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let register = self.take_register();

        let doc = self.editor.document(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id);
        let primary = selection.primary();

        // Extract selected text
        let selected_text: String = text.slice(primary.from()..primary.to()).into();

        // Write to the target register
        if let Err(e) = self
            .editor
            .registers
            .write(register, vec![selected_text.clone()])
        {
            log::warn!("Failed to write to register '{register}': {e}");
        }

        // Sync internal clipboard when using '+' register
        if register == '+' {
            self.clipboard.clone_from(&selected_text);
        }

        log::info!(
            "Yanked {} characters to register '{register}'",
            selected_text.len(),
        );
    }

    /// Yank only the primary selection to the selected register.
    ///
    /// Unlike `yank` which yanks whatever range is current, this explicitly
    /// yanks only the primary selection text — useful with multi-cursor.
    fn yank_main_selection_to_clipboard(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let register = self.take_register();

        let doc = self.editor.document(doc_id).expect("doc exists");
        let text = doc.text().slice(..);
        let primary = doc.selection(view_id).primary();

        let selected_text: String = text.slice(primary.from()..primary.to()).into();

        if let Err(e) = self
            .editor
            .registers
            .write(register, vec![selected_text.clone()])
        {
            log::warn!("Failed to write to register '{register}': {e}");
        }

        if register == '+' {
            self.clipboard.clone_from(&selected_text);
        }

        log::info!(
            "Yanked main selection ({} chars) to register '{register}'",
            selected_text.len(),
        );
    }

    /// Paste from the selected register.
    fn paste(&mut self, doc_id: DocumentId, view_id: ViewId, before: bool) {
        let register = self.take_register();

        // Read from target register; for '+' fall back to internal clipboard
        let clipboard_text = self
            .editor
            .registers
            .read(register, &self.editor)
            .and_then(|mut values| values.next().map(std::borrow::Cow::into_owned))
            .or_else(|| {
                if register == '+' {
                    Some(self.clipboard.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

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

        log::info!(
            "Pasted {} characters from register '{register}'",
            clipboard_text.len(),
        );
    }

    /// Replace selection with text from the selected register (without updating that register).
    fn replace_with_yanked(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let register = self.take_register();

        // Read from target register; for '+' fall back to internal clipboard
        let clipboard_text = self
            .editor
            .registers
            .read(register, &self.editor)
            .and_then(|mut values| values.next().map(std::borrow::Cow::into_owned))
            .or_else(|| {
                if register == '+' {
                    Some(self.clipboard.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        if clipboard_text.is_empty() {
            return;
        }

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        let selection = doc.selection(view_id).clone();

        // Replace selection content with register text
        let transaction =
            helix_core::Transaction::change_by_selection(doc.text(), &selection, |range| {
                (
                    range.from(),
                    range.to(),
                    Some(clipboard_text.clone().into()),
                )
            });

        doc.apply(&transaction, view_id);
    }

    /// Delete the current selection without yanking (Alt-d).
    fn delete_selection_noyank(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let (from, to) = {
            let doc = self.editor.document(doc_id).expect("doc exists");
            let primary = doc.selection(view_id).primary();
            (primary.from(), primary.to())
        };

        if from < to {
            let doc = self.editor.document_mut(doc_id).expect("doc exists");
            let ranges = std::iter::once((from, to));
            let transaction = helix_core::Transaction::delete(doc.text(), ranges);
            doc.apply(&transaction, view_id);
        }

        self.editor.mode = Mode::Normal;
    }

    /// Delete the current selection, yanking to the selected register.
    fn delete_selection(&mut self, doc_id: DocumentId, view_id: ViewId) {
        let register = self.take_register();

        // Extract selected text and range (immutable borrow)
        let (selected_text, from, to) = {
            let doc = self.editor.document(doc_id).expect("doc exists");
            let text = doc.text().slice(..);
            let primary = doc.selection(view_id).primary();
            let selected: String = text.slice(primary.from()..primary.to()).into();
            (selected, primary.from(), primary.to())
        };

        // Yank to target register (skip for '_' black hole register)
        if register != '_' {
            if let Err(e) = self
                .editor
                .registers
                .write(register, vec![selected_text.clone()])
            {
                log::warn!("Failed to write to register '{register}': {e}");
            }
            // Sync internal clipboard when using '+' register
            if register == '+' {
                self.clipboard.clone_from(&selected_text);
            }
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

#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_text, doc_view, test_context};

    use super::*;

    #[test]
    fn yank_copies_to_default_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.yank(doc_id, view_id);
        // Default register is '"' (unnamed), not '+'
        let content = ctx
            .editor
            .registers
            .read('"', &ctx.editor)
            .and_then(|mut v| v.next().map(|s| s.into_owned()))
            .unwrap_or_default();
        assert_eq!(content, "hello");
    }

    #[test]
    fn yank_to_named_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.editor.selected_register = Some('a');
        ctx.yank(doc_id, view_id);
        let content = ctx
            .editor
            .registers
            .read('a', &ctx.editor)
            .and_then(|mut v| v.next().map(|s| s.into_owned()))
            .unwrap_or_default();
        assert_eq!(content, "hello");
    }

    #[test]
    fn yank_to_clipboard_syncs_internal() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.editor.selected_register = Some('+');
        ctx.yank(doc_id, view_id);
        assert_eq!(ctx.clipboard, "hello");
    }

    #[test]
    fn paste_from_named_register() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        // Write "world" to register 'a'
        ctx.editor
            .registers
            .write('a', vec!["world".to_string()])
            .expect("write succeeds");
        ctx.editor.selected_register = Some('a');
        ctx.paste(doc_id, view_id, false);
        assert_text(&ctx, "hworldello\n");
    }

    #[test]
    fn delete_selection_to_named_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.editor.selected_register = Some('b');
        ctx.delete_selection(doc_id, view_id);
        let content = ctx
            .editor
            .registers
            .read('b', &ctx.editor)
            .and_then(|mut v| v.next().map(|s| s.into_owned()))
            .unwrap_or_default();
        assert_eq!(content, "hello");
        assert_text(&ctx, " world\n");
    }

    #[test]
    fn delete_selection_black_hole_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.editor.selected_register = Some('_');
        ctx.delete_selection(doc_id, view_id);
        // Black hole register should not store anything
        let content = ctx
            .editor
            .registers
            .read('_', &ctx.editor)
            .and_then(|mut v| v.next().map(|s| s.into_owned()));
        assert!(content.is_none() || content.as_deref() == Some(""));
        assert_text(&ctx, " world\n");
    }

    #[test]
    fn replace_with_yanked_from_named_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        // Write "REPLACED" to register 'c'
        ctx.editor
            .registers
            .write('c', vec!["REPLACED".to_string()])
            .expect("write succeeds");
        ctx.editor.selected_register = Some('c');
        ctx.replace_with_yanked(doc_id, view_id);
        assert_text(&ctx, "REPLACED world\n");
    }

    #[test]
    fn replace_with_yanked_empty_register_noop() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.editor.selected_register = Some('z');
        ctx.replace_with_yanked(doc_id, view_id);
        assert_text(&ctx, "hello world\n");
    }

    // --- delete_selection_noyank ---

    #[test]
    fn delete_selection_noyank_deletes_without_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_selection_noyank(doc_id, view_id);
        assert_text(&ctx, " world\n");
        // Verify nothing was yanked to default register
        let content = ctx
            .editor
            .registers
            .read('"', &ctx.editor)
            .and_then(|mut v| v.next().map(|s| s.into_owned()));
        assert!(
            content.is_none(),
            "should not have written to default register"
        );
    }

    #[test]
    fn delete_selection_noyank_point_selection() {
        let mut ctx = test_context("#[h|]#ello\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.delete_selection_noyank(doc_id, view_id);
        assert_text(&ctx, "ello\n");
    }

    // --- yank_main_selection_to_clipboard ---

    #[test]
    fn yank_main_selection_to_clipboard_copies_primary() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.editor.selected_register = Some('+');
        ctx.yank_main_selection_to_clipboard(doc_id, view_id);
        assert_eq!(ctx.clipboard, "hello");
    }

    #[test]
    fn yank_main_selection_to_clipboard_to_named_register() {
        let mut ctx = test_context("#[hello|]# world\n");
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.editor.selected_register = Some('a');
        ctx.yank_main_selection_to_clipboard(doc_id, view_id);
        let content = ctx
            .editor
            .registers
            .read('a', &ctx.editor)
            .and_then(|mut v| v.next().map(|s| s.into_owned()))
            .unwrap_or_default();
        assert_eq!(content, "hello");
    }
}
