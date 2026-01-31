//! Data types for editor state management.
//!
//! This module contains all shared data structures used for state management
//! and communication between the editor core and UI components.

use std::path::PathBuf;

use helix_view::DocumentId;

use crate::lsp::{
    CodeActionSnapshot, CompletionItemSnapshot, DiagnosticSnapshot, HoverSnapshot,
    InlayHintSnapshot, LocationSnapshot, SignatureHelpSnapshot,
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
    /// Whether location picker is visible.
    pub location_picker_visible: bool,
    /// Locations to display in picker.
    pub locations: Vec<LocationSnapshot>,
    /// Selected location index.
    pub location_selected: usize,
    /// Location picker title.
    pub location_picker_title: String,

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

    // Mode changes
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeLineEnd,
    ExitInsertMode,
    EnterSelectMode,
    ExitSelectMode,

    // Editing
    InsertChar(char),
    InsertNewline,
    DeleteCharBackward,
    DeleteCharForward,
    OpenLineBelow,
    OpenLineAbove,

    // History
    Undo,
    Redo,

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

    // LSP - Signature Help
    /// Trigger signature help (usually auto-triggered on `(`).
    TriggerSignatureHelp,
    /// Close signature help popup.
    CloseSignatureHelp,

    // LSP - Internal responses (from async LSP operations)
    /// Handle LSP response (internal).
    LspResponse(crate::lsp::LspResponse),
}

/// Direction for cursor movement.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
