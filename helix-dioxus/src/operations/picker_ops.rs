//! Picker operations for the editor.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use grep_matcher::Matcher;
use grep_regex::RegexMatcherBuilder;
use grep_searcher::sinks::UTF8;
use grep_searcher::{BinaryDetection, SearcherBuilder};
use ignore::WalkBuilder;

use crate::operations::{BufferOps, ThemeOps};
use crate::state::{
    EditorCommand, EditorContext, GlobalSearchResult, PickerIcon, PickerItem, PickerMode,
};

impl EditorContext {
    /// Navigate to a specific line and column in the current document.
    pub(crate) fn goto_line_column(&mut self, line: usize, column: usize) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document_mut(doc_id) else {
            return;
        };

        let text = doc.text();
        let total_lines = text.len_lines();

        // Clamp line to valid range
        let line = line.min(total_lines.saturating_sub(1));
        let line_start = text.line_to_char(line);
        let line_len = text.line(line).len_chars();

        // Clamp column to valid range
        let column = column.min(line_len.saturating_sub(1));
        let pos = line_start + column;

        // Set cursor position
        let selection = helix_core::Selection::point(pos);
        doc.set_selection(view_id, selection);
    }
}

/// Extension trait for picker operations.
pub trait PickerOps {
    fn show_file_picker(&mut self);
    fn show_files_recursive_picker(&mut self);
    fn show_buffer_picker(&mut self);
    fn filtered_picker_items(&self) -> Vec<PickerItem>;
    fn picker_confirm(&mut self);
}

