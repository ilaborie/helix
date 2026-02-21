//! Data types for editor state management.
//!
//! This module contains all shared data structures used for state management
//! and communication between the editor core and UI components.

use std::path::{Path, PathBuf};

use helix_view::DocumentId;

use crate::config::DialogSearchMode;
use crate::lsp::{
    CodeActionPreviewState, CodeActionSnapshot, CompletionItemSnapshot, DiagnosticSeverity, DiagnosticSnapshot,
    InlayHintSnapshot, LocationSnapshot, LspServerSnapshot, SignatureHelpSnapshot,
};

/// Compute a visible window of `window_size` items centered on `selected`,
/// clamped to `[0, total)`. Returns `(start, end)`.
#[must_use]
pub fn centered_window(selected: usize, total: usize, window_size: usize) -> (usize, usize) {
    let half = window_size / 2;
    let start = if selected <= half {
        0
    } else if selected + half >= total {
        total.saturating_sub(window_size)
    } else {
        selected - half
    };
    let end = (start + window_size).min(total);
    (start, end)
}

/// Determines what action to take on startup.
#[derive(Debug, Clone)]
pub enum StartupAction {
    /// No argument provided - open scratch buffer.
    None,
    /// Single file to open.
    OpenFile(PathBuf),
    /// Multiple files to open (from glob pattern or multiple args).
    OpenFiles(Vec<PathBuf>),
    /// Directory argument - open file picker in that directory.
    OpenFilePicker,
}

/// Buffer info for the tab bar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BufferInfo {
    pub id: DocumentId,
    pub name: String,
    pub is_modified: bool,
    pub is_current: bool,
}

/// Type of diff change for a line in the gutter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineType {
    /// Line was added (new content).
    Added,
    /// Line was modified (changed content).
    Modified,
    /// Line marks a deletion point (content was removed here).
    Deleted,
}

impl DiffLineType {
    /// CSS color variable for this diff type.
    #[must_use]
    pub fn css_color(&self) -> &'static str {
        match self {
            Self::Added => "var(--success)",
            Self::Modified => "var(--accent)",
            Self::Deleted => "var(--error)",
        }
    }
}

/// Icon type for picker items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickerIcon {
    #[default]
    File,
    Folder,
    FolderOpen,
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
    // Diagnostic icons
    DiagnosticError,
    DiagnosticWarning,
    DiagnosticInfo,
    DiagnosticHint,
    // Search result icon
    SearchResult,
    // Location icons
    Reference,
    Definition,
    // Register icon
    Register,
    // Command panel icon
    Command,
    // Jump list icon
    JumpEntry,
    // Theme icon
    Theme,
    // Emoji icon
    Emoji,
    // VCS icons
    VcsAdded,
    VcsModified,
    VcsConflict,
    VcsDeleted,
    VcsRenamed,
}

/// Generic picker item with match highlighting.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PickerItem {
    pub id: String,
    pub display: String,
    pub icon: PickerIcon,
    pub match_indices: Vec<usize>,
    pub secondary: Option<String>,
    /// Nesting depth for tree-style pickers (0 = top level).
    pub depth: u16,
}

/// Picker mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PickerMode {
    #[default]
    DirectoryBrowser,
    FileExplorer,
    FilesRecursive,
    Buffers,
    DocumentSymbols,
    WorkspaceSymbols,
    DocumentDiagnostics,
    WorkspaceDiagnostics,
    GlobalSearch,
    References,
    Definitions,
    Registers,
    Commands,
    JumpList,
    Themes,
    ChangedFiles,
    Emojis,
}

impl PickerMode {
    /// Human-readable title for the picker header.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::DirectoryBrowser => "Open File",
            Self::FileExplorer => "File Explorer",
            Self::FilesRecursive => "Find Files",
            Self::Buffers => "Switch Buffer",
            Self::DocumentSymbols => "Document Symbols",
            Self::WorkspaceSymbols => "Workspace Symbols",
            Self::DocumentDiagnostics => "Document Diagnostics",
            Self::WorkspaceDiagnostics => "Workspace Diagnostics",
            Self::GlobalSearch => "Global Search",
            Self::References => "References",
            Self::Definitions => "Definitions",
            Self::Registers => "Registers",
            Self::Commands => "Commands",
            Self::JumpList => "Jump List",
            Self::Themes => "Themes",
            Self::ChangedFiles => "Changed Files",
            Self::Emojis => "Emojis",
        }
    }

    /// Hint text for the Enter key action in the help row.
    #[must_use]
    pub fn enter_hint(&self) -> &'static str {
        match self {
            Self::DirectoryBrowser => " open/enter \u{2022} ",
            Self::FileExplorer => " open/toggle \u{2022} ",
            Self::GlobalSearch => " search/open \u{2022} ",
            _ => " select \u{2022} ",
        }
    }

    /// Whether this picker mode supports file preview.
    #[must_use]
    pub fn supports_preview(&self) -> bool {
        !matches!(self, Self::Registers | Self::Commands | Self::Themes | Self::Emojis)
    }
}

