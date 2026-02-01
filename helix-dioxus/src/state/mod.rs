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

mod lsp_events;
mod types;

pub use types::{
    BufferInfo, ConfirmationAction, ConfirmationDialogSnapshot, Direction, EditorCommand,
    EditorSnapshot, GlobalSearchResult, InputDialogKind, InputDialogSnapshot, LineSnapshot,
    NotificationSeverity, NotificationSnapshot, PickerIcon, PickerItem, PickerMode, TokenSpan,
};

use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;

use anyhow::Result;
use helix_core::syntax::config::LanguageServerFeature;
use helix_lsp::lsp;
use helix_view::document::Mode;

use crate::lsp::{
    convert_code_actions, convert_completion_response, convert_document_symbols,
    convert_goto_response, convert_hover, convert_inlay_hints, convert_references_response,
    convert_signature_help, convert_workspace_symbols, CompletionItemSnapshot,
    DiagnosticPickerEntry, DiagnosticSeverity, DiagnosticSnapshot, HoverSnapshot,
    InlayHintSnapshot, LocationSnapshot, LspResponse, LspServerSnapshot, LspServerStatus,
    SignatureHelpSnapshot, StoredCodeAction, SymbolKind, SymbolSnapshot,
};
use crate::operations::{
    BufferOps, CliOps, ClipboardOps, EditingOps, LspOps, MovementOps, PickerOps, SearchOps,
    SelectionOps,
};

use lsp_events::LspEventOps;

/// The editor wrapper that lives on the main thread.
pub struct EditorContext {
    pub editor: helix_view::Editor,
    command_rx: mpsc::Receiver<EditorCommand>,
    /// Sender for sending commands back (used for LSP async responses).
    pub(crate) command_tx: mpsc::Sender<EditorCommand>,

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

    // LSP state - pub(crate) for operations access
    /// Whether the completion popup is visible.
    pub(crate) completion_visible: bool,
    /// Completion items.
    pub(crate) completion_items: Vec<CompletionItemSnapshot>,
    /// Selected completion index.
    pub(crate) completion_selected: usize,
    /// Whether hover is visible.
    pub(crate) hover_visible: bool,
    /// Hover content.
    pub(crate) hover_content: Option<HoverSnapshot>,
    /// Whether signature help is visible.
    pub(crate) signature_help_visible: bool,
    /// Signature help content.
    pub(crate) signature_help: Option<SignatureHelpSnapshot>,
    /// Cached inlay hints.
    pub(crate) inlay_hints: Vec<InlayHintSnapshot>,
    /// Whether inlay hints are enabled.
    pub(crate) inlay_hints_enabled: bool,
    /// Whether code actions menu is visible.
    pub(crate) code_actions_visible: bool,
    /// Code actions (with full data for execution).
    pub(crate) code_actions: Vec<StoredCodeAction>,
    /// Selected code action index.
    pub(crate) code_action_selected: usize,
    /// Code action filter string for searching.
    pub(crate) code_action_filter: String,
    /// Whether code actions are available at the current cursor position.
    /// This is checked proactively to show a lightbulb indicator.
    pub(crate) has_code_actions: bool,
    /// Last position where we checked for code actions (to avoid repeated checks).
    code_actions_check_position: Option<(helix_view::DocumentId, usize, usize)>,
    /// Whether location picker is visible.
    pub(crate) location_picker_visible: bool,
    /// Locations for picker.
    pub(crate) locations: Vec<LocationSnapshot>,
    /// Selected location index.
    pub(crate) location_selected: usize,
    /// Location picker title.
    pub(crate) location_picker_title: String,

    // Symbol picker state - pub(crate) for operations access
    /// Symbols for symbol picker.
    pub(crate) symbols: Vec<SymbolSnapshot>,

    // Diagnostic picker state - pub(crate) for operations access
    /// Diagnostics for diagnostic picker.
    pub(crate) picker_diagnostics: Vec<DiagnosticPickerEntry>,

    // LSP dialog state - pub(crate) for operations access
    /// Whether the LSP dialog is visible.
    pub(crate) lsp_dialog_visible: bool,
    /// Selected server index in dialog.
    pub(crate) lsp_server_selected: usize,
    /// LSP progress tracking for indexing status.
    pub(crate) lsp_progress: helix_lsp::LspProgressMap,

    // Notification state - pub(crate) for operations access
    /// Active notifications.
    pub(crate) notifications: Vec<NotificationSnapshot>,
    /// Counter for generating unique notification IDs.
    pub(crate) notification_id_counter: u64,

    // Input dialog state - pub(crate) for operations access
    /// Whether the input dialog is visible.
    pub(crate) input_dialog_visible: bool,
    /// Input dialog value.
    pub(crate) input_dialog_value: String,
    /// Input dialog title.
    pub(crate) input_dialog_title: String,
    /// Input dialog prompt.
    pub(crate) input_dialog_prompt: String,
    /// Input dialog placeholder.
    pub(crate) input_dialog_placeholder: Option<String>,
    /// Kind of input dialog operation pending.
    pub(crate) input_dialog_kind: InputDialogKind,

    // Confirmation dialog state - pub(crate) for operations access
    /// Whether the confirmation dialog is visible.
    pub(crate) confirmation_dialog_visible: bool,
    /// Current confirmation dialog snapshot.
    pub(crate) confirmation_dialog: ConfirmationDialogSnapshot,

    // Global search state - pub(crate) for operations access
    /// Global search results.
    pub(crate) global_search_results: Vec<GlobalSearchResult>,
    /// Whether a global search is currently running.
    pub(crate) global_search_running: bool,
    /// Cancellation signal for running global search.
    pub(crate) global_search_cancel: Option<tokio::sync::watch::Sender<bool>>,

    // Application state - pub(crate) for operations access
    pub(crate) should_quit: bool,

    /// Snapshot version counter, incremented on each snapshot creation.
    snapshot_version: u64,
}