impl PickerOps for EditorContext {
    /// Show the file picker with files from current directory.
    fn show_file_picker(&mut self) {
        self.command_mode = false;
        self.command_input.clear();

        // Get the current working directory
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Collect files and directories
        let mut items = Vec::new();

        // Add parent directory entry if not at root
        if cwd.parent().is_some() {
            items.push(PickerItem {
                id: "..".to_string(),
                display: "..".to_string(),
                icon: PickerIcon::Folder,
                match_indices: vec![],
                secondary: Some("Parent directory".to_string()),
                depth: 0,
            });
        }

        if let Ok(entries) = std::fs::read_dir(&cwd) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Skip hidden files (starting with .)
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                let display_name = if is_dir {
                    format!("{name}/")
                } else {
                    name.clone()
                };

                items.push(PickerItem {
                    id: path.to_string_lossy().to_string(),
                    display: display_name,
                    icon: if is_dir {
                        PickerIcon::Folder
                    } else {
                        PickerIcon::File
                    },
                    match_indices: vec![],
                    secondary: None,
                    depth: 0,
                });
            }
        }

        // Sort: directories first, then files, alphabetically
        items.sort_by(|a, b| {
            let a_is_dir = matches!(a.icon, PickerIcon::Folder);
            let b_is_dir = matches!(b.icon, PickerIcon::Folder);
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.display.to_lowercase().cmp(&b.display.to_lowercase()),
            }
        });

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::DirectoryBrowser;
        self.last_picker_mode = Some(PickerMode::DirectoryBrowser);
        self.picker_current_path = Some(cwd);
    }

    /// Show recursive file picker using the ignore crate.
    fn show_files_recursive_picker(&mut self) {
        use ignore::WalkBuilder;

        self.command_mode = false;
        self.command_input.clear();

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let mut items = Vec::new();

        let walker = WalkBuilder::new(&cwd)
            .hidden(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .build();

        for entry in walker.flatten() {
            let path = entry.path();

            // Skip directories, we only want files
            if path.is_dir() {
                continue;
            }

            // Get relative path
            let relative = path
                .strip_prefix(&cwd)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            items.push(PickerItem {
                id: path.to_string_lossy().to_string(),
                display: name,
                icon: PickerIcon::File,
                match_indices: vec![],
                secondary: Some(relative),
                depth: 0,
            });
        }

        // Sort alphabetically by display name
        items.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::FilesRecursive;
        self.last_picker_mode = Some(PickerMode::FilesRecursive);
        self.picker_current_path = Some(cwd);
    }

    /// Show buffer picker with open documents.
    fn show_buffer_picker(&mut self) {
        self.command_mode = false;
        self.command_input.clear();

        let current_doc_id = self.editor.tree.get(self.editor.tree.focus).doc;

        let items: Vec<PickerItem> = self
            .editor
            .documents
            .iter()
            .map(|(&id, doc)| {
                let name = doc.display_name().into_owned();
                let is_modified = doc.is_modified();
                let is_current = id == current_doc_id;

                PickerItem {
                    id: format!("{id}"),
                    display: name,
                    icon: if is_modified {
                        PickerIcon::BufferModified
                    } else {
                        PickerIcon::Buffer
                    },
                    match_indices: vec![],
                    secondary: if is_current {
                        Some("current".to_string())
                    } else {
                        None
                    },
                    depth: 0,
                }
            })
            .collect();

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::Buffers;
        self.last_picker_mode = Some(PickerMode::Buffers);
        self.picker_current_path = None;
    }

    /// Get filtered picker items with match indices populated.
    fn filtered_picker_items(&self) -> Vec<PickerItem> {
        if self.picker_filter.is_empty() {
            return self.picker_items.clone();
        }

        // FileExplorer: filter against all files (flat search) when filter is non-empty
        let source = if self.picker_mode == PickerMode::FileExplorer {
            &self.explorer_all_files
        } else {
            &self.picker_items
        };

        let mut results: Vec<(u16, PickerItem)> = source
            .iter()
            .filter_map(|item| {
                // Match against display name (primary) or secondary path
                let display_match = fuzzy_match_with_indices(&item.display, &self.picker_filter);
                let secondary_match = item
                    .secondary
                    .as_ref()
                    .and_then(|s| fuzzy_match_with_indices(s, &self.picker_filter));

                // Use the better match - only highlight display indices, not secondary
                match (display_match, secondary_match) {
                    (Some((score1, indices)), Some((score2, _))) if score1 >= score2 => {
                        // Display match is better or equal - use display indices
                        let mut new_item = item.clone();
                        new_item.match_indices = indices;
                        Some((score1, new_item))
                    }
                    (Some((score, indices)), None) => {
                        // Only display match - use its indices
                        let mut new_item = item.clone();
                        new_item.match_indices = indices;
                        Some((score, new_item))
                    }
                    (None, Some((score, _))) => {
                        // Only secondary match - no indices for display
                        Some((score, item.clone()))
                    }
                    (Some((score1, indices)), Some((score2, _))) if score2 > score1 => {
                        // Secondary match is better - use its score but display's indices
                        // (secondary indices don't apply to display text)
                        let mut new_item = item.clone();
                        new_item.match_indices = indices;
                        Some((score2, new_item))
                    }
                    _ => None,
                }
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.0.cmp(&a.0));

        results.into_iter().map(|(_, item)| item).collect()
    }

    /// Confirm the current picker selection.
    fn picker_confirm(&mut self) {
        let filtered = self.filtered_picker_items();
        if let Some(selected) = filtered.get(self.picker_selected).cloned() {
            match self.picker_mode {
                PickerMode::DirectoryBrowser => {
                    // Handle parent directory navigation
                    if selected.id == ".." {
                        if let Some(parent) = self
                            .picker_current_path
                            .as_ref()
                            .and_then(|p| p.parent().map(Path::to_path_buf))
                        {
                            if std::env::set_current_dir(&parent).is_ok() {
                                self.show_file_picker();
                                return;
                            }
                        }
                        return;
                    }

                    if matches!(selected.icon, PickerIcon::Folder) {
                        // It's a directory, change to it and refresh picker
                        let path = PathBuf::from(&selected.id);
                        if std::env::set_current_dir(&path).is_ok() {
                            self.show_file_picker();
                            return;
                        }
                        return;
                    }

                    // Open the file
                    let path = PathBuf::from(&selected.id);
                    self.open_file(&path);
                }
                PickerMode::FilesRecursive => {
                    let path = PathBuf::from(&selected.id);
                    self.open_file(&path);
                }
                PickerMode::Buffers => {
                    // Parse document ID and switch to it
                    // The id format is "DocumentId(N)" from Debug
                    if let Some(doc_id) = self.parse_document_id(&selected.id) {
                        self.switch_to_buffer(doc_id);
                    }
                }
                PickerMode::DocumentSymbols | PickerMode::WorkspaceSymbols => {
                    // Extract symbol data before mutable borrow
                    if let Ok(idx) = selected.id.parse::<usize>() {
                        let symbol_data = self
                            .symbols
                            .get(idx)
                            .map(|sym| (sym.path.clone(), sym.line, sym.column));

                        if let Some((path, line, column)) = symbol_data {
                            // For workspace symbols, open the file first
                            if self.picker_mode == PickerMode::WorkspaceSymbols {
                                if let Some(ref path) = path {
                                    self.open_file(path);
                                }
                            }

                            // Navigate to symbol position
                            let line = line.saturating_sub(1);
                            let column = column.saturating_sub(1);
                            self.goto_line_column(line, column);
                        }
                    }
                }
                PickerMode::DocumentDiagnostics | PickerMode::WorkspaceDiagnostics => {
                    // Extract diagnostic data before mutable borrow
                    if let Ok(idx) = selected.id.parse::<usize>() {
                        let diag_data = self.picker_diagnostics.get(idx).map(|entry| {
                            (
                                entry.doc_id,
                                entry.path.clone(),
                                entry.diagnostic.line,
                                entry.diagnostic.start_col,
                            )
                        });

                        if let Some((doc_id, path, line, column)) = diag_data {
                            // For workspace diagnostics, switch to the file first
                            if self.picker_mode == PickerMode::WorkspaceDiagnostics {
                                if let Some(ref path) = path {
                                    self.open_file(path);
                                } else if let Some(doc_id) = doc_id {
                                    self.switch_to_buffer(doc_id);
                                }
                            }

                            // Navigate to diagnostic position
                            let line = line.saturating_sub(1);
                            self.goto_line_column(line, column);
                        }
                    }
                }
                PickerMode::GlobalSearch => {
                    // Extract result data before mutable borrow
                    if let Ok(idx) = selected.id.parse::<usize>() {
                        let result_data = self
                            .global_search_results
                            .get(idx)
                            .map(|r| (r.path.clone(), r.line_num));

                        if let Some((path, line_num)) = result_data {
                            // Open the file
                            self.open_file(&path);
                            // Navigate to the line (line_num is 1-indexed, goto_line_column expects 0-indexed)
                            let line = line_num.saturating_sub(1);
                            self.goto_line_column(line, 0);
                        }
                    }
                }
                PickerMode::Registers => {
                    // Extract register char from the id (single char)
                    if let Some(ch) = selected.id.chars().next() {
                        self.editor.selected_register = Some(ch);
                        self.show_notification(
                            format!("Register '{ch}' selected"),
                            crate::state::NotificationSeverity::Info,
                        );
                    }
                }
                PickerMode::Commands => {
                    // Execute the selected command via deferred dispatch
                    if let Ok(idx) = selected.id.parse::<usize>() {
                        if let Some(cmd) = self.command_panel_commands.get(idx).cloned() {
                            let _ = self.command_tx.send(cmd);
                        }
                    }
                }
                PickerMode::JumpList => {
                    if let Ok(idx) = selected.id.parse::<usize>() {
                        if let Some((doc_id, selection)) = self.jumplist_entries.get(idx).cloned() {
                            let current_doc_id = {
                                let view = self.editor.tree.get(self.editor.tree.focus);
                                view.doc
                            };
                            if doc_id != current_doc_id {
                                self.editor
                                    .switch(doc_id, helix_view::editor::Action::Replace);
                            }
                            let (view, doc) = helix_view::current!(self.editor);
                            doc.set_selection(view.id, selection);
                            helix_view::align_view(doc, view, helix_view::Align::Center);
                        }
                    }
                }
                PickerMode::FileExplorer => {
                    // Directory: toggle expand/collapse
                    if matches!(selected.icon, PickerIcon::Folder | PickerIcon::FolderOpen) {
                        let path = PathBuf::from(&selected.id);
                        if selected.icon == PickerIcon::FolderOpen {
                            self.explorer_expanded.remove(&path);
                        } else {
                            self.explorer_expanded.insert(path);
                        }
                        self.rebuild_explorer_items();
                        return;
                    }
                    // File: open it
                    let path = PathBuf::from(&selected.id);
                    self.open_file(&path);
                }
                PickerMode::Themes => {
                    // Theme is already applied by live preview — just clear the rollback
                    self.theme_before_preview = None;
                }
                PickerMode::ChangedFiles => {
                    // Open the selected changed file
                    let path = PathBuf::from(&selected.id);
                    self.open_file(&path);
                }
                PickerMode::Emojis => {
                    // Insert the emoji text via deferred dispatch (same pattern as Commands)
                    let _ = self
                        .command_tx
                        .send(EditorCommand::InsertText(selected.id.clone()));
                }
                PickerMode::References | PickerMode::Definitions => {
                    // Extract location data before mutable borrow
                    if let Ok(idx) = selected.id.parse::<usize>() {
                        let location_data = self
                            .locations
                            .get(idx)
                            .map(|loc| (loc.path.clone(), loc.line, loc.column));

                        if let Some((path, line, column)) = location_data {
                            // Open the file
                            self.open_file(&path);
                            // Navigate to position (1-indexed to 0-indexed)
                            let line = line.saturating_sub(1);
                            let column = column.saturating_sub(1);
                            self.goto_line_column(line, column);
                        }
                    }
                }
            }
        } else if self.picker_mode == PickerMode::GlobalSearch && !self.picker_filter.is_empty() {
            // No items selected but filter is present - execute search
            self.execute_global_search();
            return; // Don't close the picker
        }

        self.picker_visible = false;
        self.picker_items.clear();
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_mode = PickerMode::default();
        self.picker_current_path = None;
        self.symbols.clear();
        self.picker_diagnostics.clear();
        self.global_search_results.clear();
        self.locations.clear();
        self.command_panel_commands.clear();
        self.jumplist_entries.clear();
        self.cancel_global_search();
        // Explorer cleanup
        self.explorer_expanded.clear();
        self.explorer_root = None;
        self.explorer_all_files.clear();
    }
}

impl EditorContext {
    /// Show file picker in the current buffer's directory.
    pub(crate) fn show_file_picker_in_buffer_dir(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let buffer_dir = self
            .editor
            .document(doc_id)
            .and_then(|doc| doc.path())
            .and_then(|p| p.parent().map(Path::to_path_buf));

        if let Some(dir) = buffer_dir {
            if std::env::set_current_dir(&dir).is_ok() {
                self.show_file_picker();
                return;
            }
        }

        // Fallback: open regular file picker
        self.show_file_picker();
    }

    /// Show the file explorer rooted at the given directory.
    pub(crate) fn show_file_explorer_at(&mut self, root: PathBuf) {
        self.command_mode = false;
        self.command_input.clear();

        self.explorer_expanded.clear();
        self.explorer_root = Some(root.clone());

        // Collect all files recursively for flat filtering
        self.explorer_all_files = collect_all_files(&root);

        self.rebuild_explorer_items();
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::FileExplorer;
        self.last_picker_mode = Some(PickerMode::FileExplorer);
        self.picker_current_path = Some(root);
    }

    /// Show file explorer at the current working directory (Space e).
    pub(crate) fn show_file_explorer(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self.show_file_explorer_at(cwd);
    }

    /// Show file explorer in the current buffer's directory (Space E).
    pub(crate) fn show_file_explorer_in_buffer_dir(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let buffer_dir = self
            .editor
            .document(doc_id)
            .and_then(|doc| doc.path())
            .and_then(|p| p.parent().map(Path::to_path_buf));

        let root = buffer_dir
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        self.show_file_explorer_at(root);
    }

    /// Rebuild the tree-view picker items from the explorer root and expanded set.
    pub(crate) fn rebuild_explorer_items(&mut self) {
        let Some(root) = self.explorer_root.clone() else {
            return;
        };
        let mut items = Vec::new();
        build_tree_items(&root, &self.explorer_expanded, 0, &mut items);
        self.picker_items = items;
    }

    /// Expand the selected directory in the file explorer.
    pub(crate) fn explorer_expand_selected(&mut self) {
        if self.picker_mode != PickerMode::FileExplorer {
            return;
        }
        let filtered = self.filtered_picker_items();
        let Some(selected) = filtered.get(self.picker_selected) else {
            return;
        };
        if selected.icon != PickerIcon::Folder {
            return;
        }
        let path = PathBuf::from(&selected.id);
        self.explorer_expanded.insert(path);
        let sel_id = selected.id.clone();
        self.rebuild_explorer_items();
        // Restore selection position
        if let Some(pos) = self.picker_items.iter().position(|i| i.id == sel_id) {
            self.picker_selected = pos;
        }
    }

    /// Collapse the selected directory or navigate to its parent in the file explorer.
    pub(crate) fn explorer_collapse_or_parent(&mut self) {
        if self.picker_mode != PickerMode::FileExplorer {
            return;
        }
        let filtered = self.filtered_picker_items();
        let Some(selected) = filtered.get(self.picker_selected) else {
            return;
        };
        if selected.icon == PickerIcon::FolderOpen {
            // Collapse this directory
            let path = PathBuf::from(&selected.id);
            self.explorer_expanded.remove(&path);
            let sel_id = selected.id.clone();
            self.rebuild_explorer_items();
            if let Some(pos) = self.picker_items.iter().position(|i| i.id == sel_id) {
                self.picker_selected = pos;
            }
        } else {
            // Navigate to parent directory in the item list
            let sel_path = PathBuf::from(&selected.id);
            if let Some(parent) = sel_path.parent() {
                let parent_str = parent.to_string_lossy().to_string();
                if let Some(pos) = self.picker_items.iter().position(|i| i.id == parent_str) {
                    self.picker_selected = pos;
                }
            }
        }
    }

    /// Show the register picker with all populated registers.
    pub(crate) fn show_register_picker(&mut self) {
        self.command_mode = false;
        self.command_input.clear();

        let mut items: Vec<PickerItem> = self
            .editor
            .registers
            .iter_preview()
            .map(|(name, preview)| {
                // For clipboard registers, try to read actual content
                let content = if matches!(name, '+' | '*') {
                    self.editor
                        .registers
                        .first(name, &self.editor)
                        .map(|cow| {
                            let s = cow.as_ref();
                            // Take first line, truncated
                            s.lines().next().unwrap_or(s).to_string()
                        })
                        .unwrap_or_else(|| preview.to_string())
                } else {
                    preview.to_string()
                };

                PickerItem {
                    id: name.to_string(),
                    display: format!("\"{name}\""),
                    icon: PickerIcon::Register,
                    match_indices: vec![],
                    secondary: Some(content),
                    depth: 0,
                }
            })
            .collect();

        // Sort: populated registers first, then by name
        items.sort_by(|a, b| {
            let a_populated = a.secondary.as_ref().is_some_and(|s| !s.starts_with('<'));
            let b_populated = b.secondary.as_ref().is_some_and(|s| !s.starts_with('<'));
            match (a_populated, b_populated) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => {
                    let a_name = a.id.chars().next().unwrap_or('\0');
                    let b_name = b.id.chars().next().unwrap_or('\0');
                    a_name.cmp(&b_name)
                }
            }
        });

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::Registers;
        self.last_picker_mode = Some(PickerMode::Registers);
        self.picker_current_path = None;
    }

    /// Show the command panel with all available editor commands.
    pub(crate) fn show_command_panel(&mut self) {
        self.command_mode = false;
        self.command_input.clear();

        let entries = command_panel_entries();
        let items: Vec<PickerItem> = entries
            .iter()
            .enumerate()
            .map(|(idx, (_, name, hint))| PickerItem {
                id: format!("{idx}"),
                display: (*name).to_string(),
                icon: PickerIcon::Command,
                match_indices: vec![],
                secondary: hint.map(ToString::to_string),
                depth: 0,
            })
            .collect();

        self.command_panel_commands = entries.iter().map(|(cmd, _, _)| cmd.clone()).collect();
        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::Commands;
        self.last_picker_mode = Some(PickerMode::Commands);
        self.picker_current_path = None;
    }

    /// Show the theme picker with all available themes.
    pub(crate) fn show_theme_picker(&mut self) {
        self.command_mode = false;
        self.command_input.clear();

        let themes = self.list_themes();
        let current = self.current_theme_name().to_string();

        // Save current theme for live preview rollback on Escape
        self.theme_before_preview = Some(current.clone());

        let items: Vec<PickerItem> = themes
            .iter()
            .map(|name| PickerItem {
                id: name.clone(),
                display: name.clone(),
                icon: PickerIcon::Theme,
                match_indices: vec![],
                secondary: if *name == current {
                    Some("current".to_string())
                } else {
                    None
                },
                depth: 0,
            })
            .collect();

        // Pre-select the current theme
        let selected = items
            .iter()
            .position(|item| item.id == current)
            .unwrap_or(0);

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = selected;
        self.picker_visible = true;
        self.picker_mode = PickerMode::Themes;
        self.last_picker_mode = Some(PickerMode::Themes);
        self.picker_current_path = None;
    }

    /// Show the emoji picker populated from the `emojis` crate.
    pub(crate) fn show_emoji_picker(&mut self) {
        self.command_mode = false;
        self.command_input.clear();

        let items: Vec<PickerItem> = emojis::iter()
            .map(|emoji| {
                let shortcode = emoji.shortcode().map(|sc| format!(":{sc}:"));

                PickerItem {
                    id: emoji.as_str().to_string(),
                    display: format!("{} {}", emoji.as_str(), emoji.name()),
                    icon: PickerIcon::Emoji,
                    match_indices: vec![],
                    secondary: shortcode,
                    depth: 0,
                }
            })
            .collect();

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::Emojis;
        self.last_picker_mode = Some(PickerMode::Emojis);
        self.picker_current_path = None;
    }

    /// Show the global search picker.
    pub(crate) fn show_global_search_picker(&mut self) {
        // Cancel any existing search
        self.cancel_global_search();

        // Reset picker state
        self.command_mode = false;
        self.command_input.clear();
        self.picker_items.clear();
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::GlobalSearch;
        self.last_picker_mode = Some(PickerMode::GlobalSearch);
        self.picker_current_path = None;
        self.global_search_results.clear();
    }

    /// Execute global search with the current filter pattern.
    pub(crate) fn execute_global_search(&mut self) {
        let pattern = self.picker_filter.trim().to_string();
        if pattern.is_empty() {
            return;
        }

        // Cancel any existing search
        self.cancel_global_search();

        // Clear previous results
        self.global_search_results.clear();
        self.picker_items.clear();
        self.picker_selected = 0;
        self.global_search_running = true;

        // Create cancellation channel
        let (cancel_tx, cancel_rx) = tokio::sync::watch::channel(false);
        self.global_search_cancel = Some(cancel_tx);

        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let command_tx = self.command_tx.clone();

        // Collect open documents' paths and their in-memory content
        let open_docs: std::collections::HashMap<PathBuf, String> = self
            .editor
            .documents
            .values()
            .filter_map(|doc| {
                doc.path().map(|p| {
                    let content = doc.text().to_string();
                    (p.to_path_buf(), content)
                })
            })
            .collect();
        let open_docs = Arc::new(open_docs);

        // Spawn search task on blocking thread pool (CPU-bound operation)
        tokio::task::spawn_blocking(move || {
            let result = execute_global_search_blocking(
                pattern,
                cwd,
                open_docs,
                command_tx.clone(),
                cancel_rx,
            );

            if let Err(e) = result {
                log::error!("Global search error: {e:?}");
            }

            // Signal completion
            let _ = command_tx.send(EditorCommand::GlobalSearchComplete);
        });
    }

    /// Cancel any running global search.
    pub(crate) fn cancel_global_search(&mut self) {
        if let Some(cancel_tx) = self.global_search_cancel.take() {
            let _ = cancel_tx.send(true);
        }
        self.global_search_running = false;
    }

    /// Update picker items from global search results.
    pub(crate) fn update_global_search_picker_items(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        self.picker_items = self
            .global_search_results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                // Get relative path for display
                let relative_path = result
                    .path
                    .strip_prefix(&cwd)
                    .unwrap_or(&result.path)
                    .to_string_lossy();

                let display = format!("{relative_path}:{}", result.line_num);

                PickerItem {
                    id: idx.to_string(),
                    display,
                    icon: PickerIcon::SearchResult,
                    match_indices: vec![],
                    secondary: Some(result.line_content.clone()),
                    depth: 0,
                }
            })
            .collect();
    }

    /// Update picker items from locations (used by References mode).
    pub(crate) fn update_references_picker_items(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        self.picker_items = self
            .locations
            .iter()
            .enumerate()
            .map(|(idx, loc)| {
                let relative_path = loc
                    .path
                    .strip_prefix(&cwd)
                    .unwrap_or(&loc.path)
                    .to_string_lossy();

                let display = format!("{relative_path}:{}:{}", loc.line, loc.column);
                let secondary = loc.preview.clone();

                PickerItem {
                    id: idx.to_string(),
                    display,
                    icon: PickerIcon::Reference,
                    match_indices: vec![],
                    secondary,
                    depth: 0,
                }
            })
            .collect();
    }

    /// Show references in the generic picker.
    pub(crate) fn show_references_picker(&mut self) {
        self.update_references_picker_items();
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::References;
        self.last_picker_mode = Some(PickerMode::References);
        self.picker_current_path = None;
    }

    /// Update picker items from locations (used by Definitions mode).
    pub(crate) fn update_definitions_picker_items(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        self.picker_items = self
            .locations
            .iter()
            .enumerate()
            .map(|(idx, loc)| {
                let relative_path = loc
                    .path
                    .strip_prefix(&cwd)
                    .unwrap_or(&loc.path)
                    .to_string_lossy();

                let display = format!("{relative_path}:{}:{}", loc.line, loc.column);
                let secondary = loc.preview.clone();

                PickerItem {
                    id: idx.to_string(),
                    display,
                    icon: PickerIcon::Definition,
                    match_indices: vec![],
                    secondary,
                    depth: 0,
                }
            })
            .collect();
    }

    /// Show definitions in the generic picker.
    pub(crate) fn show_definitions_picker(&mut self) {
        self.update_definitions_picker_items();
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::Definitions;
        self.last_picker_mode = Some(PickerMode::Definitions);
        self.picker_current_path = None;
    }
}