/// A single line in the picker preview panel.
#[derive(Debug, Clone, PartialEq)]
pub struct PreviewLine {
    /// Line number (1-indexed).
    pub line_number: usize,
    /// Line text content (no trailing newline).
    pub content: String,
    /// Syntax highlight spans for this line.
    pub tokens: Vec<TokenSpan>,
    /// Whether this is the target/focus line.
    pub is_focus_line: bool,
}

/// Image file extensions supported for preview.
const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "avif", "tiff", "tif", "heic",
    "heif", "apng", "jfif",
];

/// Check if a path refers to an image file based on its extension.
#[must_use]
pub fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| IMAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

/// Content variants for picker preview.
#[derive(Debug, Clone, PartialEq)]
pub enum PreviewContent {
    /// Syntax-highlighted text preview.
    Text {
        lines: Vec<PreviewLine>,
        focus_line: Option<usize>,
        search_pattern: Option<String>,
    },
    /// Image preview with metadata.
    Image {
        absolute_path: String,
        file_size: u64,
        dimensions: Option<(usize, usize)>,
        format: String,
    },
}

/// File preview data for the picker panel.
#[derive(Debug, Clone, PartialEq)]
pub struct PickerPreview {
    /// Display path (relative if possible).
    pub file_path: String,
    /// Preview content (text or image).
    pub content: PreviewContent,
}

/// Minimal diagnostic info for scrollbar markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollbarDiagnostic {
    /// Line number (0-indexed).
    pub line: usize,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// Diagnostic message (truncated for display).
    pub message: String,
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
    /// Number of selections (cursors) in the current document.
    pub selection_count: usize,
    pub lines: Vec<LineSnapshot>,

    // UI state
    pub command_mode: bool,
    pub command_input: String,
    /// Filtered command completions for the current command input.
    pub command_completions: Vec<CommandCompletionItem>,
    /// Selected index in the command completion list.
    pub command_completion_selected: usize,
    pub search_mode: bool,
    pub search_backwards: bool,
    pub search_input: String,
    pub regex_mode: bool,
    pub regex_split: bool,
    pub regex_input: String,
    /// Line numbers with search matches (for scrollbar markers).
    pub search_match_lines: Vec<usize>,
    /// Lines with jump list entries (1-indexed, for gutter markers).
    pub jump_lines: Vec<usize>,
    /// Lines with VCS diff changes (1-indexed line number, diff type).
    pub diff_lines: Vec<(usize, DiffLineType)>,

    // Picker state
    pub picker_visible: bool,
    /// Pre-windowed picker items (only the visible ~15 items, not all filtered items).
    pub picker_items: Vec<PickerItem>,
    pub picker_filter: String,
    pub picker_selected: usize,
    /// Total number of unfiltered source items.
    pub picker_total: usize,
    /// Number of items after filtering (may differ from `picker_items.len()` due to windowing).
    pub picker_filtered_count: usize,
    /// Start index of the windowed items in the full filtered list.
    pub picker_window_offset: usize,
    pub picker_mode: PickerMode,
    pub picker_current_path: Option<String>,
    /// File preview for the selected picker item.
    pub picker_preview: Option<PickerPreview>,

    // Buffer bar state
    pub open_buffers: Vec<BufferInfo>,
    pub buffer_scroll_offset: usize,

    // LSP state
    /// Diagnostics for the current document (visible lines only).
    pub diagnostics: Vec<DiagnosticSnapshot>,
    /// All diagnostics summary for scrollbar markers (line + severity only).
    pub all_diagnostics_summary: Vec<ScrollbarDiagnostic>,
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
    /// Pre-rendered hover HTML with syntax-highlighted code blocks.
    pub hover_html: Option<String>,
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
    /// Preview of the currently selected code action.
    pub code_action_preview: Option<CodeActionPreviewState>,
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

    // Shell mode state
    /// Whether the shell prompt is active.
    pub shell_mode: bool,
    /// Current shell input text.
    pub shell_input: String,
    /// Shell prompt prefix (e.g., "pipe:", "insert-output:").
    pub shell_prompt: String,

    // Word jump state
    /// Whether word jump labels are active.
    pub word_jump_active: bool,
    /// Word jump labels to render over the editor content.
    pub word_jump_labels: Vec<WordJumpLabel>,
    /// First character already typed (None = waiting for first char).
    pub word_jump_first_char: Option<char>,

    // Register state
    /// Register snapshots for display in the help bar.
    pub registers: Vec<RegisterSnapshot>,
    /// Currently selected register for the next operation (e.g., `"a`).
    pub selected_register: Option<char>,
    /// Register currently being recorded to (None if not recording).
    pub macro_recording: Option<char>,

    // Theme state
    /// Current theme name.
    pub current_theme: String,
    /// CSS variable overrides generated from the current theme.
    pub theme_css_vars: String,

    // Dialog configuration
    /// Dialog search mode (direct or vim-style).
    pub dialog_search_mode: DialogSearchMode,
    /// Whether the picker search input is focused (vim-style mode only).
    pub picker_search_focused: bool,

    // Application state
    pub should_quit: bool,
}

