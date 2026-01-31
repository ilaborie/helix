//! Editor state management for Dioxus integration.
//!
//! Since `helix_view::Editor` contains non-Sync types (Cell, etc.), we cannot
//! share it directly via Dioxus context. Instead, we use a message-passing
//! approach where the Editor lives on the main thread and we communicate
//! via channels.
//!
//! This module provides:
//! - `EditorHandle`: A thread-safe handle to send commands to the editor
//! - `EditorSnapshot`: A read-only snapshot of editor state for rendering

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;

use anyhow::Result;
use helix_view::document::Mode;

/// Commands that can be sent to the editor.
#[derive(Debug, Clone)]
pub enum EditorCommand {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    GotoFirstLine,
    GotoLastLine,

    // Mode changes
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeLineEnd,
    ExitInsertMode,

    // Editing
    InsertChar(char),
    InsertNewline,
    DeleteCharBackward,
    DeleteCharForward,
    OpenLineBelow,
    OpenLineAbove,

    // Selection
    ExtendLeft,
    ExtendRight,
    ExtendUp,
    ExtendDown,

    // Command mode
    EnterCommandMode,
    ExitCommandMode,
    CommandInput(char),
    CommandBackspace,
    CommandExecute,

    // File picker
    ShowFilePicker,
    PickerUp,
    PickerDown,
    PickerConfirm,
    PickerCancel,
    PickerInput(char),
    PickerBackspace,

    // File operations
    OpenFile(PathBuf),
}

/// A snapshot of the editor state for rendering.
/// This is Clone + Send + Sync so it can be used with Dioxus.
#[derive(Debug, Clone, Default)]
pub struct EditorSnapshot {
    pub mode: String,
    pub file_name: String,
    pub is_modified: bool,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub total_lines: usize,
    pub visible_start: usize,
    pub lines: Vec<LineSnapshot>,

    // UI state
    pub command_mode: bool,
    pub command_input: String,
    pub picker_visible: bool,
    pub picker_items: Vec<String>,
    pub picker_filtered: Vec<String>,
    pub picker_filter: String,
    pub picker_selected: usize,
    pub picker_total: usize,

    // Application state
    pub should_quit: bool,
}

/// Snapshot of a single line for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct LineSnapshot {
    pub line_number: usize,
    pub content: String,
    pub is_cursor_line: bool,
    pub cursor_col: Option<usize>,
    pub tokens: Vec<TokenSpan>,
}

/// A span of text with a specific color for syntax highlighting.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSpan {
    /// Start character offset in the line (0-indexed).
    pub start: usize,
    /// End character offset in the line (exclusive).
    pub end: usize,
    /// CSS color string, e.g., "#e06c75".
    pub color: String,
}

/// The editor wrapper that lives on the main thread.
pub struct EditorContext {
    pub editor: helix_view::Editor,
    command_rx: mpsc::Receiver<EditorCommand>,

    // UI state
    command_mode: bool,
    command_input: String,
    picker_visible: bool,
    picker_items: Vec<String>,
    picker_filter: String,
    picker_selected: usize,

    // Application state
    should_quit: bool,
}