/// Execute global search on a blocking thread.
fn execute_global_search_blocking(
    pattern: String,
    cwd: PathBuf,
    open_docs: Arc<std::collections::HashMap<PathBuf, String>>,
    command_tx: std::sync::mpsc::Sender<EditorCommand>,
    cancel_rx: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    // Determine if pattern is case-sensitive (smart case: uppercase = case-sensitive)
    let has_uppercase = pattern.chars().any(char::is_uppercase);

    // Build regex matcher
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(!has_uppercase)
        .build(&pattern)?;

    let mut results: Vec<GlobalSearchResult> = Vec::new();
    let batch_size = 50;
    let max_results = 1000;
    let mut total_results = 0;

    // Walk files respecting .gitignore
    let walker = WalkBuilder::new(&cwd)
        .hidden(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        // Check for cancellation
        if *cancel_rx.borrow() {
            return Ok(());
        }

        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Check if this file is open in the editor (search in-memory content)
        let canonical_path = helix_stdx::path::canonicalize(path);
        if let Some(content) = open_docs.get(&canonical_path) {
            // Search in-memory content
            for (line_idx, line) in content.lines().enumerate() {
                if matcher.is_match(line.as_bytes())? {
                    let line_num = line_idx + 1;
                    let line_content = line.trim().to_string();

                    results.push(GlobalSearchResult {
                        path: canonical_path.clone(),
                        line_num,
                        line_content,
                    });

                    total_results += 1;
                    if total_results >= max_results {
                        break;
                    }
                }
            }
        } else {
            // Search file on disk
            let mut searcher = SearcherBuilder::new()
                .binary_detection(BinaryDetection::quit(b'\x00'))
                .build();

            let canonical_path_clone = canonical_path.clone();
            let search_result = searcher.search_path(
                &matcher,
                path,
                UTF8(|line_num, line| {
                    let line_content = line.trim().to_string();

                    results.push(GlobalSearchResult {
                        path: canonical_path_clone.clone(),
                        line_num: line_num as usize,
                        line_content,
                    });

                    total_results += 1;
                    Ok(total_results < max_results)
                }),
            );

            if let Err(e) = search_result {
                // Skip files that can't be read (binary, permission denied, etc.)
                log::debug!("Skipping file {path:?}: {e:?}");
            }
        }

        // Send batch if we have enough results
        if results.len() >= batch_size {
            let batch = std::mem::take(&mut results);
            let _ = command_tx.send(EditorCommand::GlobalSearchResults(batch));
        }

        if total_results >= max_results {
            break;
        }
    }

    // Send remaining results
    if !results.is_empty() {
        let _ = command_tx.send(EditorCommand::GlobalSearchResults(results));
    }

    Ok(())
}