/// Snapshot of a single line for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct LineSnapshot {
    pub line_number: usize,
    pub content: String,
    pub is_cursor_line: bool,
    /// All cursor column positions on this line (multiple cursors from multi-selection).
    pub cursor_cols: Vec<usize>,
    /// The primary cursor column on this line (used for `id="editor-cursor"` and scrollIntoView).
    pub primary_cursor_col: Option<usize>,
    pub tokens: Vec<TokenSpan>,
    /// Selection ranges within this line (`start_col`, `end_col`) - for visual mode highlighting.
    /// Each range [start, end) should be highlighted as selected.
    pub selection_ranges: Vec<(usize, usize)>,
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
    /// Scroll to make a specific line visible (0-indexed).
    ScrollToLine(usize),
    /// Move cursor to a specific line (0-indexed) and scroll view.
    GoToLine(usize),
    /// Find character forward on current line.
    FindCharForward(char),
    /// Find character backward on current line.
    FindCharBackward(char),
    /// Move till (before) character forward on current line.
    TillCharForward(char),
    /// Move till (after) character backward on current line.
    TillCharBackward(char),
    /// Move to end of word.
    MoveWordEnd,
    /// Move to next long word (WORD) start.
    MoveLongWordForward,
    /// Move to end of long word (WORD).
    MoveLongWordEnd,
    /// Move to previous long word (WORD) start.
    MoveLongWordBackward,
    /// Repeat last find/till motion.
    RepeatLastFind,
    /// Search for the word under the cursor.
    SearchWordUnderCursor,
    /// Jump to matching bracket.
    MatchBracket,
    /// Scroll half page up and move cursor.
    HalfPageUp,
    /// Scroll half page down and move cursor.
    HalfPageDown,
    /// Move to first non-whitespace character on line (gs).
    GotoFirstNonWhitespace,
    /// Move cursor to column 1 on current line (g|).
    GotoColumn,
    /// Align view so cursor is centered vertically (zz / zc).
    AlignViewCenter,
    /// Align view so cursor is at the top (zt).
    AlignViewTop,
    /// Align view so cursor is at the bottom (zb).
    AlignViewBottom,
    /// Move cursor to top of visible window (gt).
    GotoWindowTop,
    /// Move cursor to center of visible window (gc).
    GotoWindowCenter,
    /// Move cursor to bottom of visible window (gb).
    GotoWindowBottom,
    /// Jump to last accessed file (ga).
    GotoLastAccessedFile,
    /// Jump to last modified file (gm).
    GotoLastModifiedFile,
    /// Jump to last modification position in current document (g.).
    GotoLastModification,
    /// Jump to first diagnostic ([D).
    GotoFirstDiagnostic,
    /// Jump to last diagnostic (]D).
    GotoLastDiagnostic,
    /// Move to next paragraph (]p).
    NextParagraph,
    /// Move to previous paragraph ([p).
    PrevParagraph,
    /// Move to next function (]f).
    NextFunction,
    /// Move to previous function ([f).
    PrevFunction,
    /// Move to next class/type (]t).
    NextClass,
    /// Move to previous class/type ([t).
    PrevClass,
    /// Move to next parameter (]a).
    NextParameter,
    /// Move to previous parameter ([a).
    PrevParameter,
    /// Move to next comment (]c).
    NextComment,
    /// Move to previous comment ([c).
    PrevComment,
    /// Extend selection to full line bounds (X).
    ExtendToLineBounds,
    /// Shrink selection to line bounds (Alt-x).
    ShrinkToLineBounds,
    /// Expand selection to parent syntax node (Alt-o).
    ExpandSelection,
    /// Shrink selection to child syntax node (Alt-i).
    ShrinkSelection,

    // Mode changes
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeLineEnd,
    /// Insert at first non-whitespace of line.
    EnterInsertModeLineStart,
    ExitInsertMode,
    EnterSelectMode,
    ExitSelectMode,

    // Editing
    InsertChar(char),
    InsertTab,
    InsertNewline,
    DeleteCharBackward,
    DeleteCharForward,
    /// Delete word backward (Ctrl+w in insert mode).
    DeleteWordBackward,
    /// Delete to line start (Ctrl+u in insert mode).
    DeleteToLineStart,
    /// Delete word forward (Alt+d in insert mode).
    DeleteWordForward,
    /// Kill to line end (Ctrl+k in insert mode).
    KillToLineEnd,
    OpenLineBelow,
    OpenLineAbove,
    /// Indent the current line or selection.
    IndentLine,
    /// Unindent the current line or selection.
    UnindentLine,
    /// Change selection: delete + enter insert mode.
    ChangeSelection,
    /// Change selection without yanking (Alt-c): delete + enter insert mode, skip register.
    ChangeSelectionNoYank,
    /// Replace each character in selection with the given character.
    ReplaceChar(char),
    /// Join lines in selection.
    JoinLines,
    /// Toggle case of characters in selection.
    ToggleCase,
    /// Convert selection to lowercase.
    ToLowercase,
    /// Convert selection to uppercase.
    ToUppercase,
    /// Add a newline below current line without entering insert mode.
    AddNewlineBelow,
    /// Add a newline above current line without entering insert mode.
    AddNewlineAbove,
    /// Increment number or date under cursor (C-a).
    Increment,
    /// Decrement number or date under cursor (C-x).
    Decrement,
    /// Align selections by inserting spaces (&).
    AlignSelections,
    /// Add surround pair around selection.
    SurroundAdd(char),
    /// Delete surround pair.
    SurroundDelete(char),
    /// Replace surround pair (old, new).
    SurroundReplace(char, char),

    // History
    Undo,
    Redo,
    /// Commit an undo checkpoint (C-s in insert mode).
    CommitUndoCheckpoint,
    /// Insert register content at cursor (C-r then register char in insert mode).
    InsertRegister(char),

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
    /// Extend selection to word end.
    ExtendWordEnd,
    /// Extend selection to next long word start.
    ExtendLongWordForward,
    /// Extend selection to long word end.
    ExtendLongWordEnd,
    /// Extend selection to previous long word start.
    ExtendLongWordBackward,
    ExtendLineStart,
    ExtendLineEnd,
    SelectLine,
    ExtendLine,
    /// Collapse selection to cursor position (;).
    CollapseSelection,
    /// Keep only primary selection, remove others (,).
    KeepPrimarySelection,
    /// Select inside a bracket/quote pair.
    SelectInsidePair(char),
    /// Select around a bracket/quote pair.
    SelectAroundPair(char),
    /// Trim whitespace from selection edges (_).
    TrimSelections,
    /// Select entire buffer (%).
    SelectAll,
    /// Flip selection (swap anchor and head) (Alt+;).
    FlipSelections,
    /// Extend selection to find char forward (select mode).
    ExtendFindCharForward(char),
    /// Extend selection to find char backward (select mode).
    ExtendFindCharBackward(char),
    /// Extend selection till char forward (select mode).
    ExtendTillCharForward(char),
    /// Extend selection till char backward (select mode).
    ExtendTillCharBackward(char),
    /// Extend selection to first line (select mode gg).
    ExtendToFirstLine,
    /// Extend selection to last line (select mode ge/G).
    ExtendToLastLine,
    /// Extend selection to first non-whitespace on line (select mode gs).
    ExtendGotoFirstNonWhitespace,
    /// Extend selection to column 1 (select mode g|).
    ExtendGotoColumn,
    /// Extend selection to next search match (select mode).
    ExtendSearchNext,
    /// Extend selection to previous search match (select mode).
    ExtendSearchPrev,

    // Multi-selection operations
    /// Split selection on newlines (A-s).
    SplitSelectionOnNewline,
    /// Copy selection to next line (C).
    CopySelectionOnNextLine,
    /// Copy selection to previous line (A-C).
    CopySelectionOnPrevLine,
    /// Rotate selections forward ()).
    RotateSelectionsForward,
    /// Rotate selections backward (().
    RotateSelectionsBackward,

    // Clipboard operations
    Yank,
    /// Yank only the primary selection to the clipboard (Space Y).
    YankMainSelectionToClipboard,
    Paste,
    PasteBefore,

    // Delete
    DeleteSelection,
    /// Delete selection without yanking (Alt-d).
    DeleteSelectionNoYank,
    /// Replace selection with yanked text (R).
    ReplaceWithYanked,

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

    // Regex select/split mode
    /// Enter regex prompt mode (s = select, S = split).
    EnterRegexMode {
        split: bool,
    },
    ExitRegexMode,
    RegexInput(char),
    RegexBackspace,
    RegexExecute,

    // Command mode
    EnterCommandMode,
    ExitCommandMode,
    CommandInput(char),
    CommandBackspace,
    CommandExecute,
    /// Execute a typable command directly (from `[keys]` config, e.g. `:write`).
    TypeableCommand(String),
    /// Navigate up in command completion list.
    CommandCompletionUp,
    /// Navigate down in command completion list.
    CommandCompletionDown,
    /// Accept selected command completion (Tab).
    CommandCompletionAccept,

    // File picker
    ShowFilePicker,
    ShowFilesRecursivePicker,
    /// Show file explorer at CWD (Space e).
    ShowFileExplorer,
    /// Show file explorer in buffer's directory (Space E).
    ShowFileExplorerInBufferDir,
    /// Expand the selected directory in the file explorer.
    ExplorerExpand,
    /// Collapse the selected directory or navigate to parent in the file explorer.
    ExplorerCollapseOrParent,
    ShowBufferPicker,
    /// Resume last picker (Space ').
    ShowLastPicker,
    PickerUp,
    PickerDown,
    PickerConfirm,
    PickerCancel,
    PickerInput(char),
    PickerBackspace,
    /// Click on a picker item to select and confirm it.
    PickerConfirmItem(usize),
    /// Jump to first picker item.
    PickerFirst,
    /// Jump to last picker item.
    PickerLast,
    /// Page up in picker (jump by 10 items).
    PickerPageUp,
    /// Page down in picker (jump by 10 items).
    PickerPageDown,

    // Buffer navigation
    BufferBarScrollLeft,
    BufferBarScrollRight,
    SwitchToBuffer(DocumentId),
    CloseBuffer(DocumentId),
    NextBuffer,
    PreviousBuffer,

    // File operations
    /// Open the file path under the cursor (gf).
    GotoFileUnderCursor,
    /// Show file picker in the current buffer's directory (Space F).
    ShowFilePickerInBufferDir,
    OpenFile(PathBuf),
    /// Save document to a specific path (used by Save As dialog).
    SaveDocumentToPath(PathBuf),

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
    /// Go to declaration of symbol under cursor.
    GotoDeclaration,
    /// Find references to symbol under cursor.
    GotoReferences,
    /// Go to type definition of symbol under cursor.
    GotoTypeDefinition,
    /// Go to implementation of symbol under cursor.
    GotoImplementation,
    /// Select all references to symbol under cursor (document highlights).
    SelectReferencesToSymbol,
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

    // VCS - Hunk navigation
    /// Jump to next change hunk (]g).
    NextChange,
    /// Jump to previous change hunk ([g).
    PrevChange,
    /// Jump to first change hunk ([G).
    GotoFirstChange,
    /// Jump to last change hunk (]G).
    GotoLastChange,
    /// Show changed files picker (Space g).
    ShowChangedFilesPicker,

    // LSP - Diagnostics
    /// Jump to next diagnostic.
    NextDiagnostic,
    /// Jump to previous diagnostic.
    PrevDiagnostic,
    /// Show document diagnostics picker.
    ShowDocumentDiagnostics,
    /// Show workspace diagnostics picker.
    ShowWorkspaceDiagnostics,

    // LSP - Format
    /// Format the current document.
    FormatDocument,
    /// Format selections via LSP range formatting (=).
    FormatSelections,

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

    // Global Search
    /// Show the global search picker.
    ShowGlobalSearch,
    /// Execute global search with current filter.
    GlobalSearchExecute,
    /// Receive batch of global search results.
    GlobalSearchResults(Vec<GlobalSearchResult>),
    /// Global search completed.
    GlobalSearchComplete,

    // Buffer management - additional commands
    /// Reload the current document from disk.
    ReloadDocument,
    /// Save all modified buffers.
    WriteAll,
    /// Quit all buffers and exit.
    QuitAll {
        force: bool,
    },
    /// Close all buffers.
    BufferCloseAll {
        force: bool,
    },
    /// Close all buffers except the current one.
    BufferCloseOthers,

    // Directory commands
    /// Change the current working directory.
    ChangeDirectory(PathBuf),
    /// Print the current working directory.
    PrintWorkingDirectory,

    // History navigation
    /// Undo to an earlier state (multiple steps).
    Earlier(usize),
    /// Redo to a later state (multiple steps).
    Later(usize),

    // Register management
    /// Set the selected register for the next yank/paste/delete operation.
    SetSelectedRegister(char),
    /// Clear a register by name ('+' = clipboard, '/' = search).
    ClearRegister(char),

    // Command panel
    /// Show the command panel (fuzzy command palette).
    ShowCommandPanel,

    // Theme
    /// Set the editor theme by name.
    SetTheme(String),
    /// Show the theme picker.
    ShowThemePicker,

    // Jump list
    /// Jump backward through position history (C-o).
    JumpBackward,
    /// Jump forward through position history (C-i).
    JumpForward,
    /// Save current position to the jump list (C-s).
    SaveSelection,
    /// Show the jump list picker (Space j).
    ShowJumpListPicker,

    // Shell integration
    /// Enter shell prompt mode with the given behavior.
    EnterShellMode(ShellBehavior),
    /// Exit shell prompt mode.
    ExitShellMode,
    /// Add a character to the shell input.
    ShellInput(char),
    /// Remove the last character from the shell input.
    ShellBackspace,
    /// Execute the shell command.
    ShellExecute,

    // Word jump (EasyMotion-style)
    /// Initiate word jump from normal mode (gw).
    GotoWord,
    /// Initiate word jump from select mode (gw) — extends selection.
    ExtendToWord,
    /// First character typed during word jump label matching.
    WordJumpFirstChar(char),
    /// Second character typed during word jump label matching.
    WordJumpSecondChar(char),
    /// Cancel word jump.
    CancelWordJump,

    // Macro recording/replay
    /// Toggle macro recording (Q). Starts/stops recording to a register.
    ToggleMacroRecording,
    /// Replay macro from register (q).
    ReplayMacro,

    // Picker search focus (vim-style dialog mode)
    /// Focus picker search input (/ in vim-style mode).
    PickerFocusSearch,
    /// Unfocus picker search input (Esc in vim-style mode when search is focused).
    PickerUnfocusSearch,

    // Emoji picker
    /// Show the emoji picker.
    ShowEmojiPicker,
    /// Insert a multi-character text string at cursor (used by emoji picker).
    InsertText(String),

    // CLI passthrough
    /// Execute a CLI command by string (e.g., ":sort", ":reflow").
    CliCommand(String),

    /// Repeat the last insert mode session (dot command).
    RepeatLastInsert,
}