impl EditorContext {
    /// Create a new editor context with the given file.
    pub fn new(
        file: Option<PathBuf>,
        command_rx: mpsc::Receiver<EditorCommand>,
        command_tx: mpsc::Sender<EditorCommand>,
    ) -> Result<Self> {
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

        // Create handlers and register essential hooks
        let handlers = create_handlers();

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
            command_tx,
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
            // LSP state
            completion_visible: false,
            completion_items: Vec::new(),
            completion_selected: 0,
            hover_visible: false,
            hover_content: None,
            signature_help_visible: false,
            signature_help: None,
            inlay_hints: Vec::new(),
            inlay_hints_enabled: true,
            code_actions_visible: false,
            code_actions: Vec::new(),
            code_action_selected: 0,
            code_action_filter: String::new(),
            has_code_actions: false,
            code_actions_check_position: None,
            location_picker_visible: false,
            locations: Vec::new(),
            location_selected: 0,
            location_picker_title: String::new(),
            // Symbol picker state
            symbols: Vec::new(),
            // Diagnostic picker state
            picker_diagnostics: Vec::new(),
            // LSP dialog state
            lsp_dialog_visible: false,
            lsp_server_selected: 0,
            lsp_progress: helix_lsp::LspProgressMap::new(),
            // Notification state
            notifications: Vec::new(),
            notification_id_counter: 0,
            // Input dialog state
            input_dialog_visible: false,
            input_dialog_value: String::new(),
            input_dialog_title: String::new(),
            input_dialog_prompt: String::new(),
            input_dialog_placeholder: None,
            input_dialog_kind: InputDialogKind::default(),
            // Confirmation dialog state
            confirmation_dialog_visible: false,
            confirmation_dialog: ConfirmationDialogSnapshot::default(),
            // Global search state
            global_search_results: Vec::new(),
            global_search_running: false,
            global_search_cancel: None,
            should_quit: false,
            snapshot_version: 0,
        })
    }

    /// Process pending commands.
    pub fn process_commands(&mut self) {
        while let Ok(cmd) = self.command_rx.try_recv() {
            self.handle_command(cmd);
        }

        // Poll for LSP events (diagnostics, progress, etc.)
        self.poll_lsp_events();

        // Check for code actions at cursor (for lightbulb indicator)
        self.check_code_actions_available();

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
            EditorCommand::PageUp => self.page_up(doc_id, view_id),
            EditorCommand::PageDown => self.page_down(doc_id, view_id),
            EditorCommand::ScrollUp(lines) => self.scroll_up(doc_id, view_id, lines),
            EditorCommand::ScrollDown(lines) => self.scroll_down(doc_id, view_id, lines),

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
            EditorCommand::InsertTab => self.insert_tab(doc_id, view_id),
            EditorCommand::InsertNewline => self.insert_newline(doc_id, view_id),
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

            // Comments
            EditorCommand::ToggleLineComment => self.toggle_line_comment(doc_id, view_id),
            EditorCommand::ToggleBlockComment => self.toggle_block_comment(doc_id, view_id),

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
                // Cancel any running global search
                if self.picker_mode == PickerMode::GlobalSearch {
                    self.cancel_global_search();
                    self.global_search_results.clear();
                }
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
            EditorCommand::SaveDocumentToPath(path) => {
                self.save_document(Some(path), false);
            }

            // LSP - Completion
            EditorCommand::TriggerCompletion => {
                self.trigger_completion();
            }
            EditorCommand::CompletionUp => {
                if self.completion_selected > 0 {
                    self.completion_selected -= 1;
                }
            }
            EditorCommand::CompletionDown => {
                if self.completion_selected + 1 < self.completion_items.len() {
                    self.completion_selected += 1;
                }
            }
            EditorCommand::CompletionConfirm => {
                self.apply_completion();
            }
            EditorCommand::CompletionCancel => {
                self.completion_visible = false;
                self.completion_items.clear();
                self.completion_selected = 0;
            }

            // LSP - Hover
            EditorCommand::TriggerHover => {
                self.trigger_hover();
            }
            EditorCommand::CloseHover => {
                self.hover_visible = false;
                self.hover_content = None;
            }

            // LSP - Goto
            EditorCommand::GotoDefinition => {
                self.trigger_goto_definition();
            }
            EditorCommand::GotoReferences => {
                self.trigger_goto_references();
            }
            EditorCommand::GotoTypeDefinition => {
                self.trigger_goto_type_definition();
            }
            EditorCommand::GotoImplementation => {
                self.trigger_goto_implementation();
            }
            EditorCommand::LocationUp => {
                if self.location_selected > 0 {
                    self.location_selected -= 1;
                }
            }
            EditorCommand::LocationDown => {
                if self.location_selected + 1 < self.locations.len() {
                    self.location_selected += 1;
                }
            }
            EditorCommand::LocationConfirm => {
                self.jump_to_location();
            }
            EditorCommand::LocationCancel => {
                self.location_picker_visible = false;
                self.locations.clear();
                self.location_selected = 0;
            }

            // LSP - Code Actions
            EditorCommand::ShowCodeActions => {
                // Clear the filter when opening
                self.code_action_filter.clear();
                // If we already have cached code actions from the proactive check, show them
                if self.has_code_actions && !self.code_actions.is_empty() {
                    self.code_actions_visible = true;
                    self.code_action_selected = 0;
                } else {
                    // Otherwise trigger a fresh request
                    self.trigger_code_actions();
                }
            }
            EditorCommand::CodeActionUp => {
                let filtered_count = self.filtered_code_actions_count();
                if self.code_action_selected > 0 && filtered_count > 0 {
                    self.code_action_selected -= 1;
                }
            }
            EditorCommand::CodeActionDown => {
                let filtered_count = self.filtered_code_actions_count();
                if filtered_count > 0 && self.code_action_selected + 1 < filtered_count {
                    self.code_action_selected += 1;
                }
            }
            EditorCommand::CodeActionConfirm => {
                self.apply_code_action();
            }
            EditorCommand::CodeActionCancel => {
                self.code_actions_visible = false;
                self.code_actions.clear();
                self.code_action_selected = 0;
                self.code_action_filter.clear();
            }
            EditorCommand::CodeActionFilterChar(ch) => {
                self.code_action_filter.push(ch);
                self.code_action_selected = 0; // Reset selection when filter changes
            }
            EditorCommand::CodeActionFilterBackspace => {
                self.code_action_filter.pop();
                self.code_action_selected = 0; // Reset selection when filter changes
            }

            // LSP - Diagnostics
            EditorCommand::NextDiagnostic => {
                self.next_diagnostic(doc_id, view_id);
            }
            EditorCommand::PrevDiagnostic => {
                self.prev_diagnostic(doc_id, view_id);
            }
            EditorCommand::ShowDocumentDiagnostics => {
                self.show_document_diagnostics_picker();
            }
            EditorCommand::ShowWorkspaceDiagnostics => {
                self.show_workspace_diagnostics_picker();
            }

            // LSP - Format
            EditorCommand::FormatDocument => {
                // TODO: Trigger LSP format document
                log::info!("FormatDocument - not yet implemented");
            }

            // LSP - Rename
            EditorCommand::RenameSymbol => {
                self.show_rename_dialog();
            }

            // LSP - Inlay Hints
            EditorCommand::ToggleInlayHints => {
                self.inlay_hints_enabled = !self.inlay_hints_enabled;
                if !self.inlay_hints_enabled {
                    self.inlay_hints.clear();
                }
                log::info!(
                    "Inlay hints {}",
                    if self.inlay_hints_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            EditorCommand::RefreshInlayHints => {
                self.refresh_inlay_hints();
            }

            // LSP - Symbol Picker
            EditorCommand::ShowDocumentSymbols => {
                self.show_document_symbols();
            }
            EditorCommand::ShowWorkspaceSymbols => {
                self.show_workspace_symbols();
            }

            // LSP - Signature Help
            EditorCommand::TriggerSignatureHelp => {
                self.trigger_signature_help();
            }
            EditorCommand::CloseSignatureHelp => {
                self.signature_help_visible = false;
                self.signature_help = None;
            }

            // LSP - Internal responses
            EditorCommand::LspResponse(response) => {
                self.handle_lsp_response(response);
            }

            // LSP Dialog
            EditorCommand::ToggleLspDialog => {
                self.lsp_dialog_visible = !self.lsp_dialog_visible;
                if self.lsp_dialog_visible {
                    self.lsp_server_selected = 0;
                }
            }
            EditorCommand::CloseLspDialog => {
                self.lsp_dialog_visible = false;
            }
            EditorCommand::LspDialogUp => {
                if self.lsp_server_selected > 0 {
                    self.lsp_server_selected -= 1;
                }
            }
            EditorCommand::LspDialogDown => {
                let servers = self.collect_lsp_servers();
                if self.lsp_server_selected + 1 < servers.len() {
                    self.lsp_server_selected += 1;
                }
            }
            EditorCommand::RestartSelectedLsp => {
                let servers = self.collect_lsp_servers();
                if let Some(server) = servers.get(self.lsp_server_selected) {
                    let name = server.name.clone();
                    self.restart_lsp_server(&name);
                }
            }
            EditorCommand::RestartLspServer(name) => {
                self.restart_lsp_server(&name);
            }

            // Notifications
            EditorCommand::ShowNotification { message, severity } => {
                self.show_notification(message, severity);
            }
            EditorCommand::DismissNotification(id) => {
                self.notifications.retain(|n| n.id != id);
            }
            EditorCommand::DismissAllNotifications => {
                self.notifications.clear();
            }

            // Input Dialog
            EditorCommand::ShowInputDialog {
                title,
                prompt,
                placeholder,
                prefill,
                kind,
            } => {
                self.input_dialog_visible = true;
                self.input_dialog_title = title;
                self.input_dialog_prompt = prompt;
                self.input_dialog_placeholder = placeholder;
                self.input_dialog_value = prefill.unwrap_or_default();
                self.input_dialog_kind = kind;
            }
            EditorCommand::InputDialogInput(ch) => {
                self.input_dialog_value.push(ch);
            }
            EditorCommand::InputDialogBackspace => {
                self.input_dialog_value.pop();
            }
            EditorCommand::InputDialogConfirm => {
                self.handle_input_dialog_confirm();
            }
            EditorCommand::InputDialogCancel => {
                self.input_dialog_visible = false;
                self.input_dialog_value.clear();
                self.input_dialog_kind = InputDialogKind::None;
            }

            // Confirmation Dialog
            EditorCommand::ShowConfirmationDialog(dialog) => {
                self.confirmation_dialog = dialog;
                self.confirmation_dialog_visible = true;
            }
            EditorCommand::ConfirmationDialogConfirm => {
                self.handle_confirmation_dialog_confirm();
            }
            EditorCommand::ConfirmationDialogDeny => {
                self.handle_confirmation_dialog_deny();
            }
            EditorCommand::ConfirmationDialogCancel => {
                self.confirmation_dialog_visible = false;
                self.confirmation_dialog = ConfirmationDialogSnapshot::default();
            }

            // Global Search
            EditorCommand::ShowGlobalSearch => {
                self.show_global_search_picker();
            }
            EditorCommand::GlobalSearchExecute => {
                self.execute_global_search();
            }
            EditorCommand::GlobalSearchResults(results) => {
                self.global_search_results.extend(results);
                self.update_global_search_picker_items();
            }
            EditorCommand::GlobalSearchComplete => {
                self.global_search_running = false;
            }
        }
    }

    /// Handle an LSP response.
    fn handle_lsp_response(&mut self, response: LspResponse) {
        match response {
            LspResponse::Completions(items) => {
                self.completion_items = items;
                self.completion_selected = 0;
                self.completion_visible = !self.completion_items.is_empty();
            }
            LspResponse::Hover(hover) => {
                self.hover_content = hover;
                self.hover_visible = self.hover_content.is_some();
            }
            LspResponse::SignatureHelp(help) => {
                self.signature_help = help;
                self.signature_help_visible = self.signature_help.is_some();
            }
            LspResponse::InlayHints(hints) => {
                if self.inlay_hints_enabled {
                    self.inlay_hints = hints;
                }
            }
            LspResponse::GotoDefinition(locations) => {
                if locations.len() == 1 {
                    // Single location - jump directly
                    self.locations = locations;
                    self.location_selected = 0;
                    self.jump_to_location();
                } else if !locations.is_empty() {
                    // Multiple locations - show picker
                    self.locations = locations;
                    self.location_selected = 0;
                    self.location_picker_title = "Definitions".to_string();
                    self.location_picker_visible = true;
                }
            }
            LspResponse::References(locations) => {
                if locations.len() == 1 {
                    // Single location - jump directly
                    self.locations = locations;
                    self.location_selected = 0;
                    self.jump_to_location();
                } else if !locations.is_empty() {
                    // Multiple locations - show picker
                    self.locations = locations;
                    self.location_selected = 0;
                    self.location_picker_title = "References".to_string();
                    self.location_picker_visible = true;
                }
            }
            LspResponse::CodeActions(actions) => {
                self.code_actions = actions;
                self.code_action_selected = 0;
                self.code_actions_visible = !self.code_actions.is_empty();
                self.has_code_actions = !self.code_actions.is_empty();
            }
            LspResponse::CodeActionsAvailable(has_actions, cached_actions) => {
                self.has_code_actions = has_actions;
                // Cache the actions for quick access when menu is opened
                if has_actions && !cached_actions.is_empty() {
                    self.code_actions = cached_actions;
                    self.code_action_selected = 0;
                }
            }
            LspResponse::DiagnosticsUpdated => {
                // Diagnostics are pulled from the document in snapshot()
            }
            LspResponse::FormatApplied | LspResponse::WorkspaceEditApplied => {
                // Nothing to do - changes already applied
            }
            LspResponse::RenameResult {
                edit,
                offset_encoding,
                new_name,
            } => {
                // Apply the workspace edit
                if let Err(e) = self.editor.apply_workspace_edit(offset_encoding, &edit) {
                    log::error!("Failed to apply rename edit: {:?}", e);
                    self.show_notification(
                        format!("Rename failed: {:?}", e),
                        NotificationSeverity::Error,
                    );
                } else {
                    self.show_notification(
                        format!("Renamed to '{}'", new_name),
                        NotificationSeverity::Success,
                    );
                }
            }
            LspResponse::DocumentSymbols(symbols) => {
                self.symbols = symbols;
                self.populate_symbol_picker_items();
            }
            LspResponse::WorkspaceSymbols(symbols) => {
                self.symbols = symbols;
                self.populate_symbol_picker_items();
            }
            LspResponse::Error(msg) => {
                log::error!("LSP error: {}", msg);
            }
        }
    }

    /// Count the number of code actions that match the current filter.
    fn filtered_code_actions_count(&self) -> usize {
        if self.code_action_filter.is_empty() {
            return self.code_actions.len();
        }
        let filter_lower = self.code_action_filter.to_lowercase();
        self.code_actions
            .iter()
            .filter(|a| a.snapshot.title.to_lowercase().contains(&filter_lower))
            .count()
    }

    /// Get the filtered code actions based on the current filter.
    fn filtered_code_actions(&self) -> Vec<&StoredCodeAction> {
        if self.code_action_filter.is_empty() {
            return self.code_actions.iter().collect();
        }
        let filter_lower = self.code_action_filter.to_lowercase();
        self.code_actions
            .iter()
            .filter(|a| a.snapshot.title.to_lowercase().contains(&filter_lower))
            .collect()
    }

    /// Collect snapshots of all language servers.
    fn collect_lsp_servers(&self) -> Vec<LspServerSnapshot> {
        use helix_lsp::lsp::WorkDoneProgress;

        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let current_doc = self.editor.document(view.doc);

        // Get the current document's language server IDs for comparison
        let current_ls_ids: Vec<_> = current_doc
            .map(|doc| doc.language_servers().map(|ls| ls.id()).collect())
            .unwrap_or_default();

        // Iterate through all clients in the language server registry
        let mut servers: Vec<LspServerSnapshot> = self
            .editor
            .language_servers
            .iter_clients()
            .map(|client| {
                let name = client.name().to_string();
                let is_initialized = client.is_initialized();
                let is_progressing = self.lsp_progress.is_progressing(client.id());

                // Determine status based on initialization and progress
                let status = if !is_initialized {
                    LspServerStatus::Starting
                } else if is_progressing {
                    LspServerStatus::Indexing
                } else {
                    LspServerStatus::Running
                };

                // Get progress message if available
                let progress_message =
                    self.lsp_progress
                        .progress_map(client.id())
                        .and_then(|tokens| {
                            // Get the most recent progress with a message
                            tokens.values().find_map(|status| {
                                status.progress().and_then(|p| match p {
                                    WorkDoneProgress::Begin(begin) => Some(begin.title.clone()),
                                    WorkDoneProgress::Report(report) => {
                                        // Prefer message over title if available
                                        report.message.clone()
                                    }
                                    WorkDoneProgress::End(_) => None,
                                })
                            })
                        });

                // Get supported languages from client capabilities
                // Note: helix-lsp doesn't expose this directly, so we track it differently
                let languages = Vec::new(); // Will be populated from document associations

                // Check if this server is active for current document
                let active_for_current = current_ls_ids.contains(&client.id());

                LspServerSnapshot {
                    name,
                    status,
                    languages,
                    active_for_current,
                    progress_message,
                }
            })
            .collect();

        // Sort by name for consistent ordering
        servers.sort_by(|a, b| a.name.cmp(&b.name));

        servers
    }

    /// Restart a language server by name.
    fn restart_lsp_server(&mut self, name: &str) {
        log::info!("Restarting LSP server: {}", name);

        // Get the current document and its language config
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        // Extract necessary data from the document before mutable operations
        // Clone the Arc<LanguageConfiguration> so we can release the borrow on editor
        let (doc_path, lang_config) = {
            let Some(doc) = self.editor.document(doc_id) else {
                log::warn!("No document for LSP restart");
                return;
            };

            let Some(lang_config) = doc.language.clone() else {
                log::warn!("No language config for document");
                return;
            };

            (doc.path().map(|p| p.to_path_buf()), lang_config)
        };

        // Get editor config for workspace roots and snippets
        let editor_config = self.editor.config();
        let root_dirs = editor_config.workspace_lsp_roots.clone();
        let enable_snippets = editor_config.lsp.snippets;

        // Restart the server via registry
        match self.editor.language_servers.restart_server(
            name,
            &lang_config,
            doc_path.as_ref(),
            &root_dirs,
            enable_snippets,
        ) {
            Some(Ok(_client)) => {
                log::info!("LSP server '{}' restarted successfully", name);
            }
            Some(Err(e)) => {
                log::error!("Failed to restart LSP server '{}': {}", name, e);
                return;
            }
            None => {
                log::warn!("LSP server '{}' not found in registry", name);
                return;
            }
        }

        // Collect all document IDs that use this server
        let document_ids_to_refresh: Vec<helix_view::DocumentId> = self
            .editor
            .documents()
            .filter_map(|doc| {
                doc.language_config().and_then(|config| {
                    let uses_this_server = config.language_servers.iter().any(|ls| ls.name == name);
                    if uses_this_server {
                        Some(doc.id())
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Refresh language servers for all affected documents
        for document_id in &document_ids_to_refresh {
            self.editor.refresh_language_servers(*document_id);
        }

        log::info!(
            "Refreshed {} documents after restart",
            document_ids_to_refresh.len()
        );
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

        log::trace!(
            "snapshot: cursor={}, cursor_line={}, view_offset.anchor={}, visible_start={}, visible_end={}",
            cursor, cursor_line, view_offset.anchor, visible_start, visible_end
        );

        // Compute syntax highlighting tokens for visible lines
        let line_tokens = self.compute_syntax_tokens(doc, visible_start, visible_end);

        // Get selection range for highlighting
        let primary_range = selection.primary();
        let sel_start = primary_range.from();
        let sel_end = primary_range.to();

        // Only show selection highlighting in Select mode
        let is_select_mode = self.editor.mode() == Mode::Select;
        let has_selection = is_select_mode && sel_end > sel_start;

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

        // Collect diagnostics from doc before releasing the borrow
        let diagnostics = self.collect_diagnostics(doc, visible_start, visible_end);

        // Count total errors and warnings in the document
        let all_diagnostics = doc.diagnostics();
        let error_count = all_diagnostics
            .iter()
            .filter(|d| {
                d.severity
                    .is_some_and(|s| s == helix_core::diagnostic::Severity::Error)
            })
            .count();
        let warning_count = all_diagnostics
            .iter()
            .filter(|d| {
                d.severity
                    .is_some_and(|s| s == helix_core::diagnostic::Severity::Warning)
            })
            .count();

        let (open_buffers, buffer_scroll_offset) = self.buffer_bar_snapshot();

        // Increment snapshot version for change detection
        self.snapshot_version += 1;

        EditorSnapshot {
            snapshot_version: self.snapshot_version,
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
            // LSP state
            diagnostics,
            error_count,
            warning_count,
            completion_visible: self.completion_visible,
            completion_items: self.completion_items.clone(),
            completion_selected: self.completion_selected,
            hover_visible: self.hover_visible,
            hover_content: self.hover_content.clone(),
            signature_help_visible: self.signature_help_visible,
            signature_help: self.signature_help.clone(),
            inlay_hints: self.inlay_hints.clone(),
            inlay_hints_enabled: self.inlay_hints_enabled,
            code_actions_visible: self.code_actions_visible,
            code_actions: self
                .code_actions
                .iter()
                .map(|a| a.snapshot.clone())
                .collect(),
            code_action_selected: self.code_action_selected,
            code_action_filter: self.code_action_filter.clone(),
            has_code_actions: self.has_code_actions,
            location_picker_visible: self.location_picker_visible,
            locations: self.locations.clone(),
            location_selected: self.location_selected,
            location_picker_title: self.location_picker_title.clone(),
            // LSP dialog state
            lsp_dialog_visible: self.lsp_dialog_visible,
            lsp_servers: self.collect_lsp_servers(),
            lsp_server_selected: self.lsp_server_selected,
            // Notification state
            notifications: self.notifications.clone(),
            // Input dialog state
            input_dialog_visible: self.input_dialog_visible,
            input_dialog: InputDialogSnapshot {
                title: self.input_dialog_title.clone(),
                prompt: self.input_dialog_prompt.clone(),
                value: self.input_dialog_value.clone(),
                placeholder: self.input_dialog_placeholder.clone(),
            },
            // Confirmation dialog state
            confirmation_dialog_visible: self.confirmation_dialog_visible,
            confirmation_dialog: self.confirmation_dialog.clone(),
            should_quit: self.should_quit,
        }
    }

    /// Collect diagnostics for visible lines from the document.
    fn collect_diagnostics(
        &self,
        doc: &helix_view::Document,
        visible_start: usize,
        visible_end: usize,
    ) -> Vec<DiagnosticSnapshot> {
        let text = doc.text();
        let all_diags = doc.diagnostics();

        // Debug: log total diagnostics count
        if !all_diags.is_empty() {
            log::info!(
                "collect_diagnostics: found {} total diagnostics, visible range [{}, {})",
                all_diags.len(),
                visible_start,
                visible_end
            );
        }

        all_diags
            .iter()
            .filter_map(|diag| {
                let line = diag.line;
                // Only include diagnostics for visible lines
                if line < visible_start || line >= visible_end {
                    return None;
                }

                log::debug!(
                    "Including diagnostic on line {}: {}",
                    line + 1,
                    &diag.message[..diag.message.len().min(50)]
                );

                let line_start = text.line_to_char(line);
                let start_col = diag.range.start.saturating_sub(line_start);
                let end_col = diag.range.end.saturating_sub(line_start);

                Some(DiagnosticSnapshot {
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
                })
            })
            .collect()
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

// LSP operations implementation
impl EditorContext {
    /// Trigger completion at the current cursor position.
    pub(crate) fn trigger_completion(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for completion");
            return;
        };

        // Get language server with completion support
        let ls = match doc
            .language_servers_with_feature(LanguageServerFeature::Completion)
            .next()
        {
            Some(ls) => ls,
            None => {
                log::info!("No language server supports completion");
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let pos = doc.position(view_id, offset_encoding);
        let doc_id = doc.identifier();

        let context = lsp::CompletionContext {
            trigger_kind: lsp::CompletionTriggerKind::INVOKED,
            trigger_character: None,
        };

        let Some(future) = ls.completion(doc_id, pos, None, context) else {
            log::warn!("Failed to create completion request");
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(response)) => {
                    let items = convert_completion_response(response);
                    log::info!("Received {} completion items", items.len());
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::Completions(items)));
                }
                Ok(None) => {
                    log::info!("No completions received");
                }
                Err(e) => {
                    log::error!("Completion request failed: {}", e);
                }
            }
        });
    }

    /// Apply the selected completion item.
    pub(crate) fn apply_completion(&mut self) {
        let Some(item) = self.completion_items.get(self.completion_selected).cloned() else {
            return;
        };

        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document_mut(doc_id) else {
            return;
        };

        let text = doc.text();
        let selection = doc.selection(view_id);
        let cursor = selection.primary().cursor(text.slice(..));

        // Find word start to replace
        let mut word_start = cursor;
        while word_start > 0 {
            let ch = text.char(word_start - 1);
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            word_start -= 1;
        }

        // Create transaction to replace word with completion
        let transaction = helix_core::Transaction::change(
            text,
            [(word_start, cursor, Some(item.insert_text.as_str().into()))].into_iter(),
        );

        doc.apply(&transaction, view_id);

        // Clear completion state
        self.completion_visible = false;
        self.completion_items.clear();
        self.completion_selected = 0;
    }

    /// Trigger hover at the current cursor position.
    pub(crate) fn trigger_hover(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for hover");
            return;
        };

        // Get language server with hover support
        let ls = match doc
            .language_servers_with_feature(LanguageServerFeature::Hover)
            .next()
        {
            Some(ls) => ls,
            None => {
                log::info!("No language server supports hover");
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let pos = doc.position(view_id, offset_encoding);
        let doc_id = doc.identifier();

        let Some(future) = ls.text_document_hover(doc_id, pos, None) else {
            log::warn!("Failed to create hover request");
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(hover)) => {
                    let snapshot = convert_hover(hover);
                    log::info!("Received hover info");
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::Hover(Some(
                        snapshot,
                    ))));
                }
                Ok(None) => {
                    log::info!("No hover info available");
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::Hover(None)));
                }
                Err(e) => {
                    log::error!("Hover request failed: {}", e);
                }
            }
        });
    }

    /// Trigger goto definition at the current cursor position.
    pub(crate) fn trigger_goto_definition(&mut self) {
        self.trigger_goto(LanguageServerFeature::GotoDefinition, "Definition");
    }

    /// Trigger goto type definition at the current cursor position.
    pub(crate) fn trigger_goto_type_definition(&mut self) {
        self.trigger_goto(LanguageServerFeature::GotoTypeDefinition, "Type Definition");
    }

    /// Trigger goto implementation at the current cursor position.
    pub(crate) fn trigger_goto_implementation(&mut self) {
        self.trigger_goto(LanguageServerFeature::GotoImplementation, "Implementation");
    }

    /// Generic goto operation helper - spawns the async task.
    fn spawn_goto_request<F>(
        tx: mpsc::Sender<EditorCommand>,
        future: F,
        offset_encoding: helix_lsp::OffsetEncoding,
        title: String,
    ) where
        F: std::future::Future<Output = helix_lsp::Result<Option<lsp::GotoDefinitionResponse>>>
            + Send
            + 'static,
    {
        tokio::spawn(async move {
            match future.await {
                Ok(Some(response)) => {
                    let locations = convert_goto_response(response, offset_encoding);
                    log::info!("Received {} {} locations", locations.len(), title);
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::GotoDefinition(
                        locations,
                    )));
                }
                Ok(None) => {
                    log::info!("No {} found", title);
                }
                Err(e) => {
                    log::error!("{} request failed: {}", title, e);
                }
            }
        });
    }

    /// Generic goto operation.
    fn trigger_goto(&mut self, feature: LanguageServerFeature, title: &str) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for goto");
            return;
        };

        // Get language server with the feature
        let ls = match doc.language_servers_with_feature(feature).next() {
            Some(ls) => ls,
            None => {
                log::info!("No language server supports {:?}", feature);
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let pos = doc.position(view_id, offset_encoding);
        let doc_id_lsp = doc.identifier();
        let tx = self.command_tx.clone();
        let title_string = title.to_string();

        match feature {
            LanguageServerFeature::GotoDefinition => {
                if let Some(future) = ls.goto_definition(doc_id_lsp, pos, None) {
                    Self::spawn_goto_request(tx, future, offset_encoding, title_string);
                }
            }
            LanguageServerFeature::GotoTypeDefinition => {
                if let Some(future) = ls.goto_type_definition(doc_id_lsp, pos, None) {
                    Self::spawn_goto_request(tx, future, offset_encoding, title_string);
                }
            }
            LanguageServerFeature::GotoImplementation => {
                if let Some(future) = ls.goto_implementation(doc_id_lsp, pos, None) {
                    Self::spawn_goto_request(tx, future, offset_encoding, title_string);
                }
            }
            _ => {
                log::warn!("Unsupported goto feature: {:?}", feature);
            }
        }
    }

    /// Trigger find references at the current cursor position.
    pub(crate) fn trigger_goto_references(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for references");
            return;
        };

        // Get language server with references support
        let ls = match doc
            .language_servers_with_feature(LanguageServerFeature::GotoReference)
            .next()
        {
            Some(ls) => ls,
            None => {
                log::info!("No language server supports references");
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let pos = doc.position(view_id, offset_encoding);
        let doc_id = doc.identifier();

        let Some(future) = ls.goto_reference(doc_id, pos, true, None) else {
            log::warn!("Failed to create references request");
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(locations)) => {
                    let snapshots = convert_references_response(locations, offset_encoding);
                    log::info!("Received {} references", snapshots.len());
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::References(
                        snapshots,
                    )));
                }
                Ok(None) => {
                    log::info!("No references found");
                }
                Err(e) => {
                    log::error!("References request failed: {}", e);
                }
            }
        });
    }

    /// Jump to the selected location.
    pub(crate) fn jump_to_location(&mut self) {
        let Some(location) = self.locations.get(self.location_selected).cloned() else {
            return;
        };

        // Close picker
        self.location_picker_visible = false;
        self.locations.clear();
        self.location_selected = 0;

        // Open file and jump to position
        let view_id = self.editor.tree.focus;

        // Open the file
        if let Err(e) = self
            .editor
            .open(&location.path, helix_view::editor::Action::Replace)
        {
            log::error!("Failed to open file {:?}: {}", location.path, e);
            return;
        }

        // Get the newly opened document
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;
        let doc = match self.editor.document_mut(doc_id) {
            Some(d) => d,
            None => return,
        };

        // Calculate cursor position
        let text = doc.text();
        let line = (location.line - 1).min(text.len_lines().saturating_sub(1));
        let line_start = text.line_to_char(line);
        let line_len = text.line(line).len_chars();
        let col = (location.column - 1).min(line_len.saturating_sub(1));
        let pos = line_start + col;

        // Set cursor position
        let selection = helix_core::Selection::point(pos);
        doc.set_selection(view_id, selection);
    }

    /// Trigger signature help at the current cursor position.
    pub(crate) fn trigger_signature_help(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for signature help");
            return;
        };

        // Get language server with signature help support
        let ls = match doc
            .language_servers_with_feature(LanguageServerFeature::SignatureHelp)
            .next()
        {
            Some(ls) => ls,
            None => {
                log::info!("No language server supports signature help");
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let pos = doc.position(view_id, offset_encoding);
        let doc_id = doc.identifier();

        let Some(future) = ls.text_document_signature_help(doc_id, pos, None) else {
            log::warn!("Failed to create signature help request");
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(help)) => {
                    let snapshot = convert_signature_help(help);
                    log::info!("Received signature help");
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::SignatureHelp(
                        Some(snapshot),
                    )));
                }
                Ok(None) => {
                    log::info!("No signature help available");
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::SignatureHelp(None)));
                }
                Err(e) => {
                    log::error!("Signature help request failed: {}", e);
                }
            }
        });
    }

    /// Trigger code actions at the current cursor position.
    pub(crate) fn trigger_code_actions(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for code actions");
            return;
        };

        // Get language server with code actions support
        let ls = match doc
            .language_servers_with_feature(LanguageServerFeature::CodeAction)
            .next()
        {
            Some(ls) => ls,
            None => {
                log::info!("No language server supports code actions");
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let language_server_id = ls.id();
        let text = doc.text();
        let selection = doc.selection(view_id);
        let cursor = selection.primary().cursor(text.slice(..));
        let cursor_line = text.char_to_line(cursor);
        let line_start = text.line_to_char(cursor_line);
        let cursor_col = cursor - line_start;

        // Create range for the cursor position
        let range = lsp::Range {
            start: lsp::Position {
                line: cursor_line as u32,
                character: cursor_col as u32,
            },
            end: lsp::Position {
                line: cursor_line as u32,
                character: cursor_col as u32,
            },
        };

        // Get diagnostics at cursor position for context
        let diagnostics: Vec<lsp::Diagnostic> = doc
            .diagnostics()
            .iter()
            .filter(|d| d.line == cursor_line)
            .filter_map(|d| {
                // Convert to LSP diagnostic (simplified)
                Some(lsp::Diagnostic {
                    range: lsp::Range {
                        start: lsp::Position {
                            line: d.line as u32,
                            character: (d.range.start - line_start) as u32,
                        },
                        end: lsp::Position {
                            line: d.line as u32,
                            character: (d.range.end - line_start) as u32,
                        },
                    },
                    message: d.message.clone(),
                    severity: d.severity.map(|s| match s {
                        helix_core::diagnostic::Severity::Error => lsp::DiagnosticSeverity::ERROR,
                        helix_core::diagnostic::Severity::Warning => {
                            lsp::DiagnosticSeverity::WARNING
                        }
                        helix_core::diagnostic::Severity::Info => {
                            lsp::DiagnosticSeverity::INFORMATION
                        }
                        helix_core::diagnostic::Severity::Hint => lsp::DiagnosticSeverity::HINT,
                    }),
                    ..Default::default()
                })
            })
            .collect();

        let context = lsp::CodeActionContext {
            diagnostics,
            only: None,
            trigger_kind: Some(lsp::CodeActionTriggerKind::INVOKED),
        };

        let doc_id = doc.identifier();

        let Some(future) = ls.code_actions(doc_id, range, context) else {
            log::warn!("Failed to create code actions request");
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(actions)) => {
                    let stored_actions =
                        convert_code_actions(actions, language_server_id, offset_encoding);
                    log::info!("Received {} code actions", stored_actions.len());
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::CodeActions(
                        stored_actions,
                    )));
                }
                Ok(None) => {
                    log::info!("No code actions available");
                }
                Err(e) => {
                    log::error!("Code actions request failed: {}", e);
                }
            }
        });
    }

    /// Apply the selected code action.
    pub(crate) fn apply_code_action(&mut self) {
        // Get the selected action from the filtered list
        let filtered = self.filtered_code_actions();
        let Some(action) = filtered.get(self.code_action_selected).cloned().cloned() else {
            self.code_actions_visible = false;
            self.code_actions.clear();
            self.code_action_selected = 0;
            self.code_action_filter.clear();
            return;
        };

        // Close the menu first
        self.code_actions_visible = false;
        self.code_actions.clear();
        self.code_action_selected = 0;
        self.code_action_filter.clear();

        match &action.lsp_item {
            lsp::CodeActionOrCommand::Command(command) => {
                log::info!("Executing LSP command: {}", command.title);
                // Execute command on language server
                self.editor
                    .execute_lsp_command(command.clone(), action.language_server_id);
            }
            lsp::CodeActionOrCommand::CodeAction(code_action) => {
                log::info!("Applying code action: {}", code_action.title);

                // Resolve code action if edit or command is missing.
                // Many LSP servers don't include the full edit in the initial response
                // and require a "codeAction/resolve" request to get the workspace edit.
                let resolved_code_action = if code_action.edit.is_none()
                    || code_action.command.is_none()
                {
                    if let Some(ls) = self.editor.language_server_by_id(action.language_server_id) {
                        if let Some(future) = ls.resolve_code_action(code_action) {
                            match helix_lsp::block_on(future) {
                                Ok(resolved) => {
                                    log::info!(
                                        "Resolved code action, edit present: {}",
                                        resolved.edit.is_some()
                                    );
                                    Some(resolved)
                                }
                                Err(e) => {
                                    log::error!("Failed to resolve code action: {}", e);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        log::warn!("Language server not found for code action");
                        None
                    }
                } else {
                    None
                };

                let resolved = resolved_code_action.as_ref().unwrap_or(code_action);

                // Apply workspace edit if present
                if let Some(ref workspace_edit) = resolved.edit {
                    log::info!(
                        "Applying workspace edit (has changes: {}, has document_changes: {})",
                        workspace_edit.changes.is_some(),
                        workspace_edit.document_changes.is_some()
                    );
                    if let Err(e) = self
                        .editor
                        .apply_workspace_edit(action.offset_encoding, workspace_edit)
                    {
                        log::error!("Failed to apply workspace edit: {:?}", e);
                    }
                } else {
                    log::warn!("Code action has no workspace edit after resolution");
                }

                // Execute command if present (after edit)
                if let Some(command) = &resolved.command {
                    self.editor
                        .execute_lsp_command(command.clone(), action.language_server_id);
                }
            }
        }
    }

    /// Check if code actions are available at the current cursor position.
    /// This is called proactively to update the lightbulb indicator.
    pub(crate) fn check_code_actions_available(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            self.has_code_actions = false;
            return;
        };

        // Get cursor position
        let text = doc.text();
        let selection = doc.selection(view_id);
        let cursor = selection.primary().cursor(text.slice(..));
        let cursor_line = text.char_to_line(cursor);
        let line_start = text.line_to_char(cursor_line);
        let cursor_col = cursor - line_start;

        // Check if position changed since last check
        let current_pos = (doc_id, cursor_line, cursor_col);
        if self.code_actions_check_position == Some(current_pos) {
            // Position unchanged, skip check
            return;
        }
        self.code_actions_check_position = Some(current_pos);

        // Get language server with code actions support
        let ls = match doc
            .language_servers_with_feature(LanguageServerFeature::CodeAction)
            .next()
        {
            Some(ls) => ls,
            None => {
                self.has_code_actions = false;
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let language_server_id = ls.id();

        // Create range for the cursor position
        let range = lsp::Range {
            start: lsp::Position {
                line: cursor_line as u32,
                character: cursor_col as u32,
            },
            end: lsp::Position {
                line: cursor_line as u32,
                character: cursor_col as u32,
            },
        };

        // Get diagnostics at cursor position for context
        let diagnostics: Vec<lsp::Diagnostic> = doc
            .diagnostics()
            .iter()
            .filter(|d| d.line == cursor_line)
            .filter_map(|d| {
                Some(lsp::Diagnostic {
                    range: lsp::Range {
                        start: lsp::Position {
                            line: d.line as u32,
                            character: (d.range.start - line_start) as u32,
                        },
                        end: lsp::Position {
                            line: d.line as u32,
                            character: (d.range.end - line_start) as u32,
                        },
                    },
                    message: d.message.clone(),
                    severity: d.severity.map(|s| match s {
                        helix_core::diagnostic::Severity::Error => lsp::DiagnosticSeverity::ERROR,
                        helix_core::diagnostic::Severity::Warning => {
                            lsp::DiagnosticSeverity::WARNING
                        }
                        helix_core::diagnostic::Severity::Info => {
                            lsp::DiagnosticSeverity::INFORMATION
                        }
                        helix_core::diagnostic::Severity::Hint => lsp::DiagnosticSeverity::HINT,
                    }),
                    ..Default::default()
                })
            })
            .collect();

        let context = lsp::CodeActionContext {
            diagnostics,
            only: None,
            trigger_kind: Some(lsp::CodeActionTriggerKind::AUTOMATIC),
        };

        let doc_id = doc.identifier();

        let Some(future) = ls.code_actions(doc_id, range, context) else {
            self.has_code_actions = false;
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(actions)) => {
                    let has_actions = !actions.is_empty();
                    // Send response to update has_code_actions state
                    let stored_actions =
                        convert_code_actions(actions, language_server_id, offset_encoding);
                    let _ = tx.send(EditorCommand::LspResponse(
                        LspResponse::CodeActionsAvailable(has_actions, stored_actions),
                    ));
                }
                Ok(None) => {
                    let _ = tx.send(EditorCommand::LspResponse(
                        LspResponse::CodeActionsAvailable(false, Vec::new()),
                    ));
                }
                Err(e) => {
                    log::debug!("Code actions check failed: {}", e);
                    let _ = tx.send(EditorCommand::LspResponse(
                        LspResponse::CodeActionsAvailable(false, Vec::new()),
                    ));
                }
            }
        });
    }

    /// Refresh inlay hints for the visible viewport.
    pub(crate) fn refresh_inlay_hints(&mut self) {
        if !self.inlay_hints_enabled {
            return;
        }

        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for inlay hints");
            return;
        };

        // Get language server with inlay hints support
        let ls = match doc
            .language_servers_with_feature(LanguageServerFeature::InlayHints)
            .next()
        {
            Some(ls) => ls,
            None => {
                log::debug!("No language server supports inlay hints");
                return;
            }
        };

        let offset_encoding = ls.offset_encoding();
        let text = doc.text();
        let total_lines = text.len_lines();

        // Request hints for whole document (could optimize for viewport later)
        let range = lsp::Range {
            start: lsp::Position {
                line: 0,
                character: 0,
            },
            end: lsp::Position {
                line: total_lines as u32,
                character: 0,
            },
        };

        let doc_id = doc.identifier();

        let Some(future) = ls.text_document_range_inlay_hints(doc_id, range, None) else {
            log::warn!("Failed to create inlay hints request");
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(hints)) => {
                    let snapshots = convert_inlay_hints(hints, offset_encoding);
                    log::info!("Received {} inlay hints", snapshots.len());
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::InlayHints(
                        snapshots,
                    )));
                }
                Ok(None) => {
                    log::debug!("No inlay hints available");
                }
                Err(e) => {
                    log::error!("Inlay hints request failed: {}", e);
                }
            }
        });
    }

    /// Show a notification toast.
    pub(crate) fn show_notification(&mut self, message: String, severity: NotificationSeverity) {
        self.notification_id_counter += 1;
        let notification = NotificationSnapshot {
            id: self.notification_id_counter,
            message,
            severity,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };
        self.notifications.push(notification);

        // Auto-dismiss after 5 seconds - schedule via command
        let id = self.notification_id_counter;
        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let _ = tx.send(EditorCommand::DismissNotification(id));
        });
    }

    /// Show the rename dialog with the word under cursor prefilled.
    fn show_rename_dialog(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            self.show_notification(
                "No document for rename".to_string(),
                NotificationSeverity::Error,
            );
            return;
        };

        // Check if rename is supported
        if doc
            .language_servers_with_feature(LanguageServerFeature::RenameSymbol)
            .next()
            .is_none()
        {
            self.show_notification(
                "No language server supports rename".to_string(),
                NotificationSeverity::Error,
            );
            return;
        }

        // Get the word under cursor
        let word = self.get_word_under_cursor(doc_id, view_id);

        // Show input dialog
        self.input_dialog_visible = true;
        self.input_dialog_title = "Rename Symbol".to_string();
        self.input_dialog_prompt = "New name:".to_string();
        self.input_dialog_placeholder = Some(word.clone());
        self.input_dialog_value = word;
        self.input_dialog_kind = InputDialogKind::RenameSymbol;
    }

    /// Get the word under cursor.
    fn get_word_under_cursor(
        &self,
        doc_id: helix_view::DocumentId,
        view_id: helix_view::ViewId,
    ) -> String {
        let Some(doc) = self.editor.document(doc_id) else {
            return String::new();
        };

        let text = doc.text();
        let selection = doc.selection(view_id);
        let cursor = selection.primary().cursor(text.slice(..));

        // Find word boundaries
        let mut start = cursor;
        let mut end = cursor;

        // Move start to beginning of word
        while start > 0 {
            let ch = text.char(start - 1);
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            start -= 1;
        }

        // Move end to end of word
        let len = text.len_chars();
        while end < len {
            let ch = text.char(end);
            if !ch.is_alphanumeric() && ch != '_' {
                break;
            }
            end += 1;
        }

        if start < end {
            text.slice(start..end).to_string()
        } else {
            String::new()
        }
    }

    /// Handle input dialog confirmation based on the dialog kind.
    fn handle_input_dialog_confirm(&mut self) {
        let value = self.input_dialog_value.clone();
        let kind = self.input_dialog_kind;

        // Close the dialog
        self.input_dialog_visible = false;
        self.input_dialog_value.clear();
        self.input_dialog_kind = InputDialogKind::None;

        match kind {
            InputDialogKind::None => {}
            InputDialogKind::RenameSymbol => {
                self.execute_rename_symbol(&value);
            }
        }
    }

    /// Show the native Save As dialog for scratch buffers.
    /// Uses the system file dialog via rfd crate (async to avoid blocking UI).
    pub(crate) fn show_save_as_dialog(&mut self) {
        use rfd::AsyncFileDialog;

        // Get current working directory for default location
        let start_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            // Show native save dialog asynchronously
            let file_handle = AsyncFileDialog::new()
                .set_directory(&start_dir)
                .set_file_name("untitled")
                .save_file()
                .await;

            if let Some(handle) = file_handle {
                let path = handle.path().to_path_buf();
                let _ = tx.send(EditorCommand::SaveDocumentToPath(path));
            }
            // If user cancelled, do nothing
        });
    }

    /// Handle confirmation dialog confirm button (yes/save action).
    fn handle_confirmation_dialog_confirm(&mut self) {
        use types::ConfirmationAction;

        let action = self.confirmation_dialog.action;

        // Close the dialog
        self.confirmation_dialog_visible = false;
        self.confirmation_dialog = ConfirmationDialogSnapshot::default();

        match action {
            ConfirmationAction::None => {}
            ConfirmationAction::SaveAndQuit => {
                // Save first, then quit
                self.save_document(None, false);
                self.should_quit = true;
            }
            ConfirmationAction::QuitWithoutSave => {
                // This is handled by deny, but can also be confirm if only two buttons
                self.should_quit = true;
            }
            ConfirmationAction::CloseBuffer => {
                // Force close the buffer
                self.close_current_buffer(true);
            }
            ConfirmationAction::ReloadFile => {
                // TODO: Implement file reload
                log::info!("ReloadFile not yet implemented");
            }
        }
    }

    /// Handle confirmation dialog deny button (no/don't save action).
    fn handle_confirmation_dialog_deny(&mut self) {
        use types::ConfirmationAction;

        let action = self.confirmation_dialog.action;

        // Close the dialog
        self.confirmation_dialog_visible = false;
        self.confirmation_dialog = ConfirmationDialogSnapshot::default();

        match action {
            ConfirmationAction::None => {}
            ConfirmationAction::SaveAndQuit => {
                // User chose "Don't Save" - quit without saving
                self.should_quit = true;
            }
            ConfirmationAction::QuitWithoutSave | ConfirmationAction::CloseBuffer => {
                // Deny on these actions means "cancel" - do nothing
            }
            ConfirmationAction::ReloadFile => {
                // Don't reload - do nothing
            }
        }
    }

    /// Execute LSP rename with the given new name.
    fn execute_rename_symbol(&mut self, new_name: &str) {
        if new_name.is_empty() {
            self.show_notification(
                "Rename cancelled: empty name".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }

        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        // Extract what we need in a scope to release the borrow
        let rename_data = {
            let Some(doc) = self.editor.document(doc_id) else {
                return;
            };

            // Get language server with rename support
            let ls = match doc
                .language_servers_with_feature(LanguageServerFeature::RenameSymbol)
                .next()
            {
                Some(ls) => ls,
                None => {
                    return;
                }
            };

            let offset_encoding = ls.offset_encoding();
            let pos = doc.position(view_id, offset_encoding);
            let doc_id_lsp = doc.identifier();
            let new_name_owned = new_name.to_string();

            let future = ls.rename_symbol(doc_id_lsp, pos, new_name_owned.clone());

            future.map(|f| (f, offset_encoding, new_name_owned))
        };

        let Some((future, offset_encoding, new_name_owned)) = rename_data else {
            self.show_notification(
                "No language server supports rename".to_string(),
                NotificationSeverity::Error,
            );
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(edit)) => {
                    log::info!("Received rename response");
                    // Send the workspace edit to be applied
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::RenameResult {
                        edit,
                        offset_encoding,
                        new_name: new_name_owned,
                    }));
                }
                Ok(None) => {
                    log::info!("No rename results");
                    let _ = tx.send(EditorCommand::ShowNotification {
                        message: "No rename results".to_string(),
                        severity: NotificationSeverity::Info,
                    });
                }
                Err(e) => {
                    log::error!("Rename request failed: {}", e);
                    let _ = tx.send(EditorCommand::ShowNotification {
                        message: format!("Rename failed: {}", e),
                        severity: NotificationSeverity::Error,
                    });
                }
            }
        });
    }

    /// Show the document symbols picker.
    fn show_document_symbols(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        // Extract needed data before mutable borrow
        let future = {
            let Some(doc) = self.editor.document(doc_id) else {
                log::warn!("No document for document symbols");
                return;
            };

            // Get language server with document symbols support
            let ls = match doc
                .language_servers_with_feature(LanguageServerFeature::DocumentSymbols)
                .next()
            {
                Some(ls) => ls,
                None => {
                    log::info!("No language server supports document symbols");
                    return;
                }
            };

            let doc_id_lsp = doc.identifier();
            ls.document_symbols(doc_id_lsp)
        };

        let Some(future) = future else {
            log::warn!("Failed to create document symbols request");
            return;
        };

        // Set up picker state
        self.picker_mode = PickerMode::DocumentSymbols;
        self.picker_visible = true;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_items.clear();
        self.symbols.clear();
        self.picker_current_path = None;

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(response)) => {
                    let symbols = convert_document_symbols(response);
                    log::info!("Received {} document symbols", symbols.len());
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::DocumentSymbols(
                        symbols,
                    )));
                }
                Ok(None) => {
                    log::info!("No document symbols available");
                }
                Err(e) => {
                    log::error!("Document symbols request failed: {}", e);
                }
            }
        });
    }

    /// Show the workspace symbols picker.
    fn show_workspace_symbols(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        // Extract language server ID before mutable borrow
        let language_server_id = {
            let Some(doc) = self.editor.document(doc_id) else {
                log::warn!("No document for workspace symbols");
                return;
            };

            // Get language server with workspace symbols support
            let ls = match doc
                .language_servers_with_feature(LanguageServerFeature::WorkspaceSymbols)
                .next()
            {
                Some(ls) => ls,
                None => {
                    log::info!("No language server supports workspace symbols");
                    return;
                }
            };

            ls.id()
        };

        // Set up picker state - initially empty, will populate on filter input
        self.picker_mode = PickerMode::WorkspaceSymbols;
        self.picker_visible = true;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_items.clear();
        self.symbols.clear();
        self.picker_current_path = None;

        // Trigger initial workspace symbols search with empty query
        self.trigger_workspace_symbols_search(language_server_id, String::new());
    }

    /// Trigger a workspace symbols search with the given query.
    fn trigger_workspace_symbols_search(
        &self,
        language_server_id: helix_lsp::LanguageServerId,
        query: String,
    ) {
        let Some(ls) = self.editor.language_server_by_id(language_server_id) else {
            return;
        };

        let Some(future) = ls.workspace_symbols(query) else {
            log::warn!("Failed to create workspace symbols request");
            return;
        };

        let tx = self.command_tx.clone();
        tokio::spawn(async move {
            match future.await {
                Ok(Some(response)) => {
                    let symbols = convert_workspace_symbols(response);
                    log::info!("Received {} workspace symbols", symbols.len());
                    let _ = tx.send(EditorCommand::LspResponse(LspResponse::WorkspaceSymbols(
                        symbols,
                    )));
                }
                Ok(None) => {
                    log::info!("No workspace symbols available");
                }
                Err(e) => {
                    log::error!("Workspace symbols request failed: {}", e);
                }
            }
        });
    }

    /// Show the document diagnostics picker.
    fn show_document_diagnostics_picker(&mut self) {
        let view_id = self.editor.tree.focus;
        let view = self.editor.tree.get(view_id);
        let doc_id = view.doc;

        let Some(doc) = self.editor.document(doc_id) else {
            log::warn!("No document for document diagnostics");
            return;
        };

        let text = doc.text();

        // Collect diagnostics from the current document
        let mut entries: Vec<DiagnosticPickerEntry> = doc
            .diagnostics()
            .iter()
            .map(|d| {
                let line = text.char_to_line(d.range.start);
                let line_start = text.line_to_char(line);
                let start_col = d.range.start - line_start;
                let end_col = d.range.end - line_start;

                let snapshot = DiagnosticSnapshot {
                    line: line + 1, // 1-indexed
                    start_col,
                    end_col,
                    message: d.message.clone(),
                    severity: d
                        .severity
                        .map(DiagnosticSeverity::from)
                        .unwrap_or_default(),
                    source: d.source.clone(),
                    code: convert_diagnostic_code(&d.code),
                };

                DiagnosticPickerEntry {
                    diagnostic: snapshot,
                    doc_id: Some(doc_id),
                    path: None,
                }
            })
            .collect();

        // Sort by line number
        entries.sort_by_key(|e| e.diagnostic.line);

        self.picker_diagnostics = entries;
        self.populate_diagnostic_picker_items();

        // Set up picker state
        self.picker_mode = PickerMode::DocumentDiagnostics;
        self.picker_visible = true;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_current_path = None;

        log::info!(
            "Showing {} document diagnostics",
            self.picker_diagnostics.len()
        );
    }

    /// Show the workspace diagnostics picker.
    fn show_workspace_diagnostics_picker(&mut self) {
        // Collect diagnostics from all open documents
        let mut entries: Vec<DiagnosticPickerEntry> = Vec::new();

        for (&doc_id, doc) in self.editor.documents.iter() {
            let text = doc.text();
            let path = doc.path().map(|p| p.to_path_buf());

            for d in doc.diagnostics() {
                let line = text.char_to_line(d.range.start);
                let line_start = text.line_to_char(line);
                let start_col = d.range.start - line_start;
                let end_col = d.range.end - line_start;

                let snapshot = DiagnosticSnapshot {
                    line: line + 1, // 1-indexed
                    start_col,
                    end_col,
                    message: d.message.clone(),
                    severity: d
                        .severity
                        .map(DiagnosticSeverity::from)
                        .unwrap_or_default(),
                    source: d.source.clone(),
                    code: convert_diagnostic_code(&d.code),
                };

                entries.push(DiagnosticPickerEntry {
                    diagnostic: snapshot,
                    doc_id: Some(doc_id),
                    path: path.clone(),
                });
            }
        }

        // Sort by severity (errors first), then by file, then by line
        entries.sort_by(|a, b| {
            // Compare severity (Error > Warning > Info > Hint)
            let sev_a = get_severity_sort_key(a.diagnostic.severity);
            let sev_b = get_severity_sort_key(b.diagnostic.severity);
            match sev_a.cmp(&sev_b) {
                std::cmp::Ordering::Equal => {
                    // Then by path
                    match (&a.path, &b.path) {
                        (Some(pa), Some(pb)) => match pa.cmp(pb) {
                            std::cmp::Ordering::Equal => a.diagnostic.line.cmp(&b.diagnostic.line),
                            ord => ord,
                        },
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => a.diagnostic.line.cmp(&b.diagnostic.line),
                    }
                }
                ord => ord,
            }
        });

        self.picker_diagnostics = entries;
        self.populate_diagnostic_picker_items();

        // Set up picker state
        self.picker_mode = PickerMode::WorkspaceDiagnostics;
        self.picker_visible = true;
        self.picker_filter.clear();
        self.picker_selected = 0;
        self.picker_current_path = None;

        log::info!(
            "Showing {} workspace diagnostics",
            self.picker_diagnostics.len()
        );
    }

    /// Populate picker items from diagnostics.
    fn populate_diagnostic_picker_items(&mut self) {
        self.picker_items = self
            .picker_diagnostics
            .iter()
            .enumerate()
            .map(|(idx, entry)| {
                let icon = get_diagnostic_icon(entry.diagnostic.severity);

                // Build display with severity badge and optional code
                let severity_label = match entry.diagnostic.severity {
                    DiagnosticSeverity::Error => "error",
                    DiagnosticSeverity::Warning => "warn",
                    DiagnosticSeverity::Info => "info",
                    DiagnosticSeverity::Hint => "hint",
                };

                // Format: "[error] message" or "[error E0308] message"
                let prefix = match &entry.diagnostic.code {
                    Some(code) => format!("[{} {}]", severity_label, code),
                    None => format!("[{}]", severity_label),
                };

                // Truncate message to fit (accounting for prefix)
                let max_msg_len = 70usize.saturating_sub(prefix.len());
                let message = if entry.diagnostic.message.len() > max_msg_len {
                    format!("{}...", &entry.diagnostic.message[..max_msg_len.saturating_sub(3)])
                } else {
                    entry.diagnostic.message.clone()
                };

                let display = format!("{} {}", prefix, message);

                // Secondary: "filename:line" for workspace, "Line N" for document
                let secondary = match &entry.path {
                    Some(path) => {
                        let filename = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        Some(format!("{}:{}", filename, entry.diagnostic.line))
                    }
                    None => Some(format!("Line {}", entry.diagnostic.line)),
                };

                PickerItem {
                    id: idx.to_string(),
                    display,
                    icon,
                    match_indices: vec![],
                    secondary,
                }
            })
            .collect();
    }

    /// Populate picker items from symbols.
    fn populate_symbol_picker_items(&mut self) {
        self.picker_items = self
            .symbols
            .iter()
            .enumerate()
            .map(|(idx, sym)| {
                let icon = symbol_kind_to_picker_icon(sym.kind);
                let secondary = match (&sym.container_name, &sym.path) {
                    (Some(container), Some(path)) => {
                        Some(format!("{} · {}", container, path.display()))
                    }
                    (Some(container), None) => Some(container.clone()),
                    (None, Some(path)) => Some(format!("{}", path.display())),
                    (None, None) => Some(format!("Line {}", sym.line)),
                };

                PickerItem {
                    id: idx.to_string(),
                    display: sym.name.clone(),
                    icon,
                    match_indices: vec![],
                    secondary,
                }
            })
            .collect();
    }
}