/// Fuzzy match with indices: returns (score, match_indices) or None if no match.
/// Score is based on consecutive matches and start-of-word bonuses.
/// Case-insensitive matching.
/// Returns the static list of command panel entries: (command, display name, keybinding hint).
fn command_panel_entries() -> Vec<(EditorCommand, &'static str, Option<&'static str>)> {
    vec![
        // File operations
        (EditorCommand::ShowFilePicker, "Open File", Some("Space f")),
        (
            EditorCommand::ShowFilesRecursivePicker,
            "Find Files",
            Some("Ctrl+f"),
        ),
        (EditorCommand::ShowBufferPicker, "Switch Buffer", Some(":b")),
        // Buffer management
        (EditorCommand::NextBuffer, "Next Buffer", Some("Ctrl+l")),
        (
            EditorCommand::PreviousBuffer,
            "Previous Buffer",
            Some("Ctrl+h"),
        ),
        (
            EditorCommand::ReloadDocument,
            "Reload Document",
            Some(":reload"),
        ),
        (EditorCommand::WriteAll, "Save All", Some(":wa")),
        // Navigation
        (EditorCommand::GotoFirstLine, "Go to First Line", Some("gg")),
        (EditorCommand::GotoLastLine, "Go to Last Line", Some("G")),
        (
            EditorCommand::GotoFirstNonWhitespace,
            "Go to First Non-Whitespace",
            Some("gs"),
        ),
        (EditorCommand::PageUp, "Page Up", Some("C-b")),
        (EditorCommand::PageDown, "Page Down", Some("C-f")),
        (EditorCommand::HalfPageUp, "Half Page Up", Some("C-u")),
        (EditorCommand::HalfPageDown, "Half Page Down", Some("C-d")),
        // Search
        (
            EditorCommand::ShowGlobalSearch,
            "Global Search",
            Some("Space /"),
        ),
        (
            EditorCommand::EnterSearchMode { backwards: false },
            "Search Forward",
            Some("/"),
        ),
        (
            EditorCommand::EnterSearchMode { backwards: true },
            "Search Backward",
            Some("?"),
        ),
        // Editing
        (
            EditorCommand::AlignSelections,
            "Align Selections",
            Some("&"),
        ),
        // LSP
        (EditorCommand::FormatDocument, "Format Document", None),
        (
            EditorCommand::FormatSelections,
            "Format Selections",
            Some("="),
        ),
        (
            EditorCommand::RenameSymbol,
            "Rename Symbol",
            Some("Space r"),
        ),
        (
            EditorCommand::GotoDeclaration,
            "Go to Declaration",
            Some("gD"),
        ),
        (
            EditorCommand::GotoDefinition,
            "Go to Definition",
            Some("gd"),
        ),
        (
            EditorCommand::GotoReferences,
            "Go to References",
            Some("gr"),
        ),
        (
            EditorCommand::GotoTypeDefinition,
            "Go to Type Definition",
            Some("gy"),
        ),
        (
            EditorCommand::GotoImplementation,
            "Go to Implementation",
            Some("gi"),
        ),
        (
            EditorCommand::ShowCodeActions,
            "Show Code Actions",
            Some("Space a"),
        ),
        (
            EditorCommand::ShowDocumentSymbols,
            "Document Symbols",
            Some("Space s"),
        ),
        (
            EditorCommand::ShowWorkspaceSymbols,
            "Workspace Symbols",
            Some("Space S"),
        ),
        (
            EditorCommand::ShowDocumentDiagnostics,
            "Document Diagnostics",
            Some("Space d"),
        ),
        (
            EditorCommand::ShowWorkspaceDiagnostics,
            "Workspace Diagnostics",
            Some("Space D"),
        ),
        (EditorCommand::NextDiagnostic, "Next Diagnostic", Some("]d")),
        (
            EditorCommand::PrevDiagnostic,
            "Previous Diagnostic",
            Some("[d"),
        ),
        (
            EditorCommand::ToggleInlayHints,
            "Toggle Inlay Hints",
            Some("Space i"),
        ),
        (EditorCommand::ToggleLspDialog, "LSP Server Status", None),
        (
            EditorCommand::TriggerHover,
            "Trigger Hover",
            Some("Space k"),
        ),
        (
            EditorCommand::TriggerCompletion,
            "Trigger Completion",
            Some("Ctrl+Space"),
        ),
        // Editing
        (
            EditorCommand::ToggleLineComment,
            "Toggle Line Comment",
            Some("Space c"),
        ),
        (
            EditorCommand::ToggleBlockComment,
            "Toggle Block Comment",
            Some("Space C"),
        ),
        (EditorCommand::Undo, "Undo", Some("u")),
        (EditorCommand::Redo, "Redo", Some("U")),
        (EditorCommand::IndentLine, "Indent", Some(">")),
        (EditorCommand::UnindentLine, "Unindent", Some("<")),
        (EditorCommand::SelectAll, "Select All", Some("%")),
        (
            EditorCommand::FlipSelections,
            "Flip Selections",
            Some("A-;"),
        ),
        (
            EditorCommand::ExpandSelection,
            "Expand Selection (tree-sitter)",
            Some("A-o"),
        ),
        (
            EditorCommand::ShrinkSelection,
            "Shrink Selection (tree-sitter)",
            Some("A-i"),
        ),
        (EditorCommand::JoinLines, "Join Lines", Some("J")),
        (EditorCommand::ToggleCase, "Toggle Case", Some("~")),
        // Jump list
        (EditorCommand::JumpBackward, "Jump Backward", Some("C-o")),
        (EditorCommand::JumpForward, "Jump Forward", Some("C-i")),
        (
            EditorCommand::SaveSelection,
            "Save Position to Jump List",
            Some("C-s"),
        ),
        (
            EditorCommand::ShowJumpListPicker,
            "Jump List",
            Some("Space j"),
        ),
        (
            EditorCommand::CliCommand("jumplist-clear".to_string()),
            "Clear Jump List",
            Some(":jumplist-clear"),
        ),
        // Application
        (
            EditorCommand::PrintWorkingDirectory,
            "Print Working Directory",
            Some(":pwd"),
        ),
        // Shell integration
        (
            EditorCommand::EnterShellMode(crate::state::ShellBehavior::Replace),
            "Shell Pipe (Replace)",
            Some("|"),
        ),
        (
            EditorCommand::EnterShellMode(crate::state::ShellBehavior::Insert),
            "Shell Insert Output",
            Some("!"),
        ),
        (
            EditorCommand::EnterShellMode(crate::state::ShellBehavior::Ignore),
            "Shell Pipe-To (Discard)",
            Some("A-|"),
        ),
        (
            EditorCommand::EnterShellMode(crate::state::ShellBehavior::Append),
            "Shell Append Output",
            Some("A-!"),
        ),
        // File explorer
        (
            EditorCommand::ShowFileExplorer,
            "File Explorer",
            Some("Space e"),
        ),
        (
            EditorCommand::ShowFileExplorerInBufferDir,
            "File Explorer (Buffer Dir)",
            Some("Space E"),
        ),
        // Word jump
        (EditorCommand::GotoWord, "Goto Word (Jump)", Some("gw")),
        // Theme
        (
            EditorCommand::ShowThemePicker,
            "Switch Theme",
            Some(":theme"),
        ),
        // VCS
        (
            EditorCommand::ShowChangedFilesPicker,
            "Changed Files",
            Some("Space g"),
        ),
        (EditorCommand::NextChange, "Next Change", Some("]g")),
        (EditorCommand::PrevChange, "Previous Change", Some("[g")),
        (EditorCommand::GotoFirstChange, "First Change", Some("[G")),
        (EditorCommand::GotoLastChange, "Last Change", Some("]G")),
        // Emoji picker
        (
            EditorCommand::ShowEmojiPicker,
            "Insert Emoji",
            Some("C-Cmd-Spc"),
        ),
        // Text manipulation
        (
            EditorCommand::CliCommand("sort".to_string()),
            "Sort Selections",
            Some(":sort"),
        ),
        (
            EditorCommand::CliCommand("reflow".to_string()),
            "Reflow Text",
            Some(":reflow"),
        ),
        // Configuration
        (
            EditorCommand::CliCommand("config-open".to_string()),
            "Open Config File",
            Some(":config-open"),
        ),
        (
            EditorCommand::CliCommand("config-reload".to_string()),
            "Reload Config",
            Some(":config-reload"),
        ),
        (
            EditorCommand::CliCommand("log-open".to_string()),
            "Open Log File",
            Some(":log-open"),
        ),
        // Document info
        (
            EditorCommand::CliCommand("encoding".to_string()),
            "Show Encoding",
            Some(":encoding"),
        ),
        (
            EditorCommand::CliCommand("tree-sitter-scopes".to_string()),
            "Tree-sitter Scopes",
            Some(":tree-sitter-scopes"),
        ),
    ]
}