impl EditorContext {
    /// Create a new editor context with the given file.
    pub fn new(file: Option<PathBuf>, command_rx: mpsc::Receiver<EditorCommand>) -> Result<Self> {
        // Load syntax configuration
        let syn_loader = helix_core::config::default_lang_config();
        let syn_loader = helix_core::syntax::Loader::new(syn_loader)?;
        let syn_loader = Arc::new(arc_swap::ArcSwap::from_pointee(syn_loader));

        // Load theme
        let theme_loader = helix_view::theme::Loader::new(&[]);
        let theme_loader = Arc::new(theme_loader);

        // Create editor configuration
        let config = helix_view::editor::Config::default();
        let config: Arc<dyn arc_swap::access::DynAccess<helix_view::editor::Config>> =
            Arc::new(arc_swap::ArcSwap::from_pointee(config));

        // Create dummy handlers
        let handlers = create_dummy_handlers();

        // Create the editor
        let mut editor = helix_view::Editor::new(
            helix_view::graphics::Rect::new(0, 0, 120, 40),
            theme_loader,
            syn_loader,
            config,
            handlers,
        );

        // Initialize syntax highlighting scopes from the theme
        // This is required for the highlighter to produce meaningful highlights
        let scopes = editor.theme.scopes();
        editor.syn_loader.load().set_scopes(scopes.to_vec());

        // Open file if provided
        // Note: Use VerticalSplit for initial file - Replace assumes an existing view
        if let Some(path) = file {
            let path = helix_stdx::path::canonicalize(&path);
            editor.open(&path, helix_view::editor::Action::VerticalSplit)?;
        } else {
            // Create a scratch buffer
            editor.new_file(helix_view::editor::Action::VerticalSplit);
        }

        Ok(Self {
            editor,
            command_rx,
            command_mode: false,
            command_input: String::new(),
            picker_visible: false,
            picker_items: Vec::new(),
            picker_filter: String::new(),
            picker_selected: 0,
            should_quit: false,
        })
    }

    /// Process pending commands.
    pub fn process_commands(&mut self) {
        while let Ok(cmd) = self.command_rx.try_recv() {
            self.handle_command(cmd);
        }

        // Ensure cursor stays visible in viewport after any cursor movements
        let view_id = self.editor.tree.focus;
        self.editor.ensure_cursor_in_view(view_id);
    }