/// Convert SymbolKind to PickerIcon.
fn symbol_kind_to_picker_icon(kind: SymbolKind) -> PickerIcon {
    match kind {
        SymbolKind::Function => PickerIcon::SymbolFunction,
        SymbolKind::Method | SymbolKind::Constructor => PickerIcon::SymbolMethod,
        SymbolKind::Class => PickerIcon::SymbolClass,
        SymbolKind::Struct => PickerIcon::SymbolStruct,
        SymbolKind::Enum | SymbolKind::EnumMember => PickerIcon::SymbolEnum,
        SymbolKind::Interface => PickerIcon::SymbolInterface,
        SymbolKind::Variable => PickerIcon::SymbolVariable,
        SymbolKind::Constant => PickerIcon::SymbolConstant,
        SymbolKind::Field | SymbolKind::Property => PickerIcon::SymbolField,
        SymbolKind::Module | SymbolKind::Namespace | SymbolKind::Package => {
            PickerIcon::SymbolModule
        }
        _ => PickerIcon::SymbolOther,
    }
}

/// Convert DiagnosticSeverity to PickerIcon.
fn get_diagnostic_icon(severity: DiagnosticSeverity) -> PickerIcon {
    match severity {
        DiagnosticSeverity::Error => PickerIcon::DiagnosticError,
        DiagnosticSeverity::Warning => PickerIcon::DiagnosticWarning,
        DiagnosticSeverity::Info => PickerIcon::DiagnosticInfo,
        DiagnosticSeverity::Hint => PickerIcon::DiagnosticHint,
    }
}

