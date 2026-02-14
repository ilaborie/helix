//! VCS (Version Control System) operations.
//!
//! Provides hunk navigation (`]g`/`[g`), first/last change (`]G`/`[G`),
//! and changed files picker (`Space g`).

use std::sync::mpsc;
use std::time::Duration;

use helix_core::Selection;
use helix_vcs::FileChange;
use helix_view::document::Mode;
use helix_view::DocumentId;
use imara_diff::Hunk;

use super::JumpOps;
use crate::state::{EditorContext, NotificationSeverity, PickerIcon, PickerItem, PickerMode};

/// VCS operations as an extension trait on `EditorContext`.
pub trait VcsOps {
    /// Jump to the next change hunk from cursor.
    fn goto_next_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId);
    /// Jump to the previous change hunk from cursor.
    fn goto_prev_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId);
    /// Jump to the first change hunk in the document.
    fn goto_first_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId);
    /// Jump to the last change hunk in the document.
    fn goto_last_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId);
    /// Show a picker listing all changed files in the working directory.
    fn show_changed_files_picker(&mut self);
}

/// Compute the selection `Range` for a hunk in the given text.
///
/// Additions and modifications cover the changed line range.
/// Deletions are represented as a point at the deletion location.
fn hunk_range(hunk: Hunk, text: helix_core::RopeSlice) -> helix_core::Range {
    let anchor = text.line_to_char(hunk.after.start as usize);
    let head = if hunk.after.is_empty() {
        anchor + 1
    } else {
        text.line_to_char(hunk.after.end as usize)
    };
    helix_core::Range::new(anchor, head)
}

/// Convert a `FileChange` to a `PickerItem` for the changed files picker.
fn file_change_to_picker_item(change: &FileChange, cwd: &std::path::Path) -> PickerItem {
    match change {
        FileChange::Untracked { path } => {
            let display = path_display(path, cwd);
            PickerItem {
                id: path.to_string_lossy().to_string(),
                display: format!("+ {display}"),
                icon: PickerIcon::VcsAdded,
                secondary: Some("untracked".to_string()),
                ..Default::default()
            }
        }
        FileChange::Modified { path } => {
            let display = path_display(path, cwd);
            PickerItem {
                id: path.to_string_lossy().to_string(),
                display: format!("~ {display}"),
                icon: PickerIcon::VcsModified,
                secondary: Some("modified".to_string()),
                ..Default::default()
            }
        }
        FileChange::Conflict { path } => {
            let display = path_display(path, cwd);
            PickerItem {
                id: path.to_string_lossy().to_string(),
                display: format!("x {display}"),
                icon: PickerIcon::VcsConflict,
                secondary: Some("conflict".to_string()),
                ..Default::default()
            }
        }
        FileChange::Deleted { path } => {
            let display = path_display(path, cwd);
            PickerItem {
                id: path.to_string_lossy().to_string(),
                display: format!("- {display}"),
                icon: PickerIcon::VcsDeleted,
                secondary: Some("deleted".to_string()),
                ..Default::default()
            }
        }
        FileChange::Renamed { from_path, to_path } => {
            let from_display = path_display(from_path, cwd);
            let to_display = path_display(to_path, cwd);
            PickerItem {
                id: to_path.to_string_lossy().to_string(),
                display: format!("> {from_display} → {to_display}"),
                icon: PickerIcon::VcsRenamed,
                secondary: Some("renamed".to_string()),
                ..Default::default()
            }
        }
    }
}