pub(crate) fn fuzzy_match_with_indices(text: &str, pattern: &str) -> Option<(u16, Vec<usize>)> {
    if pattern.is_empty() {
        return Some((0, vec![]));
    }

    let text_lower: Vec<char> = text.to_lowercase().chars().collect();
    let pattern_lower: Vec<char> = pattern.to_lowercase().chars().collect();

    let mut match_indices = Vec::with_capacity(pattern_lower.len());
    let mut pattern_idx = 0;
    let mut score: u16 = 0;
    let mut prev_match_idx: Option<usize> = None;

    for (text_idx, &text_char) in text_lower.iter().enumerate() {
        if pattern_idx < pattern_lower.len() && text_char == pattern_lower[pattern_idx] {
            match_indices.push(text_idx);

            // Scoring bonuses
            if text_idx == 0 {
                // Start of string bonus
                score = score.saturating_add(10);
            } else if let Some(prev_idx) = prev_match_idx {
                if text_idx == prev_idx + 1 {
                    // Consecutive match bonus
                    score = score.saturating_add(5);
                }
            }

            // Word boundary bonus (after separator)
            if text_idx > 0 {
                let prev_char = text.chars().nth(text_idx - 1);
                if matches!(prev_char, Some('/' | '\\' | '_' | '-' | ' ' | '.')) {
                    score = score.saturating_add(8);
                }
            }

            prev_match_idx = Some(text_idx);
            pattern_idx += 1;
        }
    }

    if pattern_idx == pattern_lower.len() {
        // All pattern characters matched
        // Bonus for shorter text (prefer exact or near-exact matches)
        let len_bonus = (100u16).saturating_sub(text.len() as u16);
        score = score.saturating_add(len_bonus / 10);
        Some((score, match_indices))
    } else {
        None
    }
}

