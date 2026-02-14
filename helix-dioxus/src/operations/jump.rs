//! Jump list operations for the editor.

use std::path::PathBuf;

use helix_view::editor::Action;

use crate::state::{EditorContext, NotificationSeverity, PickerIcon, PickerItem, PickerMode};

/// Extension trait for jump list operations.
pub trait JumpOps {
    /// Push the current position onto the jump list.
    fn push_jump(&mut self);
    /// Jump backward through position history (C-o).
    fn jump_backward(&mut self);
    /// Jump forward through position history (C-i).
    fn jump_forward(&mut self);
    /// Save the current position to the jump list and show a notification (C-s).
    fn save_selection(&mut self);
    /// Show the jump list picker (Space j).
    fn show_jumplist_picker(&mut self);
    /// Clear all entries from the jump list.
    fn clear_jumplist(&mut self);
}

impl JumpOps for EditorContext {
    fn push_jump(&mut self) {
        let (view, doc) = helix_view::current!(self.editor);
        let jump = (doc.id(), doc.selection(view.id).clone());
        view.jumps.push(jump);
    }

    fn jump_backward(&mut self) {
        let (view, doc) = helix_view::current!(self.editor);
        let view_id = view.id;

        if let Some(&(doc_id, ref selection)) = view.jumps.backward(view_id, doc, 1) {
            let selection = selection.clone();
            let current_doc_id = doc.id();

            if doc_id != current_doc_id {
                self.editor.switch(doc_id, Action::Replace);
            }

            let (view, doc) = helix_view::current!(self.editor);
            doc.set_selection(view.id, selection);
            helix_view::align_view(doc, view, helix_view::Align::Center);
        }
    }

    fn jump_forward(&mut self) {
        let (view, doc) = helix_view::current!(self.editor);

        if let Some(&(doc_id, ref selection)) = view.jumps.forward(1) {
            let selection = selection.clone();
            let current_doc_id = doc.id();

            if doc_id != current_doc_id {
                self.editor.switch(doc_id, Action::Replace);
            }

            let (view, doc) = helix_view::current!(self.editor);
            doc.set_selection(view.id, selection);
            helix_view::align_view(doc, view, helix_view::Align::Center);
        }
    }

    fn save_selection(&mut self) {
        self.push_jump();
        self.show_notification(
            "Position saved to jump list".to_string(),
            NotificationSeverity::Info,
        );
    }

    fn clear_jumplist(&mut self) {
        let view = self.editor.tree.get_mut(self.editor.tree.focus);
        view.jumps.clear();
        self.show_notification(
            "Jump list cleared".to_string(),
            NotificationSeverity::Info,
        );
    }