    /// Handle a single command.
    fn handle_command(&mut self, cmd: EditorCommand) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        match cmd {
            EditorCommand::MoveLeft => self.move_cursor(doc_id, view_id, Direction::Left),
            EditorCommand::MoveRight => self.move_cursor(doc_id, view_id, Direction::Right),
            EditorCommand::MoveUp => self.move_cursor(doc_id, view_id, Direction::Up),
            EditorCommand::MoveDown => self.move_cursor(doc_id, view_id, Direction::Down),
            EditorCommand::MoveWordForward => self.move_word_forward(doc_id, view_id),
            EditorCommand::MoveWordBackward => self.move_word_backward(doc_id, view_id),
            EditorCommand::MoveLineStart => self.move_line_start(doc_id, view_id),
            EditorCommand::MoveLineEnd => self.move_line_end(doc_id, view_id),
            EditorCommand::GotoFirstLine => self.goto_first_line(doc_id, view_id),
            EditorCommand::GotoLastLine => self.goto_last_line(doc_id, view_id),
            EditorCommand::EnterInsertMode => self.set_mode(Mode::Insert),
            EditorCommand::EnterInsertModeAfter => {
                self.move_cursor(doc_id, view_id, Direction::Right);
                self.set_mode(Mode::Insert);
            }
            EditorCommand::EnterInsertModeLineEnd => {
                self.move_line_end(doc_id, view_id);
                self.set_mode(Mode::Insert);
            }
            EditorCommand::ExitInsertMode => self.set_mode(Mode::Normal),
            EditorCommand::InsertChar(c) => self.insert_char(doc_id, view_id, c),
            EditorCommand::InsertNewline => self.insert_char(doc_id, view_id, '\n'),
            EditorCommand::DeleteCharBackward => self.delete_char_backward(doc_id, view_id),
            EditorCommand::DeleteCharForward => self.delete_char_forward(doc_id, view_id),
            EditorCommand::OpenLineBelow => {
                self.open_line_below(doc_id, view_id);
                self.set_mode(Mode::Insert);
            }
            EditorCommand::OpenLineAbove => {
                self.open_line_above(doc_id, view_id);
                self.set_mode(Mode::Insert);
            }
            EditorCommand::ExtendLeft => self.extend_selection(doc_id, view_id, Direction::Left),
            EditorCommand::ExtendRight => self.extend_selection(doc_id, view_id, Direction::Right),
            EditorCommand::ExtendUp => self.extend_selection(doc_id, view_id, Direction::Up),
            EditorCommand::ExtendDown => self.extend_selection(doc_id, view_id, Direction::Down),

            // Command mode
            EditorCommand::EnterCommandMode => {
                self.command_mode = true;
                self.command_input.clear();
            }
            EditorCommand::ExitCommandMode => {
                self.command_mode = false;
                self.command_input.clear();
            }
            EditorCommand::CommandInput(c) => {
                self.command_input.push(c);
            }
            EditorCommand::CommandBackspace => {
                self.command_input.pop();
            }
            EditorCommand::CommandExecute => {
                self.execute_command();
            }

            // File picker
            EditorCommand::ShowFilePicker => {
                self.show_file_picker();
            }
            EditorCommand::PickerUp => {
                if self.picker_selected > 0 {
                    self.picker_selected -= 1;
                }
            }
            EditorCommand::PickerDown => {
                let filtered_len = self.filtered_picker_items().len();
                if self.picker_selected + 1 < filtered_len {
                    self.picker_selected += 1;
                }
            }
            EditorCommand::PickerConfirm => {
                self.picker_confirm();
            }
            EditorCommand::PickerCancel => {
                self.picker_visible = false;
                self.picker_items.clear();
                self.picker_filter.clear();
                self.picker_selected = 0;
            }
            EditorCommand::PickerInput(c) => {
                self.picker_filter.push(c);
                self.picker_selected = 0;
            }
            EditorCommand::PickerBackspace => {
                self.picker_filter.pop();
                self.picker_selected = 0;
            }

            // File operations
            EditorCommand::OpenFile(path) => {
                self.open_file(&path);
            }
        }
    }

    /// Execute the current command input.
    fn execute_command(&mut self) {
        let input = self.command_input.trim();
        if input.is_empty() {
            self.command_mode = false;
            return;
        }

        // Parse the command
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let args = parts.get(1).map(|s| s.trim());

        match cmd {
            "o" | "open" => {
                if let Some(filename) = args {
                    // Open specific file
                    let path = PathBuf::from(filename);
                    self.open_file(&path);
                } else {
                    // Show file picker
                    self.show_file_picker();
                }
            }
            "q" | "quit" => {
                self.try_quit(false);
            }
            "q!" | "quit!" => {
                self.try_quit(true);
            }
            "w" | "write" => {
                let path = args.map(PathBuf::from);
                self.save_document(path, false);
            }
            "w!" | "write!" => {
                let path = args.map(PathBuf::from);
                self.save_document(path, true);
            }
            "wq" | "x" => {
                self.save_document(None, false);
                self.try_quit(false);
            }
            "wq!" | "x!" => {
                self.save_document(None, true);
                self.try_quit(true);
            }
            _ => {
                log::warn!("Unknown command: {}", cmd);
            }
        }

        self.command_mode = false;
        self.command_input.clear();
    }

    /// Show the file picker with files from current directory.
    fn show_file_picker(&mut self) {
        self.command_mode = false;
        self.command_input.clear();

        // Get the current working directory
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Collect files and directories
        let mut items = Vec::new();

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

                // Add directory indicator
                let display_name = if path.is_dir() {
                    format!("{}/", name)
                } else {
                    name
                };

                items.push(display_name);
            }
        }

        // Sort: directories first, then files, alphabetically
        items.sort_by(|a, b| {
            let a_is_dir = a.ends_with('/');
            let b_is_dir = b.ends_with('/');
            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.to_lowercase().cmp(&b.to_lowercase()),
            }
        });

        self.picker_items = items;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_visible = true;
    }

    /// Get filtered picker items based on current filter.
    fn filtered_picker_items(&self) -> Vec<String> {
        if self.picker_filter.is_empty() {
            return self.picker_items.clone();
        }

        self.picker_items
            .iter()
            .filter(|item| fuzzy_match(item, &self.picker_filter))
            .cloned()
            .collect()
    }

    /// Confirm the current picker selection.
    fn picker_confirm(&mut self) {
        let filtered = self.filtered_picker_items();
        if let Some(selected) = filtered.get(self.picker_selected).cloned() {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            if selected.ends_with('/') {
                // It's a directory, change to it and refresh picker
                let dir_name = selected.trim_end_matches('/');
                let new_path = cwd.join(dir_name);
                if std::env::set_current_dir(&new_path).is_ok() {
                    self.show_file_picker();
                    return;
                }
                return;
            }

            // Build full path for file
            let path = cwd.join(&selected);
            self.open_file(&path);
        }

        self.picker_visible = false;
        self.picker_items.clear();
        self.picker_filter.clear();
        self.picker_selected = 0;
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

    /// Create a snapshot of the current editor state.
    pub fn snapshot(&self, viewport_lines: usize) -> EditorSnapshot {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc = match self.editor.document(view.doc) {
            Some(doc) => doc,
            None => return EditorSnapshot::default(),
        };

        let mode = match self.editor.mode() {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Select => "SELECT",
        };

        let file_name = doc
            .path()
            .map(|p| {
                p.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| "[scratch]".to_string());

        let text = doc.text();
        let selection = doc.selection(view_id);
        let cursor = selection.primary().cursor(text.slice(..));

        let cursor_line = text.char_to_line(cursor);
        let line_start = text.line_to_char(cursor_line);
        let cursor_col = cursor - line_start;

        let total_lines = text.len_lines();

        // Calculate visible range
        let view_offset = doc.view_offset(view_id);
        let visible_start = text.char_to_line(view_offset.anchor.min(text.len_chars()));
        let visible_end = (visible_start + viewport_lines).min(total_lines);

        // Compute syntax highlighting tokens for visible lines
        let line_tokens = self.compute_syntax_tokens(doc, visible_start, visible_end);

        let lines: Vec<LineSnapshot> = (visible_start..visible_end)
            .enumerate()
            .map(|(idx, line_idx)| {
                let line_content = text.line(line_idx).to_string();
                let is_cursor_line = line_idx == cursor_line;
                let cursor_col_opt = if is_cursor_line {
                    Some(cursor_col)
                } else {
                    None
                };

                LineSnapshot {
                    line_number: line_idx + 1,
                    content: line_content,
                    is_cursor_line,
                    cursor_col: cursor_col_opt,
                    tokens: line_tokens.get(idx).cloned().unwrap_or_default(),
                }
            })
            .collect();

        EditorSnapshot {
            mode: mode.to_string(),
            file_name,
            is_modified: doc.is_modified(),
            cursor_line: cursor_line + 1,
            cursor_col: cursor_col + 1,
            total_lines,
            visible_start,
            lines,
            command_mode: self.command_mode,
            command_input: self.command_input.clone(),
            picker_visible: self.picker_visible,
            picker_items: self.picker_items.clone(),
            picker_filtered: self.filtered_picker_items(),
            picker_filter: self.picker_filter.clone(),
            picker_selected: self.picker_selected,
            picker_total: self.picker_items.len(),
            should_quit: self.should_quit,
        }
    }

    /// Compute syntax highlighting tokens for a range of visible lines.
    /// Returns a Vec of Vec<TokenSpan>, one for each line in the range.
    ///
    /// This follows helix-term's pattern from document.rs - maintaining a computed
    /// style that gets updated as we process highlight events.
    fn compute_syntax_tokens(
        &self,
        doc: &helix_view::Document,
        visible_start: usize,
        visible_end: usize,
    ) -> Vec<Vec<TokenSpan>> {
        use helix_core::syntax::HighlightEvent;

        let text = doc.text();
        let text_slice = text.slice(..);

        // Get syntax information
        let syntax = match doc.syntax() {
            Some(s) => s,
            None => {
                return vec![Vec::new(); visible_end - visible_start];
            }
        };

        let loader = self.editor.syn_loader.load();
        let theme = &self.editor.theme;

        // Calculate byte range for visible lines
        let start_char = text.line_to_char(visible_start);
        let end_char = if visible_end >= text.len_lines() {
            text.len_chars()
        } else {
            text.line_to_char(visible_end)
        };
        let start_byte = text.char_to_byte(start_char) as u32;
        let end_byte = text.char_to_byte(end_char) as u32;

        // Create highlighter for the visible range
        let mut highlighter = syntax.highlighter(text_slice, &loader, start_byte..end_byte);

        // Prepare storage for each line
        let mut line_tokens: Vec<Vec<TokenSpan>> = vec![Vec::new(); visible_end - visible_start];

        // Default text style (no foreground color)
        let text_style = helix_view::theme::Style::default();

        // Current computed style - following helix-term's SyntaxHighlighter pattern
        let mut current_style = text_style;

        // Current position in bytes
        let mut pos = start_byte;

        // Process highlight events following helix-term's pattern
        loop {
            // Get the position of the next event
            let next_event_pos = highlighter.next_event_offset();

            // If no more events (u32::MAX), process remaining text to end_byte
            let span_end = if next_event_pos == u32::MAX {
                end_byte
            } else {
                next_event_pos
            };

            // Emit a span from pos to span_end with current style
            if span_end > pos {
                // Only emit if we have a foreground color
                if let Some(fg) = current_style.fg {
                    if let Some(css_color) = color_to_css(&fg) {
                        // Convert byte positions to character positions
                        let span_start_char = text.byte_to_char(pos as usize);
                        let span_end_char = text.byte_to_char(span_end as usize);

                        // Find which lines this span affects
                        let span_start_line = text.char_to_line(span_start_char);
                        let span_end_line =
                            text.char_to_line(span_end_char.saturating_sub(1).max(span_start_char));

                        for line_idx in span_start_line..=span_end_line {
                            if line_idx < visible_start || line_idx >= visible_end {
                                continue;
                            }
                            let line_start_char = text.line_to_char(line_idx);
                            let line_end_char = if line_idx + 1 < text.len_lines() {
                                text.line_to_char(line_idx + 1)
                            } else {
                                text.len_chars()
                            };

                            // Calculate token start/end within this line
                            let token_start =
                                span_start_char.max(line_start_char) - line_start_char;
                            let token_end = span_end_char.min(line_end_char) - line_start_char;

                            if token_start < token_end {
                                let line_slot = line_idx - visible_start;
                                line_tokens[line_slot].push(TokenSpan {
                                    start: token_start,
                                    end: token_end,
                                    color: css_color.clone(),
                                });
                            }
                        }
                    }
                }
            }

            // If no more events, we're done
            if next_event_pos == u32::MAX || next_event_pos >= end_byte {
                break;
            }

            // Move position to the event location
            pos = next_event_pos;

            // Process the highlight event - following helix-term's exact pattern
            let (event, highlights) = highlighter.advance();

            // Determine the base style based on event type
            let base = match event {
                HighlightEvent::Refresh => text_style,
                HighlightEvent::Push => current_style,
            };

            // Fold all highlights onto the base style
            current_style =
                highlights.fold(base, |acc, highlight| acc.patch(theme.highlight(highlight)));
        }

        line_tokens
    }

    // Helper methods for editing operations

    fn set_mode(&mut self, mode: Mode) {
        self.editor.mode = mode;
    }

    fn move_cursor(
        &mut self,
        doc_id: helix_view::DocumentId,
        view_id: helix_view::ViewId,
        direction: Direction,
    ) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let new_cursor = match direction {
                Direction::Left => cursor.saturating_sub(1),
                Direction::Right => {
                    let max = text.len_chars().saturating_sub(1);
                    (cursor + 1).min(max)
                }
                Direction::Up => {
                    let line = text.char_to_line(cursor);
                    if line == 0 {
                        return range;
                    }
                    let col = cursor - text.line_to_char(line);
                    let new_line = line - 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
                Direction::Down => {
                    let line = text.char_to_line(cursor);
                    if line >= text.len_lines().saturating_sub(1) {
                        return range;
                    }
                    let col = cursor - text.line_to_char(line);
                    let new_line = line + 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
            };
            helix_core::Range::point(new_cursor)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn extend_selection(
        &mut self,
        doc_id: helix_view::DocumentId,
        view_id: helix_view::ViewId,
        direction: Direction,
    ) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let head = range.head;
            let anchor = range.anchor;

            let new_head = match direction {
                Direction::Left => head.saturating_sub(1),
                Direction::Right => {
                    let max = text.len_chars().saturating_sub(1);
                    (head + 1).min(max)
                }
                Direction::Up => {
                    let line = text.char_to_line(head);
                    if line == 0 {
                        return range;
                    }
                    let col = head - text.line_to_char(line);
                    let new_line = line - 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
                Direction::Down => {
                    let line = text.char_to_line(head);
                    if line >= text.len_lines().saturating_sub(1) {
                        return range;
                    }
                    let col = head - text.line_to_char(line);
                    let new_line = line + 1;
                    let new_line_len = text.line(new_line).len_chars().saturating_sub(1);
                    text.line_to_char(new_line) + col.min(new_line_len)
                }
            };

            helix_core::Range::new(anchor, new_head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn move_word_forward(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_next_word_start(text, range, 1);
            helix_core::Range::point(new_range.head.min(text.len_chars().saturating_sub(1)))
        });

        doc.set_selection(view_id, new_selection);
    }

    fn move_word_backward(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let new_range = helix_core::movement::move_prev_word_start(text, range, 1);
            helix_core::Range::point(new_range.head)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn move_line_start(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let line = text.char_to_line(cursor);
            let line_start = text.line_to_char(line);
            helix_core::Range::point(line_start)
        });

        doc.set_selection(view_id, new_selection);
    }

    fn move_line_end(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let new_selection = selection.transform(|range| {
            let cursor = range.cursor(text);
            let line = text.char_to_line(cursor);
            let line_end = text.line_to_char(line) + text.line(line).len_chars().saturating_sub(1);
            helix_core::Range::point(line_end.max(text.line_to_char(line)))
        });

        doc.set_selection(view_id, new_selection);
    }

    fn goto_first_line(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        doc.set_selection(view_id, helix_core::Selection::point(0));
    }

    fn goto_last_line(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);

        let last_line = text.len_lines().saturating_sub(1);
        let line_start = text.line_to_char(last_line);

        doc.set_selection(view_id, helix_core::Selection::point(line_start));
    }

    fn insert_char(
        &mut self,
        doc_id: helix_view::DocumentId,
        view_id: helix_view::ViewId,
        c: char,
    ) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone().cursors(text);

        let transaction =
            helix_core::Transaction::insert(doc.text(), &selection, c.to_string().into());
        doc.apply(&transaction, view_id);
    }

    fn delete_char_backward(
        &mut self,
        doc_id: helix_view::DocumentId,
        view_id: helix_view::ViewId,
    ) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        if cursor == 0 {
            return;
        }

        let ranges = std::iter::once((cursor - 1, cursor));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    fn delete_char_forward(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        if cursor >= text.len_chars() {
            return;
        }

        let ranges = std::iter::once((cursor, cursor + 1));
        let transaction = helix_core::Transaction::delete(doc.text(), ranges);
        doc.apply(&transaction, view_id);
    }

    fn open_line_below(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_end = text.line_to_char(line) + text.line(line).len_chars();

        // Move to end of line
        let new_selection = helix_core::Selection::point(line_end.saturating_sub(1));
        doc.set_selection(view_id, new_selection.clone());

        // Insert newline
        let transaction = helix_core::Transaction::insert(doc.text(), &new_selection, "\n".into());
        doc.apply(&transaction, view_id);
    }

    fn open_line_above(&mut self, doc_id: helix_view::DocumentId, view_id: helix_view::ViewId) {
        let doc = self.editor.document_mut(doc_id).unwrap();
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id).clone();

        let cursor = selection.primary().cursor(text);
        let line = text.char_to_line(cursor);
        let line_start = text.line_to_char(line);

        // Insert newline at start of current line
        let insert_selection = helix_core::Selection::point(line_start);
        let transaction =
            helix_core::Transaction::insert(doc.text(), &insert_selection, "\n".into());
        doc.apply(&transaction, view_id);

        // Move cursor to the new empty line
        doc.set_selection(view_id, helix_core::Selection::point(line_start));
    }
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Convert a helix Color to a CSS color string.
fn color_to_css(color: &helix_view::graphics::Color) -> Option<String> {
    use helix_view::graphics::Color;
    match color {
        Color::Rgb(r, g, b) => Some(format!("#{:02x}{:02x}{:02x}", r, g, b)),
        Color::Reset => None,
        // Map standard colors to One Dark palette
        Color::Black => Some("#282c34".into()),
        Color::Red => Some("#e06c75".into()),
        Color::Green => Some("#98c379".into()),
        Color::Yellow => Some("#e5c07b".into()),
        Color::Blue => Some("#61afef".into()),
        Color::Magenta => Some("#c678dd".into()),
        Color::Cyan => Some("#56b6c2".into()),
        Color::Gray => Some("#5c6370".into()),
        Color::White => Some("#abb2bf".into()),
        Color::LightRed => Some("#e06c75".into()),
        Color::LightGreen => Some("#98c379".into()),
        Color::LightYellow => Some("#e5c07b".into()),
        Color::LightBlue => Some("#61afef".into()),
        Color::LightMagenta => Some("#c678dd".into()),
        Color::LightCyan => Some("#56b6c2".into()),
        Color::LightGray => Some("#abb2bf".into()),
        // For indexed colors, use a default
        Color::Indexed(_) => Some("#abb2bf".into()),
    }
}

