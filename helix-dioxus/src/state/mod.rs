//! Editor state management for Dioxus integration.
//!
//! Since `helix_view::Editor` contains non-Sync types (Cell, etc.), we cannot
//! share it directly via Dioxus context. Instead, we use a message-passing
//! approach where the Editor lives on the main thread and we communicate
//! via channels.
//!
//! This module provides:
//! - `EditorContext`: The main editor wrapper with command handling
//! - `EditorSnapshot`: A read-only snapshot of editor state for rendering
//! - `EditorCommand`: Commands that can be sent to the editor

mod types;

pub use types::{
    BufferInfo, Direction, EditorCommand, EditorSnapshot, LineSnapshot, PickerIcon, PickerItem,
    PickerMode, TokenSpan,
};

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;

use anyhow::Result;
use helix_view::document::Mode;

use crate::operations::{
    BufferOps, CliOps, ClipboardOps, EditingOps, MovementOps, PickerOps, SearchOps, SelectionOps,
};

/// The editor wrapper that lives on the main thread.
pub struct EditorContext {
    pub editor: helix_view::Editor,
    command_rx: mpsc::Receiver<EditorCommand>,

    // UI state - pub(crate) for operations access
    pub(crate) command_mode: bool,
    pub(crate) command_input: String,
    pub(crate) search_mode: bool,
    pub(crate) search_backwards: bool,
    pub(crate) search_input: String,
    pub(crate) last_search: String,

    // Picker state - pub(crate) for operations access
    pub(crate) picker_visible: bool,
    pub(crate) picker_items: Vec<PickerItem>,
    pub(crate) picker_filter: String,
    pub(crate) picker_selected: usize,
    pub(crate) picker_mode: PickerMode,
    pub(crate) picker_current_path: Option<PathBuf>,

    // Buffer bar state - pub(crate) for operations access
    pub(crate) buffer_bar_scroll: usize,

    // Clipboard (simple string for now) - pub(crate) for operations access
    pub(crate) clipboard: String,

    // Application state - pub(crate) for operations access
    pub(crate) should_quit: bool,
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
            search_mode: false,
            search_backwards: false,
            search_input: String::new(),
            last_search: String::new(),
            picker_visible: false,
            picker_items: Vec::new(),
            picker_filter: String::new(),
            picker_selected: 0,
            picker_mode: PickerMode::default(),
            picker_current_path: None,
            buffer_bar_scroll: 0,
            clipboard: String::new(),
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