impl EditorCommand {
    /// Whether this command should be recorded for dot-repeat during insert mode.
    pub(crate) fn is_insert_recordable(&self) -> bool {
        matches!(
            self,
            Self::InsertChar(_)
                | Self::InsertTab
                | Self::InsertNewline
                | Self::InsertText(_)
                | Self::InsertRegister(_)
                | Self::DeleteCharBackward
                | Self::DeleteCharForward
                | Self::DeleteWordBackward
                | Self::DeleteWordForward
                | Self::DeleteToLineStart
                | Self::KillToLineEnd
                | Self::IndentLine
                | Self::UnindentLine
                | Self::CommitUndoCheckpoint
                | Self::ToggleLineComment
        )
    }
}

/// Shell pipe behavior for the `|`, `!`, `A-|`, `A-!` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellBehavior {
    /// Pipe selection through command, replace with stdout (`|`).
    Replace,
    /// Insert command stdout before selection (`!`).
    Insert,
    /// Pipe selection through command, discard stdout (`A-|`).
    Ignore,
    /// Append command stdout after selection (`A-!`).
    Append,
}

/// A label for word jump (EasyMotion-style) overlay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordJumpLabel {
    /// Line number (1-indexed, matching `LineSnapshot`).
    pub line: usize,
    /// Column offset (0-indexed character position in the line).
    pub col: usize,
    /// Two-character label for this word.
    pub label: [char; 2],
    /// Whether this label is dimmed (doesn't match first char typed so far).
    pub dimmed: bool,
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

