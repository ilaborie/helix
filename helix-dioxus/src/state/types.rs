//! Data types for editor state management.
//!
//! This module contains all shared data structures used for state management
//! and communication between the editor core and UI components.

use std::path::PathBuf;

use helix_view::DocumentId;

use crate::lsp::{
    CodeActionSnapshot, CompletionItemSnapshot, DiagnosticSnapshot, HoverSnapshot,
    InlayHintSnapshot, LocationSnapshot, LspServerSnapshot, SignatureHelpSnapshot,
};

/// Buffer info for the tab bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferInfo {
    pub id: DocumentId,
    pub name: String,
    pub is_modified: bool,
    pub is_current: bool,
}

/// Icon type for picker items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickerIcon {
    #[default]
    File,
    Folder,
    Buffer,
    BufferModified,
    // Symbol icons
    SymbolFunction,
    SymbolMethod,
    SymbolClass,
    SymbolStruct,
    SymbolEnum,
    SymbolInterface,
    SymbolVariable,
    SymbolConstant,
    SymbolField,
    SymbolModule,
    SymbolOther,
}

/// Generic picker item with match highlighting.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PickerItem {
    pub id: String,
    pub display: String,
    pub icon: PickerIcon,
    pub match_indices: Vec<usize>,
    pub secondary: Option<String>,
}

/// Picker mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickerMode {
    #[default]
    DirectoryBrowser,
    FilesRecursive,
    Buffers,
    DocumentSymbols,
    WorkspaceSymbols,
}

/// A snapshot of the editor state for rendering.
/// This is Clone + Send + Sync so it can be used with Dioxus.
#[derive(Debug, Clone, Default)]
pub struct EditorSnapshot {
    /// Version counter that increments on each snapshot creation.
    /// Used for change detection to avoid unnecessary re-renders.
    pub snapshot_version: u64,
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
    pub search_mode: bool,
    pub search_backwards: bool,
    pub search_input: String,

    // Picker state
    pub picker_visible: bool,
    pub picker_items: Vec<PickerItem>,
    pub picker_filter: String,
    pub picker_selected: usize,
    pub picker_total: usize,
    pub picker_mode: PickerMode,
    pub picker_current_path: Option<String>,

    // Buffer bar state
    pub open_buffers: Vec<BufferInfo>,
    pub buffer_scroll_offset: usize,

    // LSP state
    /// Diagnostics for the current document, grouped by line.
    pub diagnostics: Vec<DiagnosticSnapshot>,
    /// Total error count in the current document.
    pub error_count: usize,
    /// Total warning count in the current document.
    pub warning_count: usize,
    /// Whether the completion popup is visible.
    pub completion_visible: bool,
    /// Completion items to display.
    pub completion_items: Vec<CompletionItemSnapshot>,
    /// Selected completion item index.
    pub completion_selected: usize,
    /// Whether the hover popup is visible.
    pub hover_visible: bool,
    /// Hover content to display.
    pub hover_content: Option<HoverSnapshot>,
    /// Whether signature help is visible.
    pub signature_help_visible: bool,
    /// Signature help content.
    pub signature_help: Option<SignatureHelpSnapshot>,
    /// Inlay hints for the visible lines.
    pub inlay_hints: Vec<InlayHintSnapshot>,
    /// Whether inlay hints are enabled.
    pub inlay_hints_enabled: bool,
    /// Whether code actions menu is visible.
    pub code_actions_visible: bool,
    /// Available code actions.
    pub code_actions: Vec<CodeActionSnapshot>,
    /// Selected code action index.
    pub code_action_selected: usize,
    /// Code action filter string for searching.
    pub code_action_filter: String,
    /// Whether code actions are available at cursor position (for lightbulb indicator).
    pub has_code_actions: bool,
    /// Whether location picker is visible.
    pub location_picker_visible: bool,
    /// Locations to display in picker.
    pub locations: Vec<LocationSnapshot>,
    /// Selected location index.
    pub location_selected: usize,
    /// Location picker title.
    pub location_picker_title: String,

    // LSP Dialog state
    /// Whether the LSP dialog is visible.
    pub lsp_dialog_visible: bool,
    /// List of language servers for display.
    pub lsp_servers: Vec<LspServerSnapshot>,
    /// Selected server index in dialog.
    pub lsp_server_selected: usize,

    // Notification state
    /// Active notifications to display.
    pub notifications: Vec<NotificationSnapshot>,