/// Display a path relative to `cwd` if possible, otherwise as-is.
fn path_display(path: &std::path::Path, cwd: &std::path::Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

impl VcsOps for EditorContext {
    fn goto_next_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId) {
        let doc = match self.editor.document(doc_id) {
            Some(doc) => doc,
            None => return,
        };

        let diff_handle = match doc.diff_handle() {
            Some(handle) => handle.clone(),
            None => {
                self.show_notification(
                    "Diff not available in current buffer".to_string(),
                    NotificationSeverity::Info,
                );
                return;
            }
        };

        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let is_select = self.editor.mode == Mode::Select;

        let new_selection = selection.transform(|range| {
            let cursor_line = range.cursor_line(text) as u32;
            let diff = diff_handle.load();
            let Some(hunk_idx) = diff.next_hunk(cursor_line) else {
                return range;
            };
            let hunk = diff.nth_hunk(hunk_idx);
            let new_range = hunk_range(hunk, text);
            if is_select {
                let head = if new_range.head < range.anchor {
                    new_range.anchor
                } else {
                    new_range.head
                };
                helix_core::Range::new(range.anchor, head)
            } else {
                new_range
            }
        });

        self.push_jump();
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        doc.set_selection(view_id, new_selection);
    }

    fn goto_prev_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId) {
        let doc = match self.editor.document(doc_id) {
            Some(doc) => doc,
            None => return,
        };

        let diff_handle = match doc.diff_handle() {
            Some(handle) => handle.clone(),
            None => {
                self.show_notification(
                    "Diff not available in current buffer".to_string(),
                    NotificationSeverity::Info,
                );
                return;
            }
        };

        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();
        let is_select = self.editor.mode == Mode::Select;

        let new_selection = selection.transform(|range| {
            let cursor_line = range.cursor_line(text) as u32;
            let diff = diff_handle.load();
            let Some(hunk_idx) = diff.prev_hunk(cursor_line) else {
                return range;
            };
            let hunk = diff.nth_hunk(hunk_idx);
            let new_range = hunk_range(hunk, text);
            if is_select {
                let head = if new_range.head < range.anchor {
                    new_range.anchor
                } else {
                    new_range.head
                };
                helix_core::Range::new(range.anchor, head)
            } else {
                new_range
            }
        });

        self.push_jump();
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        doc.set_selection(view_id, new_selection);
    }

    fn goto_first_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId) {
        let doc = match self.editor.document(doc_id) {
            Some(doc) => doc,
            None => return,
        };

        let handle = match doc.diff_handle() {
            Some(h) => h.clone(),
            None => return,
        };

        let diff = handle.load();
        let hunk = diff.nth_hunk(0);
        if hunk == Hunk::NONE {
            return;
        }

        let text = doc.text().slice(..);
        let range = hunk_range(hunk, text);

        self.push_jump();
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        doc.set_selection(view_id, Selection::single(range.anchor, range.head));
    }

    fn goto_last_change(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId) {
        let doc = match self.editor.document(doc_id) {
            Some(doc) => doc,
            None => return,
        };

        let handle = match doc.diff_handle() {
            Some(h) => h.clone(),
            None => return,
        };

        let diff = handle.load();
        if diff.len() == 0 {
            return;
        }
        let hunk = diff.nth_hunk(diff.len() - 1);
        if hunk == Hunk::NONE {
            return;
        }

        let text = doc.text().slice(..);
        let range = hunk_range(hunk, text);

        self.push_jump();
        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        doc.set_selection(view_id, Selection::single(range.anchor, range.head));
    }

    fn show_changed_files_picker(&mut self) {
        let cwd = match std::env::current_dir() {
            Ok(cwd) => cwd,
            Err(_) => {
                self.show_notification(
                    "Cannot determine current directory".to_string(),
                    NotificationSeverity::Error,
                );
                return;
            }
        };

        let providers = self.editor.diff_providers.clone();
        let (tx, rx) = mpsc::channel::<FileChange>();

        providers.for_each_changed_file(cwd.clone(), move |result| match result {
            Ok(change) => tx.send(change).is_ok(),
            Err(_) => false,
        });

        // Collect results with timeout
        let mut changes = Vec::new();
        let timeout = Duration::from_secs(5);
        while let Ok(change) = rx.recv_timeout(timeout) {
            changes.push(change);
        }

        if changes.is_empty() {
            self.show_notification("No changed files".to_string(), NotificationSeverity::Info);
            return;
        }

        let items: Vec<PickerItem> = changes
            .iter()
            .map(|c| file_change_to_picker_item(c, &cwd))
            .collect();

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_mode = PickerMode::ChangedFiles;
        self.picker_visible = true;
        self.last_picker_mode = Some(PickerMode::ChangedFiles);
        self.picker_current_path = None;
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use helix_core::Rope;
    use helix_vcs::FileChange;
    use imara_diff::Hunk;

    use super::*;

    fn test_text() -> Rope {
        // 5 lines: "line0\nline1\nline2\nline3\nline4\n"
        Rope::from("line0\nline1\nline2\nline3\nline4\n")
    }

    #[test]
    fn hunk_range_addition() {
        let rope = test_text();
        let text = rope.slice(..);
        // Hunk: added lines 1..3 (lines 1 and 2)
        let hunk = Hunk {
            before: 0..0,
            after: 1..3,
        };
        let range = hunk_range(hunk, text);
        assert_eq!(range.anchor, text.line_to_char(1));
        assert_eq!(range.head, text.line_to_char(3));
    }

    #[test]
    fn hunk_range_deletion() {
        let rope = test_text();
        let text = rope.slice(..);
        // Hunk: deletion at line 2 (after range is empty)
        let hunk = Hunk {
            before: 1..3,
            after: 2..2,
        };
        let range = hunk_range(hunk, text);
        let anchor = text.line_to_char(2);
        assert_eq!(range.anchor, anchor);
        assert_eq!(range.head, anchor + 1);
    }

    #[test]
    fn hunk_range_modification() {
        let rope = test_text();
        let text = rope.slice(..);
        // Hunk: modified line 0..1
        let hunk = Hunk {
            before: 0..1,
            after: 0..1,
        };
        let range = hunk_range(hunk, text);
        assert_eq!(range.anchor, text.line_to_char(0));
        assert_eq!(range.head, text.line_to_char(1));
    }

    #[test]
    fn file_change_to_picker_item_untracked() {
        let cwd = PathBuf::from("/project");
        let change = FileChange::Untracked {
            path: PathBuf::from("/project/new_file.rs"),
        };
        let item = file_change_to_picker_item(&change, &cwd);
        assert_eq!(item.icon, PickerIcon::VcsAdded);
        assert!(item.display.contains("new_file.rs"));
        assert!(item.display.starts_with("+ "));
        assert_eq!(item.secondary.as_deref(), Some("untracked"));
    }

    #[test]
    fn file_change_to_picker_item_modified() {
        let cwd = PathBuf::from("/project");
        let change = FileChange::Modified {
            path: PathBuf::from("/project/src/lib.rs"),
        };
        let item = file_change_to_picker_item(&change, &cwd);
        assert_eq!(item.icon, PickerIcon::VcsModified);
        assert!(item.display.starts_with("~ "));
        assert_eq!(item.secondary.as_deref(), Some("modified"));
    }

    #[test]
    fn file_change_to_picker_item_deleted() {
        let cwd = PathBuf::from("/project");
        let change = FileChange::Deleted {
            path: PathBuf::from("/project/old.rs"),
        };
        let item = file_change_to_picker_item(&change, &cwd);
        assert_eq!(item.icon, PickerIcon::VcsDeleted);
        assert!(item.display.starts_with("- "));
    }

    #[test]
    fn file_change_to_picker_item_renamed() {
        let cwd = PathBuf::from("/project");
        let change = FileChange::Renamed {
            from_path: PathBuf::from("/project/old.rs"),
            to_path: PathBuf::from("/project/new.rs"),
        };
        let item = file_change_to_picker_item(&change, &cwd);
        assert_eq!(item.icon, PickerIcon::VcsRenamed);
        assert!(item.display.contains('\u{2192}')); // →
        assert!(item.display.starts_with("> "));
    }

    #[test]
    fn file_change_to_picker_item_conflict() {
        let cwd = PathBuf::from("/project");
        let change = FileChange::Conflict {
            path: PathBuf::from("/project/merge.rs"),
        };
        let item = file_change_to_picker_item(&change, &cwd);
        assert_eq!(item.icon, PickerIcon::VcsConflict);
        assert!(item.display.starts_with("x "));
        assert_eq!(item.secondary.as_deref(), Some("conflict"));
    }

    #[test]
    fn path_display_strips_prefix() {
        let cwd = PathBuf::from("/project");
        let path = PathBuf::from("/project/src/main.rs");
        assert_eq!(super::path_display(&path, &cwd), "src/main.rs");
    }

    #[test]
    fn path_display_no_prefix() {
        let cwd = PathBuf::from("/other");
        let path = PathBuf::from("/project/src/main.rs");
        assert_eq!(super::path_display(&path, &cwd), "/project/src/main.rs");
    }
}