/// Tracks pending key sequence state for multi-key commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PendingKeySequence {
    #[default]
    None,
    /// Waiting for second key after 'g'
    GPrefix,
    /// Waiting for second key after ']'
    BracketNext,
    /// Waiting for second key after '['
    BracketPrev,
    /// Waiting for second key after Space
    SpaceLeader,
    /// Waiting for character after 'f' (find forward)
    FindForward,
    /// Waiting for character after 'F' (find backward)
    FindBackward,
    /// Waiting for character after 't' (till forward)
    TillForward,
    /// Waiting for character after 'T' (till backward)
    TillBackward,
    /// Waiting for character after 'r' (replace char)
    ReplacePrefix,
    /// Waiting for register name after '"'
    RegisterPrefix,
    /// Waiting for second key after 'm' (match/surround)
    MatchPrefix,
    /// Waiting for character after 'mi' (select inside pair)
    MatchInside,
    /// Waiting for character after 'ma' (select around pair)
    MatchAround,
    /// Waiting for character after 'ms' (surround add)
    MatchSurround,
    /// Waiting for character after 'md' (surround delete)
    MatchDeleteSurround,
    /// Waiting for first character after 'mr' (surround replace: old char)
    MatchReplaceSurroundFrom,
    /// Waiting for second character after `mr<old>` (surround replace: new char)
    MatchReplaceSurroundTo(char),
    /// Waiting for sub-key after 'z' (one-shot view mode)
    ViewPrefix,
    /// Waiting for sub-key after 'Z' (sticky view mode — stays until Esc)
    ViewPrefixSticky,
    /// Waiting for register char after C-r in insert mode.
    InsertRegisterPrefix,
    /// Waiting for first label character after word jump labels appear.
    WordJumpFirstChar,
    /// Waiting for second label character during word jump.
    WordJumpSecondChar,
}