    /// Handle a single command using operation traits.
    fn handle_command(&mut self, cmd: EditorCommand) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        match cmd {
            // Movement operations
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

            // Mode changes
            EditorCommand::EnterInsertMode => self.editor.mode = Mode::Insert,
            EditorCommand::EnterInsertModeAfter => {
                self.move_cursor(doc_id, view_id, Direction::Right);
                self.editor.mode = Mode::Insert;
            }
            EditorCommand::EnterInsertModeLineEnd => {
                self.move_line_end(doc_id, view_id);
                self.editor.mode = Mode::Insert;
            }
            EditorCommand::ExitInsertMode => self.editor.mode = Mode::Normal,
            EditorCommand::EnterSelectMode => self.editor.mode = Mode::Select,
            EditorCommand::ExitSelectMode => self.editor.mode = Mode::Normal,

            // Editing operations
            EditorCommand::InsertChar(c) => self.insert_char(doc_id, view_id, c),
            EditorCommand::InsertNewline => self.insert_char(doc_id, view_id, '\n'),
            EditorCommand::DeleteCharBackward => self.delete_char_backward(doc_id, view_id),
            EditorCommand::DeleteCharForward => self.delete_char_forward(doc_id, view_id),
            EditorCommand::OpenLineBelow => {
                self.open_line_below(doc_id, view_id);
                self.editor.mode = Mode::Insert;
            }
            EditorCommand::OpenLineAbove => {
                self.open_line_above(doc_id, view_id);
                self.editor.mode = Mode::Insert;
            }

            // Selection operations
            EditorCommand::ExtendLeft => self.extend_selection(doc_id, view_id, Direction::Left),
            EditorCommand::ExtendRight => self.extend_selection(doc_id, view_id, Direction::Right),
            EditorCommand::ExtendUp => self.extend_selection(doc_id, view_id, Direction::Up),
            EditorCommand::ExtendDown => self.extend_selection(doc_id, view_id, Direction::Down),
            EditorCommand::ExtendWordForward => self.extend_word_forward(doc_id, view_id),
            EditorCommand::ExtendWordBackward => self.extend_word_backward(doc_id, view_id),
            EditorCommand::ExtendLineStart => self.extend_line_start(doc_id, view_id),
            EditorCommand::ExtendLineEnd => self.extend_line_end(doc_id, view_id),
            EditorCommand::SelectLine => self.select_line(doc_id, view_id),
            EditorCommand::ExtendLine => self.extend_line(doc_id, view_id),

            // Clipboard operations
            EditorCommand::Yank => self.yank(doc_id, view_id),
            EditorCommand::Paste => self.paste(doc_id, view_id, false),
            EditorCommand::PasteBefore => self.paste(doc_id, view_id, true),
            EditorCommand::DeleteSelection => self.delete_selection(doc_id, view_id),

            // History operations
            EditorCommand::Undo => self.undo(doc_id, view_id),
            EditorCommand::Redo => self.redo(doc_id, view_id),

            // Search operations
            EditorCommand::EnterSearchMode { backwards } => {
                self.search_mode = true;
                self.search_backwards = backwards;
                self.search_input.clear();
            }
            EditorCommand::ExitSearchMode => {
                self.search_mode = false;
                self.search_input.clear();
            }
            EditorCommand::SearchInput(ch) => {
                self.search_input.push(ch);
            }
            EditorCommand::SearchBackspace => {
                self.search_input.pop();
            }
            EditorCommand::SearchExecute => {
                self.execute_search(doc_id, view_id);
            }
            EditorCommand::SearchNext => {
                self.search_next(doc_id, view_id, false);
            }
            EditorCommand::SearchPrevious => {
                self.search_next(doc_id, view_id, true);
            }

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

            // Picker operations
            EditorCommand::ShowFilePicker => {
                self.show_file_picker();
            }
            EditorCommand::ShowFilesRecursivePicker => {
                self.show_files_recursive_picker();
            }
            EditorCommand::ShowBufferPicker => {
                self.show_buffer_picker();
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
                self.picker_mode = PickerMode::default();
                self.picker_current_path = None;
            }
            EditorCommand::PickerInput(c) => {
                self.picker_filter.push(c);
                self.picker_selected = 0;
            }
            EditorCommand::PickerBackspace => {
                self.picker_filter.pop();
                self.picker_selected = 0;
            }

            // Buffer navigation
            EditorCommand::BufferBarScrollLeft => {
                self.buffer_bar_scroll = self.buffer_bar_scroll.saturating_sub(1);
            }
            EditorCommand::BufferBarScrollRight => {
                let buffer_count = self.editor.documents.len();
                let max_scroll = buffer_count.saturating_sub(8);
                if self.buffer_bar_scroll < max_scroll {
                    self.buffer_bar_scroll += 1;
                }
            }
            EditorCommand::SwitchToBuffer(doc_id) => {
                self.switch_to_buffer(doc_id);
            }
            EditorCommand::CloseBuffer(doc_id) => {
                self.close_buffer(doc_id);
            }
            EditorCommand::NextBuffer => {
                self.cycle_buffer(1);
            }
            EditorCommand::PreviousBuffer => {
                self.cycle_buffer(-1);
            }

            // File operations
            EditorCommand::OpenFile(path) => {
                self.open_file(&path);
            }
        }
    }

    /// Create a snapshot of the current editor state.
    pub fn snapshot(&mut self, viewport_lines: usize) -> EditorSnapshot {
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

        // Get selection range for highlighting
        let primary_range = selection.primary();
        let sel_start = primary_range.from();
        let sel_end = primary_range.to();

        // Only show selection highlighting in Select mode
        let is_select_mode = self.editor.mode() == Mode::Select;
        let has_selection = is_select_mode && sel_end > sel_start;

        log::info!(
            "Selection: anchor={}, head={}, from={}, to={}, is_select_mode={}, has_selection={}",
            primary_range.anchor,
            primary_range.head,
            sel_start,
            sel_end,
            is_select_mode,
            has_selection
        );

        let is_modified = doc.is_modified();

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

                let selection_range = if has_selection {
                    let line_start_char = text.line_to_char(line_idx);
                    let line_len = text.line(line_idx).len_chars().saturating_sub(1);
                    let line_end_char = line_start_char + line_len;

                    if sel_end > line_start_char && sel_start <= line_end_char {
                        let range_start = sel_start.saturating_sub(line_start_char);
                        let range_end = (sel_end - line_start_char).min(line_len);
                        if range_start <= range_end {
                            Some((range_start, range_end))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                LineSnapshot {
                    line_number: line_idx + 1,
                    content: line_content,
                    is_cursor_line,
                    cursor_col: cursor_col_opt,
                    tokens: line_tokens.get(idx).cloned().unwrap_or_default(),
                    selection_range,
                }
            })
            .collect();

        let (open_buffers, buffer_scroll_offset) = self.buffer_bar_snapshot();

        EditorSnapshot {
            mode: mode.to_string(),
            file_name,
            is_modified,
            cursor_line: cursor_line + 1,
            cursor_col: cursor_col + 1,
            total_lines,
            visible_start,
            lines,
            command_mode: self.command_mode,
            command_input: self.command_input.clone(),
            search_mode: self.search_mode,
            search_backwards: self.search_backwards,
            search_input: self.search_input.clone(),
            picker_visible: self.picker_visible,
            picker_items: self.filtered_picker_items(),
            picker_filter: self.picker_filter.clone(),
            picker_selected: self.picker_selected,
            picker_total: self.picker_items.len(),
            picker_mode: self.picker_mode,
            picker_current_path: self
                .picker_current_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            open_buffers,
            buffer_scroll_offset,
            should_quit: self.should_quit,
        }
    }

    /// Compute syntax highlighting tokens for a range of visible lines.
    fn compute_syntax_tokens(
        &self,
        doc: &helix_view::Document,
        visible_start: usize,
        visible_end: usize,
    ) -> Vec<Vec<TokenSpan>> {
        use helix_core::syntax::HighlightEvent;

        let text = doc.text();
        let text_slice = text.slice(..);

        let syntax = match doc.syntax() {
            Some(s) => s,
            None => {
                return vec![Vec::new(); visible_end - visible_start];
            }
        };

        let loader = self.editor.syn_loader.load();
        let theme = &self.editor.theme;

        let start_char = text.line_to_char(visible_start);
        let end_char = if visible_end >= text.len_lines() {
            text.len_chars()
        } else {
            text.line_to_char(visible_end)
        };
        let start_byte = text.char_to_byte(start_char) as u32;
        let end_byte = text.char_to_byte(end_char) as u32;

        let mut highlighter = syntax.highlighter(text_slice, &loader, start_byte..end_byte);
        let mut line_tokens: Vec<Vec<TokenSpan>> = vec![Vec::new(); visible_end - visible_start];
        let text_style = helix_view::theme::Style::default();
        let mut current_style = text_style;
        let mut pos = start_byte;

        loop {
            let next_event_pos = highlighter.next_event_offset();
            let span_end = if next_event_pos == u32::MAX {
                end_byte
            } else {
                next_event_pos
            };

            if span_end > pos {
                if let Some(fg) = current_style.fg {
                    if let Some(css_color) = color_to_css(&fg) {
                        let span_start_char = text.byte_to_char(pos as usize);
                        let span_end_char = text.byte_to_char(span_end as usize);
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

            if next_event_pos == u32::MAX || next_event_pos >= end_byte {
                break;
            }

            pos = next_event_pos;
            let (event, highlights) = highlighter.advance();

            let base = match event {
                HighlightEvent::Refresh => text_style,
                HighlightEvent::Push => current_style,
            };

            current_style =
                highlights.fold(base, |acc, highlight| acc.patch(theme.highlight(highlight)));
        }

        line_tokens
    }
}

/// Convert a helix Color to a CSS color string.
fn color_to_css(color: &helix_view::graphics::Color) -> Option<String> {
    use helix_view::graphics::Color;
    match color {
        Color::Rgb(r, g, b) => Some(format!("#{:02x}{:02x}{:02x}", r, g, b)),
        Color::Reset => None,
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
        Color::Indexed(_) => Some("#abb2bf".into()),
    }
}

/// Create dummy handlers for initialization.
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