    // Input dialog state
    /// Whether the input dialog is visible.
    pub input_dialog_visible: bool,
    /// Input dialog content.
    pub input_dialog: InputDialogSnapshot,

    // Confirmation dialog state
    /// Whether the confirmation dialog is visible.
    pub confirmation_dialog_visible: bool,
    /// Confirmation dialog content.
    pub confirmation_dialog: ConfirmationDialogSnapshot,

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
    /// Selection range within this line (start_col, end_col) - for visual mode highlighting.
    /// If Some, the range [start, end) should be highlighted as selected.
    pub selection_range: Option<(usize, usize)>,
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
    PageUp,
    PageDown,
    ScrollUp(usize),
    ScrollDown(usize),

    // Mode changes
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeLineEnd,
    ExitInsertMode,
    EnterSelectMode,
    ExitSelectMode,

    // Editing
    InsertChar(char),
    InsertTab,
    InsertNewline,
    DeleteCharBackward,
    DeleteCharForward,
    OpenLineBelow,
    OpenLineAbove,

    // History
    Undo,
    Redo,

    // Comments
    ToggleLineComment,
    ToggleBlockComment,

    // Selection
    ExtendLeft,
    ExtendRight,
    ExtendUp,
    ExtendDown,
    ExtendWordForward,
    ExtendWordBackward,
    ExtendLineStart,
    ExtendLineEnd,
    SelectLine,
    ExtendLine,

    // Clipboard operations
    Yank,
    Paste,
    PasteBefore,

    // Delete
    DeleteSelection,

    // Search
    EnterSearchMode {
        backwards: bool,
    },
    ExitSearchMode,
    SearchInput(char),
    SearchBackspace,
    SearchExecute,
    SearchNext,
    SearchPrevious,

    // Command mode
    EnterCommandMode,
    ExitCommandMode,
    CommandInput(char),
    CommandBackspace,
    CommandExecute,

    // File picker
    ShowFilePicker,
    ShowFilesRecursivePicker,
    ShowBufferPicker,
    PickerUp,
    PickerDown,
    PickerConfirm,
    PickerCancel,
    PickerInput(char),
    PickerBackspace,

    // Buffer navigation
    BufferBarScrollLeft,
    BufferBarScrollRight,
    SwitchToBuffer(DocumentId),
    CloseBuffer(DocumentId),
    NextBuffer,
    PreviousBuffer,

    // File operations
    OpenFile(PathBuf),

    // LSP - Completion
    /// Trigger completion popup manually.
    TriggerCompletion,
    /// Move selection up in completion menu.
    CompletionUp,
    /// Move selection down in completion menu.
    CompletionDown,
    /// Confirm selected completion item.
    CompletionConfirm,
    /// Cancel/close completion popup.
    CompletionCancel,

    // LSP - Hover
    /// Show hover information at cursor.
    TriggerHover,
    /// Close hover popup.
    CloseHover,

    // LSP - Goto
    /// Go to definition of symbol under cursor.
    GotoDefinition,
    /// Find references to symbol under cursor.
    GotoReferences,
    /// Go to type definition of symbol under cursor.
    GotoTypeDefinition,
    /// Go to implementation of symbol under cursor.
    GotoImplementation,
    /// Confirm selected location in location picker.
    LocationConfirm,
    /// Cancel/close location picker.
    LocationCancel,
    /// Move selection up in location picker.
    LocationUp,
    /// Move selection down in location picker.
    LocationDown,

    // LSP - Code Actions
    /// Show code actions at cursor.
    ShowCodeActions,
    /// Confirm selected code action.
    CodeActionConfirm,
    /// Cancel/close code actions menu.
    CodeActionCancel,
    /// Move selection up in code actions menu.
    CodeActionUp,
    /// Move selection down in code actions menu.
    CodeActionDown,
    /// Add a character to the code action filter.
    CodeActionFilterChar(char),
    /// Remove a character from the code action filter.
    CodeActionFilterBackspace,

    // LSP - Diagnostics
    /// Jump to next diagnostic.
    NextDiagnostic,
    /// Jump to previous diagnostic.
    PrevDiagnostic,

    // LSP - Format
    /// Format the current document.
    FormatDocument,

    // LSP - Rename
    /// Rename symbol under cursor.
    RenameSymbol,

    // LSP - Inlay Hints
    /// Toggle inlay hints display.
    ToggleInlayHints,
    /// Refresh inlay hints from LSP.
    RefreshInlayHints,