/// A single register's state for display in the help bar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RegisterSnapshot {
    /// Register name character (e.g., '+', '*', '/').
    pub name: char,
    /// Register content (empty if register has no value).
    pub content: String,
}

/// A single command completion item for the command mode popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCompletionItem {
    pub name: String,
    pub description: String,
    pub match_indices: Vec<usize>,
}

/// A single global search result for the picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSearchResult {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// Line number (1-indexed, matching editor convention).
    pub line_num: usize,
    /// The matching line content (trimmed).
    pub line_content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- centered_window ---

    #[test]
    fn centered_window_at_start() {
        assert_eq!(centered_window(0, 100, 15), (0, 15));
    }

    #[test]
    fn centered_window_at_end() {
        assert_eq!(centered_window(99, 100, 15), (85, 100));
    }

    #[test]
    fn centered_window_centered() {
        assert_eq!(centered_window(50, 100, 15), (43, 58));
    }

    #[test]
    fn centered_window_small_list() {
        assert_eq!(centered_window(2, 5, 15), (0, 5));
    }

    #[test]
    fn centered_window_exact_fit() {
        assert_eq!(centered_window(7, 15, 15), (0, 15));
    }

    #[test]
    fn centered_window_empty() {
        assert_eq!(centered_window(0, 0, 15), (0, 0));
    }

    // --- DiffLineType ---

    #[test]
    fn diff_line_type_colors() {
        assert_eq!(DiffLineType::Added.css_color(), "var(--success)");
        assert_eq!(DiffLineType::Modified.css_color(), "var(--accent)");
        assert_eq!(DiffLineType::Deleted.css_color(), "var(--error)");
    }

    // --- PickerMode ---

    #[test]
    fn changed_files_title() {
        assert_eq!(PickerMode::ChangedFiles.title(), "Changed Files");
    }

    #[test]
    fn changed_files_supports_preview() {
        assert!(PickerMode::ChangedFiles.supports_preview());
    }

    // --- PickerMode::title ---

    #[test]
    fn picker_mode_title_all_variants() {
        let modes = [
            PickerMode::DirectoryBrowser,
            PickerMode::FileExplorer,
            PickerMode::FilesRecursive,
            PickerMode::Buffers,
            PickerMode::DocumentSymbols,
            PickerMode::WorkspaceSymbols,
            PickerMode::DocumentDiagnostics,
            PickerMode::WorkspaceDiagnostics,
            PickerMode::GlobalSearch,
            PickerMode::References,
            PickerMode::Definitions,
            PickerMode::Registers,
            PickerMode::Commands,
            PickerMode::JumpList,
            PickerMode::Themes,
            PickerMode::ChangedFiles,
            PickerMode::Emojis,
        ];
        for mode in modes {
            let title = mode.title();
            assert!(!title.is_empty(), "{mode:?} should have a non-empty title");
        }
    }

    // --- PickerMode::enter_hint ---

    #[test]
    fn enter_hint_directory_browser() {
        assert!(PickerMode::DirectoryBrowser.enter_hint().contains("open/enter"));
    }

    #[test]
    fn enter_hint_global_search() {
        assert!(PickerMode::GlobalSearch.enter_hint().contains("search/open"));
    }

    #[test]
    fn enter_hint_default() {
        assert!(PickerMode::Buffers.enter_hint().contains("select"));
        assert!(PickerMode::Commands.enter_hint().contains("select"));
    }

    // --- PickerMode::supports_preview ---

    #[test]
    fn supports_preview_file_modes() {
        let file_modes = [
            PickerMode::DirectoryBrowser,
            PickerMode::FileExplorer,
            PickerMode::FilesRecursive,
            PickerMode::Buffers,
            PickerMode::DocumentSymbols,
            PickerMode::WorkspaceSymbols,
            PickerMode::DocumentDiagnostics,
            PickerMode::WorkspaceDiagnostics,
            PickerMode::GlobalSearch,
            PickerMode::References,
            PickerMode::Definitions,
            PickerMode::JumpList,
            PickerMode::ChangedFiles,
        ];
        for mode in file_modes {
            assert!(mode.supports_preview(), "{mode:?} should support preview");
        }
    }

    #[test]
    fn supports_preview_non_file_modes() {
        assert!(
            !PickerMode::Registers.supports_preview(),
            "Registers should not support preview"
        );
        assert!(
            !PickerMode::Commands.supports_preview(),
            "Commands should not support preview"
        );
        assert!(
            !PickerMode::Themes.supports_preview(),
            "Themes should not support preview"
        );
        assert!(
            !PickerMode::Emojis.supports_preview(),
            "Emojis should not support preview"
        );
    }

    #[test]
    fn file_explorer_title() {
        assert_eq!(PickerMode::FileExplorer.title(), "File Explorer");
    }

    #[test]
    fn file_explorer_enter_hint() {
        assert!(PickerMode::FileExplorer.enter_hint().contains("open/toggle"));
    }

    #[test]
    fn file_explorer_supports_preview() {
        assert!(PickerMode::FileExplorer.supports_preview());
    }

    #[test]
    fn picker_item_default_depth_is_zero() {
        let item = PickerItem::default();
        assert_eq!(item.depth, 0);
    }

    #[test]
    fn themes_picker_mode_title() {
        assert_eq!(PickerMode::Themes.title(), "Themes");
    }

    #[test]
    fn themes_picker_mode_enter_hint() {
        assert!(PickerMode::Themes.enter_hint().contains("select"));
    }

    // --- EditorCommand::is_insert_recordable ---

    #[test]
    fn insert_char_is_recordable() {
        assert!(EditorCommand::InsertChar('a').is_insert_recordable());
    }

    #[test]
    fn insert_tab_is_recordable() {
        assert!(EditorCommand::InsertTab.is_insert_recordable());
    }

    #[test]
    fn insert_newline_is_recordable() {
        assert!(EditorCommand::InsertNewline.is_insert_recordable());
    }

    #[test]
    fn insert_text_is_recordable() {
        assert!(EditorCommand::InsertText("hi".into()).is_insert_recordable());
    }

    #[test]
    fn delete_char_backward_is_recordable() {
        assert!(EditorCommand::DeleteCharBackward.is_insert_recordable());
    }

    #[test]
    fn commit_undo_checkpoint_is_recordable() {
        assert!(EditorCommand::CommitUndoCheckpoint.is_insert_recordable());
    }

    #[test]
    fn exit_insert_mode_is_not_recordable() {
        assert!(!EditorCommand::ExitInsertMode.is_insert_recordable());
    }

    #[test]
    fn completion_up_is_not_recordable() {
        assert!(!EditorCommand::CompletionUp.is_insert_recordable());
    }

    #[test]
    fn move_left_is_not_recordable() {
        assert!(!EditorCommand::MoveLeft.is_insert_recordable());
    }

    // --- is_image_file ---

    #[test]
    fn is_image_file_common_formats() {
        assert!(is_image_file(Path::new("photo.png")));
        assert!(is_image_file(Path::new("photo.jpg")));
        assert!(is_image_file(Path::new("photo.jpeg")));
        assert!(is_image_file(Path::new("photo.gif")));
        assert!(is_image_file(Path::new("photo.webp")));
        assert!(is_image_file(Path::new("photo.svg")));
        assert!(is_image_file(Path::new("photo.bmp")));
        assert!(is_image_file(Path::new("photo.ico")));
    }

    #[test]
    fn is_image_file_case_insensitive() {
        assert!(is_image_file(Path::new("photo.PNG")));
        assert!(is_image_file(Path::new("photo.Jpg")));
        assert!(is_image_file(Path::new("photo.WEBP")));
    }

    #[test]
    fn is_image_file_non_image_returns_false() {
        assert!(!is_image_file(Path::new("file.rs")));
        assert!(!is_image_file(Path::new("file.txt")));
        assert!(!is_image_file(Path::new("file.toml")));
        assert!(!is_image_file(Path::new("Makefile")));
    }

    #[test]
    fn is_image_file_no_extension_returns_false() {
        assert!(!is_image_file(Path::new("README")));
    }
}