/// Get sort key for diagnostic severity (lower = higher priority).
fn get_severity_sort_key(severity: DiagnosticSeverity) -> u8 {
    match severity {
        DiagnosticSeverity::Error => 0,
        DiagnosticSeverity::Warning => 1,
        DiagnosticSeverity::Info => 2,
        DiagnosticSeverity::Hint => 3,
    }
}

/// Convert diagnostic code from NumberOrString to Option<String>.
fn convert_diagnostic_code(
    code: &Option<helix_core::diagnostic::NumberOrString>,
) -> Option<String> {
    code.as_ref().map(|c| match c {
        helix_core::diagnostic::NumberOrString::Number(n) => n.to_string(),
        helix_core::diagnostic::NumberOrString::String(s) => s.clone(),
    })
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

/// Create handlers for initialization and register essential hooks.
fn create_handlers() -> helix_view::handlers::Handlers {
    use helix_view::handlers::completion::CompletionHandler;
    use helix_view::handlers::*;
    use tokio::sync::mpsc::channel;

    let (completion_tx, _) = channel(1);
    let (signature_tx, _) = channel(1);
    let (auto_save_tx, _) = channel(1);
    let (doc_colors_tx, _) = channel(1);
    let (pull_diag_tx, _) = channel(1);
    let (pull_all_diag_tx, _) = channel(1);

    let handlers = Handlers {
        completions: CompletionHandler::new(completion_tx),
        signature_hints: signature_tx,
        auto_save: auto_save_tx,
        document_colors: doc_colors_tx,
        word_index: word_index::Handler::spawn(),
        pull_diagnostics: pull_diag_tx,
        pull_all_documents_diagnostics: pull_all_diag_tx,
    };

    // Register essential hooks from helix-view, including:
    // - DocumentDidChange -> textDocument/didChange notifications to LSP
    // - DocumentDidClose -> textDocument/didClose notifications to LSP
    // - LanguageServerInitialized -> textDocument/didOpen for all documents
    // Without this, the LSP server won't know about document changes,
    // causing issues like corrupted renames when the server has stale content.
    helix_view::handlers::register_hooks(&handlers);

    handlers
}
