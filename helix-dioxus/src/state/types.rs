//! Data types for editor state management.
//!
//! This module contains all shared data structures used for state management
//! and communication between the editor core and UI components.

use std::path::PathBuf;

use helix_view::DocumentId;

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
    EnterSearchMode { backwards: bool },
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
}

/// Direction for cursor movement.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}
