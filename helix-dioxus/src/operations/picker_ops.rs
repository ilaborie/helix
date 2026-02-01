//! Picker operations for the editor.

use std::path::PathBuf;

use crate::operations::BufferOps;
use crate::state::{EditorContext, PickerIcon, PickerItem, PickerMode};

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
                    format!("{}/", name)
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
            });
        }

        // Sort alphabetically by display name
        items.sort_by(|a, b| a.display.to_lowercase().cmp(&b.display.to_lowercase()));

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::FilesRecursive;
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
                    id: format!("{:?}", id),
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
                }
            })
            .collect();

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
        self.picker_mode = PickerMode::Buffers;
        self.picker_current_path = None;
    }

    /// Get filtered picker items with match indices populated.
    fn filtered_picker_items(&self) -> Vec<PickerItem> {
        if self.picker_filter.is_empty() {
            return self.picker_items.clone();
        }

        let mut results: Vec<(u16, PickerItem)> = self
            .picker_items
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
                            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
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
            }
        }

        self.picker_visible = false;
        self.picker_items.clear();
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_mode = PickerMode::default();
        self.picker_current_path = None;
        self.symbols.clear();
        self.picker_diagnostics.clear();
    }
}

/// Fuzzy match with indices: returns (score, match_indices) or None if no match.
/// Score is based on consecutive matches and start-of-word bonuses.
/// Case-insensitive matching.
fn fuzzy_match_with_indices(text: &str, pattern: &str) -> Option<(u16, Vec<usize>)> {
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