    // LSP - Symbol Picker
    /// Show document symbols picker.
    ShowDocumentSymbols,
    /// Show workspace symbols picker.
    ShowWorkspaceSymbols,

    // LSP - Signature Help
    /// Trigger signature help (usually auto-triggered on `(`).
    TriggerSignatureHelp,
    /// Close signature help popup.
    CloseSignatureHelp,

    // LSP - Internal responses (from async LSP operations)
    /// Handle LSP response (internal).
    LspResponse(crate::lsp::LspResponse),

    // LSP Dialog
    /// Toggle the LSP status dialog.
    ToggleLspDialog,
    /// Close the LSP status dialog.
    CloseLspDialog,
    /// Move selection up in LSP dialog.
    LspDialogUp,
    /// Move selection down in LSP dialog.
    LspDialogDown,
    /// Restart the currently selected LSP server.
    RestartSelectedLsp,
    /// Restart a specific LSP server by name.
    RestartLspServer(String),

    // Notifications
    /// Show a notification toast.
    ShowNotification {
        message: String,
        severity: NotificationSeverity,
    },
    /// Dismiss a specific notification by ID.
    DismissNotification(u64),
    /// Dismiss all notifications.
    DismissAllNotifications,

    // Input Dialog
    /// Show an input dialog.
    ShowInputDialog {
        title: String,
        prompt: String,
        placeholder: Option<String>,
        prefill: Option<String>,
        kind: InputDialogKind,
    },
    /// Add a character to the input dialog.
    InputDialogInput(char),
    /// Remove a character from the input dialog.
    InputDialogBackspace,
    /// Confirm the input dialog.
    InputDialogConfirm,
    /// Cancel the input dialog.
    InputDialogCancel,

    // Confirmation Dialog
    /// Show a confirmation dialog.
    ShowConfirmationDialog(ConfirmationDialogSnapshot),
    /// User confirmed (pressed y or clicked confirm button).
    ConfirmationDialogConfirm,
    /// User denied (pressed n or clicked deny button).
    ConfirmationDialogDeny,
    /// User cancelled (pressed Esc or clicked cancel button).
    ConfirmationDialogCancel,
}

/// Direction for cursor movement.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Notification severity levels for toast notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationSeverity {
    /// Error - file errors, LSP failures (red)
    Error,
    /// Warning - unsaved changes warnings (yellow)
    #[default]
    Warning,
    /// Info - general information (blue)
    Info,
    /// Success - operation completed (green)
    Success,
}

/// Snapshot of a notification for rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationSnapshot {
    /// Unique identifier for this notification.
    pub id: u64,
    /// The notification message.
    pub message: String,
    /// Severity level determines color styling.
    pub severity: NotificationSeverity,
    /// Timestamp when the notification was created (for auto-dismiss).
    pub created_at: u64,
}

/// Snapshot of an input dialog for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InputDialogSnapshot {
    /// Dialog title (e.g., "Rename Symbol").
    pub title: String,
    /// Input prompt text (e.g., "New name:").
    pub prompt: String,
    /// Current input value.
    pub value: String,
    /// Placeholder text when value is empty.
    pub placeholder: Option<String>,
}

/// The type of pending input dialog operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputDialogKind {
    /// No pending operation.
    #[default]
    None,
    /// Rename symbol operation.
    RenameSymbol,
}

/// The action type for confirmation dialogs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfirmationAction {
    /// No pending action.
    #[default]
    None,
    /// Save and quit the editor.
    SaveAndQuit,
    /// Quit without saving.
    QuitWithoutSave,
    /// Close a buffer with unsaved changes.
    CloseBuffer,
    /// Reload file from disk, discarding changes.
    ReloadFile,
}

/// Snapshot of a confirmation dialog for rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConfirmationDialogSnapshot {
    /// Dialog title (e.g., "Unsaved Changes").
    pub title: String,
    /// Message explaining what the user needs to confirm.
    pub message: String,
    /// Label for the confirm button (e.g., "Save & Quit").
    pub confirm_label: String,
    /// Label for the deny button (e.g., "Don't Save"). If None, no deny button is shown.
    pub deny_label: Option<String>,
    /// Label for the cancel button (e.g., "Cancel").
    pub cancel_label: String,
    /// The action to perform when confirmed.
    pub action: ConfirmationAction,
}