/// Fuzzy match: check if all characters in `pattern` appear in order in `text`.
/// Case-insensitive matching.
fn fuzzy_match(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern_lower = pattern.to_lowercase();

    let mut pattern_chars = pattern_lower.chars().peekable();

    for c in text_lower.chars() {
        if pattern_chars.peek() == Some(&c) {
            pattern_chars.next();
        }
        if pattern_chars.peek().is_none() {
            return true;
        }
    }

    pattern_chars.peek().is_none()
}

/// Create dummy handlers for initialization.
/// In a full implementation, these would be properly connected to async event handlers.
fn create_dummy_handlers() -> helix_view::handlers::Handlers {
    use helix_view::handlers::completion::CompletionHandler;
    use helix_view::handlers::*;
    use tokio::sync::mpsc::channel;

    let (completion_tx, _) = channel(1);
    let (signature_tx, _) = channel(1);
    let (auto_save_tx, _) = channel(1);
    let (doc_colors_tx, _) = channel(1);
    let (pull_diag_tx, _) = channel(1);
    let (pull_all_diag_tx, _) = channel(1);

    Handlers {
        completions: CompletionHandler::new(completion_tx),
        signature_hints: signature_tx,
        auto_save: auto_save_tx,
        document_colors: doc_colors_tx,
        word_index: word_index::Handler::spawn(),
        pull_diagnostics: pull_diag_tx,
        pull_all_documents_diagnostics: pull_all_diag_tx,
    }
}