    fn show_jumplist_picker(&mut self) {
        let (view, _doc) = helix_view::current_ref!(self.editor);

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let entries: Vec<_> = view
            .jumps
            .iter()
            .rev()
            .filter_map(|(doc_id, selection)| {
                let doc = self.editor.document(*doc_id)?;
                let text = doc.text().slice(..);
                let primary = selection.primary();
                let cursor = primary.cursor(text);
                let line = text.char_to_line(cursor);
                let col = cursor - text.line_to_char(line);

                let path_display = doc
                    .path()
                    .map(|p| {
                        p.strip_prefix(&cwd)
                            .unwrap_or(p)
                            .to_string_lossy()
                            .to_string()
                    })
                    .unwrap_or_else(|| doc.display_name().to_string());

                // Get line content for secondary text
                let line_content = text
                    .line(line)
                    .to_string()
                    .trim()
                    .chars()
                    .take(80)
                    .collect::<String>();

                Some((
                    *doc_id,
                    selection.clone(),
                    path_display,
                    line + 1,
                    col + 1,
                    line_content,
                ))
            })
            .collect();

        let items: Vec<PickerItem> = entries
            .iter()
            .enumerate()
            .map(
                |(idx, (_doc_id, _sel, path, line, col, content))| PickerItem {
                    id: idx.to_string(),
                    display: format!("{path}:{line}:{col}"),
                    icon: PickerIcon::JumpEntry,
                    match_indices: vec![],
                    secondary: Some(content.clone()),
                },
            )
            .collect();

        // Store entries for confirm handler
        self.jumplist_entries = entries
            .into_iter()
            .map(|(doc_id, sel, _, _, _, _)| (doc_id, sel))
            .collect();

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::JumpList;
        self.last_picker_mode = Some(PickerMode::JumpList);
        self.picker_current_path = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::MovementOps;
    use crate::test_helpers::{assert_state, doc_view, test_context};

    use super::*;

    #[test]
    fn push_jump_saves_current_position() {
        let mut ctx = test_context("#[h|]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);

        // Move to a different position, then push twice to build up the list
        ctx.push_jump();
        ctx.move_cursor(doc_id, view_id, crate::state::Direction::Down);
        ctx.push_jump();

        let (view, _doc) = helix_view::current_ref!(ctx.editor);
        let jumps: Vec<_> = view.jumps.iter().collect();
        assert!(
            jumps.len() >= 2,
            "expected at least 2 jumps, got {}",
            jumps.len()
        );
    }

    #[test]
    fn jump_backward_restores_position() {
        let mut ctx = test_context("#[h|]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);

        // Save position at start
        ctx.push_jump();

        // Move to last line
        ctx.goto_last_line(doc_id, view_id);

        // Push the new position so backward has something to return to
        ctx.push_jump();

        // Jump backward should restore to somewhere near the start
        ctx.jump_backward();

        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id).primary();
        let text = doc.text().slice(..);
        let cursor_line = text.char_to_line(sel.cursor(text));
        assert_eq!(cursor_line, 0, "should jump back to first line");
    }

    #[test]
    fn jump_forward_after_backward() {
        let mut ctx = test_context("#[h|]#ello\nworld\nfoo\n");
        let (doc_id, view_id) = doc_view(&ctx);

        // Save starting position
        ctx.push_jump();

        // Move to last line and save
        ctx.goto_last_line(doc_id, view_id);
        ctx.push_jump();

        // Jump backward
        ctx.jump_backward();

        // Jump forward should return to last line
        ctx.jump_forward();

        let (view, doc) = helix_view::current_ref!(ctx.editor);
        let sel = doc.selection(view.id).primary();
        let text = doc.text().slice(..);
        let cursor_line = text.char_to_line(sel.cursor(text));
        let last_line = text.len_lines().saturating_sub(1);
        assert_eq!(cursor_line, last_line, "should jump forward to last line");
    }

    #[test]
    fn jump_backward_empty_list_is_noop() {
        let mut ctx = test_context("#[h|]#ello\n");
        // Jump backward on fresh context should not panic
        ctx.jump_backward();
        assert_state(&ctx, "#[h|]#ello\n");
    }

    #[test]
    fn jump_forward_at_end_is_noop() {
        let mut ctx = test_context("#[h|]#ello\n");
        // Jump forward at end of list should not panic
        ctx.jump_forward();
        assert_state(&ctx, "#[h|]#ello\n");
    }

    #[test]
    fn clear_jumplist_removes_all_entries() {
        let _guard = crate::test_helpers::init();
        let mut ctx = test_context("#[h|]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);

        // Build up the jump list
        ctx.push_jump();
        ctx.move_cursor(doc_id, view_id, crate::state::Direction::Down);
        ctx.push_jump();

        // Verify we have jumps
        let (view, _doc) = helix_view::current_ref!(ctx.editor);
        assert!(
            view.jumps.iter().count() >= 2,
            "expected at least 2 jumps before clearing"
        );

        // Clear the jump list
        ctx.clear_jumplist();

        // Verify the jump list is empty
        let (view, _doc) = helix_view::current_ref!(ctx.editor);
        assert_eq!(view.jumps.iter().count(), 0, "jump list should be empty");
    }

    #[test]
    fn show_jumplist_picker_populates_items() {
        let mut ctx = test_context("#[h|]#ello\nworld\n");

        // Push a couple of jumps
        ctx.push_jump();
        let (doc_id, view_id) = doc_view(&ctx);
        ctx.goto_last_line(doc_id, view_id);
        ctx.push_jump();

        ctx.show_jumplist_picker();

        assert!(ctx.picker_visible);
        assert_eq!(ctx.picker_mode, PickerMode::JumpList);
        assert!(!ctx.picker_items.is_empty(), "picker should have items");
        assert!(
            !ctx.jumplist_entries.is_empty(),
            "jumplist_entries should be populated"
        );
    }
}