/// Build tree-view items from a root directory, recursing into expanded dirs.
fn build_tree_items(
    dir: &std::path::Path,
    expanded: &std::collections::HashSet<PathBuf>,
    depth: u16,
    items: &mut Vec<PickerItem>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if path.is_dir() {
            dirs.push((path, name));
        } else {
            files.push((path, name));
        }
    }

    // Sort case-insensitive
    dirs.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    files.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

    // Directories first
    for (path, name) in dirs {
        let is_expanded = expanded.contains(&path);
        items.push(PickerItem {
            id: path.to_string_lossy().to_string(),
            display: format!("{name}/"),
            icon: if is_expanded {
                PickerIcon::FolderOpen
            } else {
                PickerIcon::Folder
            },
            match_indices: vec![],
            secondary: None,
            depth,
        });
        if is_expanded {
            build_tree_items(&path, expanded, depth + 1, items);
        }
    }

    // Then files
    for (path, name) in files {
        items.push(PickerItem {
            id: path.to_string_lossy().to_string(),
            display: name,
            icon: PickerIcon::File,
            match_indices: vec![],
            secondary: None,
            depth,
        });
    }
}

/// Collect all files recursively from a root directory (for flat filtering).
fn collect_all_files(root: &std::path::Path) -> Vec<PickerItem> {
    let mut items = Vec::new();
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        items.push(PickerItem {
            id: path.to_string_lossy().to_string(),
            display: name,
            icon: PickerIcon::File,
            match_indices: vec![],
            secondary: Some(relative),
            depth: 0,
        });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn build_tree_items_sorts_dirs_first() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("zebra.txt"), "").expect("write");
        fs::write(dir.path().join("apple.txt"), "").expect("write");
        fs::create_dir(dir.path().join("beta_dir")).expect("mkdir");
        fs::create_dir(dir.path().join("alpha_dir")).expect("mkdir");

        let expanded = std::collections::HashSet::new();
        let mut items = Vec::new();
        build_tree_items(dir.path(), &expanded, 0, &mut items);

        // Dirs first (alpha_dir, beta_dir), then files (apple.txt, zebra.txt)
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].display, "alpha_dir/");
        assert_eq!(items[0].icon, PickerIcon::Folder);
        assert_eq!(items[1].display, "beta_dir/");
        assert_eq!(items[1].icon, PickerIcon::Folder);
        assert_eq!(items[2].display, "apple.txt");
        assert_eq!(items[2].icon, PickerIcon::File);
        assert_eq!(items[3].display, "zebra.txt");
    }

    #[test]
    fn build_tree_items_shows_hidden_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join(".hidden"), "").expect("write");
        fs::write(dir.path().join("visible.txt"), "").expect("write");

        let expanded = std::collections::HashSet::new();
        let mut items = Vec::new();
        build_tree_items(dir.path(), &expanded, 0, &mut items);

        let names: Vec<&str> = items.iter().map(|i| i.display.as_str()).collect();
        assert!(names.contains(&".hidden"), "hidden files should appear");
        assert!(names.contains(&"visible.txt"));
    }

    #[test]
    fn build_tree_items_depth_increments_on_expand() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sub = dir.path().join("subdir");
        fs::create_dir(&sub).expect("mkdir");
        fs::write(sub.join("inner.txt"), "").expect("write");

        let mut expanded = std::collections::HashSet::new();
        expanded.insert(sub.clone());

        let mut items = Vec::new();
        build_tree_items(dir.path(), &expanded, 0, &mut items);

        // subdir/ (depth 0, FolderOpen), inner.txt (depth 1)
        assert!(items.len() >= 2);
        let subdir_item = items
            .iter()
            .find(|i| i.display == "subdir/")
            .expect("subdir");
        assert_eq!(subdir_item.depth, 0);
        assert_eq!(subdir_item.icon, PickerIcon::FolderOpen);

        let inner_item = items
            .iter()
            .find(|i| i.display == "inner.txt")
            .expect("inner");
        assert_eq!(inner_item.depth, 1);
    }

    #[test]
    fn collect_all_files_skips_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::create_dir(dir.path().join("subdir")).expect("mkdir");
        fs::write(dir.path().join("file.txt"), "").expect("write");
        fs::write(dir.path().join("subdir/nested.txt"), "").expect("write");

        let items = collect_all_files(dir.path());
        assert!(items.iter().all(|i| i.icon == PickerIcon::File));
        assert!(items.len() >= 2);
    }
}
