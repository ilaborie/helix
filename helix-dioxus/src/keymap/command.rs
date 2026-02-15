//! Command types for the keymap trie.
//!
//! Defines `CommandSlot` (what lives in trie leaves) and `AwaitCharKind`
//! (for multi-key sequences that need one more character input).

use helix_view::document::Mode;

use crate::state::EditorCommand;

/// What lives at a trie leaf — the action to take when a key sequence is matched.
#[derive(Debug, Clone)]
pub enum CommandSlot {
    /// Execute an editor command immediately.
    Cmd(EditorCommand),
    /// Execute multiple commands in sequence.
    Seq(Vec<EditorCommand>),
    /// Wait for the next character input before resolving.
    AwaitChar(AwaitCharKind),
}

/// The kind of character-awaiting operation.
///
/// Some commands need one (or two) additional character inputs after the
/// initial key. For example, `f` waits for a character to find forward,
/// `r` waits for a replacement character, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AwaitCharKind {
    /// `f` — find character forward on current line.
    FindForward,
    /// `F` — find character backward on current line.
    FindBackward,
    /// `t` — till (before) character forward on current line.
    TillForward,
    /// `T` — till (after) character backward on current line.
    TillBackward,
    /// `r` — replace each character in selection.
    ReplaceChar,
    /// `"` — select register for next yank/paste/delete.
    SelectRegister,
    /// `mi` — select inside a bracket/quote pair.
    SelectInsidePair,
    /// `ma` — select around a bracket/quote pair.
    SelectAroundPair,
    /// `ms` — surround add.
    SurroundAdd,
    /// `md` — surround delete.
    SurroundDelete,
    /// `mr` — surround replace: awaits old char (first of two).
    SurroundReplaceFrom,
    /// `mr<old>` — surround replace: awaits new char (second of two).
    SurroundReplaceTo(char),
    /// `C-r` in insert mode — insert register content.
    InsertRegister,
}

#[must_use]
/// Resolve an `AwaitCharKind` with a character, producing commands.
///
/// Mode-awareness: in select mode, find/till produce extend variants.
pub fn resolve_await(kind: AwaitCharKind, ch: char, mode: Mode) -> Vec<EditorCommand> {
    let is_select = mode == Mode::Select;
    match kind {
        AwaitCharKind::FindForward => {
            if is_select {
                vec![EditorCommand::ExtendFindCharForward(ch)]
            } else {
                vec![EditorCommand::FindCharForward(ch)]
            }
        }
        AwaitCharKind::FindBackward => {
            if is_select {
                vec![EditorCommand::ExtendFindCharBackward(ch)]
            } else {
                vec![EditorCommand::FindCharBackward(ch)]
            }
        }
        AwaitCharKind::TillForward => {
            if is_select {
                vec![EditorCommand::ExtendTillCharForward(ch)]
            } else {
                vec![EditorCommand::TillCharForward(ch)]
            }
        }
        AwaitCharKind::TillBackward => {
            if is_select {
                vec![EditorCommand::ExtendTillCharBackward(ch)]
            } else {
                vec![EditorCommand::TillCharBackward(ch)]
            }
        }
        AwaitCharKind::ReplaceChar => vec![EditorCommand::ReplaceChar(ch)],
        AwaitCharKind::SelectRegister => vec![EditorCommand::SetSelectedRegister(ch)],
        AwaitCharKind::SelectInsidePair => vec![EditorCommand::SelectInsidePair(ch)],
        AwaitCharKind::SelectAroundPair => vec![EditorCommand::SelectAroundPair(ch)],
        AwaitCharKind::SurroundAdd => vec![EditorCommand::SurroundAdd(ch)],
        AwaitCharKind::SurroundDelete => vec![EditorCommand::SurroundDelete(ch)],
        AwaitCharKind::SurroundReplaceFrom => {
            // This doesn't produce commands — it transitions to SurroundReplaceTo.
            // The caller (DhxKeymaps) handles this by updating await_char.
            vec![]
        }
        AwaitCharKind::SurroundReplaceTo(old) => {
            vec![EditorCommand::SurroundReplace(old, ch)]
        }
        AwaitCharKind::InsertRegister => vec![EditorCommand::InsertRegister(ch)],
    }
}

/// Look up a `CommandSlot` by helix-term command name.
///
/// Returns `None` for unknown command names.
#[must_use]
pub fn command_from_name(name: &str) -> Option<CommandSlot> {
    COMMAND_REGISTRY
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, slot)| slot.clone())
}

use crate::state::ShellBehavior;

/// Registry mapping helix-term command names to `CommandSlot` variants.
///
/// Command names match helix-term so users can share `[keys]` configs.
const COMMAND_REGISTRY: &[(&str, CommandSlot)] = &[
    // Movement
    ("move_char_left", CommandSlot::Cmd(EditorCommand::MoveLeft)),
    ("move_char_right", CommandSlot::Cmd(EditorCommand::MoveRight)),
    ("move_visual_line_up", CommandSlot::Cmd(EditorCommand::MoveUp)),
    ("move_visual_line_down", CommandSlot::Cmd(EditorCommand::MoveDown)),
    ("move_line_up", CommandSlot::Cmd(EditorCommand::MoveUp)),
    ("move_line_down", CommandSlot::Cmd(EditorCommand::MoveDown)),
    ("move_next_word_start", CommandSlot::Cmd(EditorCommand::MoveWordForward)),
    (
        "move_prev_word_start",
        CommandSlot::Cmd(EditorCommand::MoveWordBackward),
    ),
    ("move_next_word_end", CommandSlot::Cmd(EditorCommand::MoveWordEnd)),
    (
        "move_next_long_word_start",
        CommandSlot::Cmd(EditorCommand::MoveLongWordForward),
    ),
    (
        "move_prev_long_word_start",
        CommandSlot::Cmd(EditorCommand::MoveLongWordBackward),
    ),
    (
        "move_next_long_word_end",
        CommandSlot::Cmd(EditorCommand::MoveLongWordEnd),
    ),
    ("goto_line_start", CommandSlot::Cmd(EditorCommand::MoveLineStart)),
    ("goto_line_end", CommandSlot::Cmd(EditorCommand::MoveLineEnd)),
    (
        "goto_first_nonwhitespace",
        CommandSlot::Cmd(EditorCommand::GotoFirstNonWhitespace),
    ),
    ("goto_column", CommandSlot::Cmd(EditorCommand::GotoColumn)),
    ("goto_file_start", CommandSlot::Cmd(EditorCommand::GotoFirstLine)),
    ("goto_file_end", CommandSlot::Cmd(EditorCommand::GotoLastLine)),
    ("goto_window_top", CommandSlot::Cmd(EditorCommand::GotoWindowTop)),
    ("goto_window_center", CommandSlot::Cmd(EditorCommand::GotoWindowCenter)),
    ("goto_window_bottom", CommandSlot::Cmd(EditorCommand::GotoWindowBottom)),
    (
        "goto_last_accessed_file",
        CommandSlot::Cmd(EditorCommand::GotoLastAccessedFile),
    ),
    (
        "goto_last_modified_file",
        CommandSlot::Cmd(EditorCommand::GotoLastModifiedFile),
    ),
    (
        "goto_last_modification",
        CommandSlot::Cmd(EditorCommand::GotoLastModification),
    ),
    ("half_page_up", CommandSlot::Cmd(EditorCommand::HalfPageUp)),
    ("half_page_down", CommandSlot::Cmd(EditorCommand::HalfPageDown)),
    ("page_up", CommandSlot::Cmd(EditorCommand::PageUp)),
    ("page_down", CommandSlot::Cmd(EditorCommand::PageDown)),
    ("find_char", CommandSlot::AwaitChar(AwaitCharKind::FindForward)),
    ("find_char_reverse", CommandSlot::AwaitChar(AwaitCharKind::FindBackward)),
    ("find_till_char", CommandSlot::AwaitChar(AwaitCharKind::TillForward)),
    (
        "find_till_char_reverse",
        CommandSlot::AwaitChar(AwaitCharKind::TillBackward),
    ),
    ("repeat_last_motion", CommandSlot::Cmd(EditorCommand::RepeatLastFind)),
    (
        "search_word_under_cursor",
        CommandSlot::Cmd(EditorCommand::SearchWordUnderCursor),
    ),
    ("match_brackets", CommandSlot::Cmd(EditorCommand::MatchBracket)),
    ("align_view_center", CommandSlot::Cmd(EditorCommand::AlignViewCenter)),
    ("align_view_top", CommandSlot::Cmd(EditorCommand::AlignViewTop)),
    ("align_view_bottom", CommandSlot::Cmd(EditorCommand::AlignViewBottom)),
    // Scroll
    ("scroll_up", CommandSlot::Cmd(EditorCommand::ScrollUp(1))),
    ("scroll_down", CommandSlot::Cmd(EditorCommand::ScrollDown(1))),
    // Mode changes
    ("insert_mode", CommandSlot::Cmd(EditorCommand::EnterInsertMode)),
    (
        "insert_at_line_start",
        CommandSlot::Cmd(EditorCommand::EnterInsertModeLineStart),
    ),
    ("append_mode", CommandSlot::Cmd(EditorCommand::EnterInsertModeAfter)),
    (
        "insert_at_line_end",
        CommandSlot::Cmd(EditorCommand::EnterInsertModeLineEnd),
    ),
    ("open_below", CommandSlot::Cmd(EditorCommand::OpenLineBelow)),
    ("open_above", CommandSlot::Cmd(EditorCommand::OpenLineAbove)),
    ("normal_mode", CommandSlot::Cmd(EditorCommand::ExitInsertMode)),
    ("select_mode", CommandSlot::Cmd(EditorCommand::EnterSelectMode)),
    // Editing
    ("change_selection", CommandSlot::Cmd(EditorCommand::ChangeSelection)),
    (
        "change_selection_noyank",
        CommandSlot::Cmd(EditorCommand::ChangeSelectionNoYank),
    ),
    ("replace", CommandSlot::AwaitChar(AwaitCharKind::ReplaceChar)),
    ("join_selections", CommandSlot::Cmd(EditorCommand::JoinLines)),
    ("toggle_comments", CommandSlot::Cmd(EditorCommand::ToggleLineComment)),
    (
        "toggle_block_comments",
        CommandSlot::Cmd(EditorCommand::ToggleBlockComment),
    ),
    ("indent", CommandSlot::Cmd(EditorCommand::IndentLine)),
    ("unindent", CommandSlot::Cmd(EditorCommand::UnindentLine)),
    ("switch_case", CommandSlot::Cmd(EditorCommand::ToggleCase)),
    ("switch_to_lowercase", CommandSlot::Cmd(EditorCommand::ToLowercase)),
    ("switch_to_uppercase", CommandSlot::Cmd(EditorCommand::ToUppercase)),
    ("increment", CommandSlot::Cmd(EditorCommand::Increment)),
    ("decrement", CommandSlot::Cmd(EditorCommand::Decrement)),
    ("align_selections", CommandSlot::Cmd(EditorCommand::AlignSelections)),
    ("add_newline_below", CommandSlot::Cmd(EditorCommand::AddNewlineBelow)),
    ("add_newline_above", CommandSlot::Cmd(EditorCommand::AddNewlineAbove)),
    // History
    ("undo", CommandSlot::Cmd(EditorCommand::Undo)),
    ("redo", CommandSlot::Cmd(EditorCommand::Redo)),
    (
        "commit_undo_checkpoint",
        CommandSlot::Cmd(EditorCommand::CommitUndoCheckpoint),
    ),
    // Selection
    ("select_all", CommandSlot::Cmd(EditorCommand::SelectAll)),
    ("select_line", CommandSlot::Cmd(EditorCommand::SelectLine)),
    ("extend_line", CommandSlot::Cmd(EditorCommand::ExtendLine)),
    (
        "extend_to_line_bounds",
        CommandSlot::Cmd(EditorCommand::ExtendToLineBounds),
    ),
    (
        "shrink_to_line_bounds",
        CommandSlot::Cmd(EditorCommand::ShrinkToLineBounds),
    ),
    ("collapse_selection", CommandSlot::Cmd(EditorCommand::CollapseSelection)),
    ("flip_selections", CommandSlot::Cmd(EditorCommand::FlipSelections)),
    (
        "keep_primary_selection",
        CommandSlot::Cmd(EditorCommand::KeepPrimarySelection),
    ),
    ("trim_selections", CommandSlot::Cmd(EditorCommand::TrimSelections)),
    ("expand_selection", CommandSlot::Cmd(EditorCommand::ExpandSelection)),
    ("shrink_selection", CommandSlot::Cmd(EditorCommand::ShrinkSelection)),
    // Extend movement
    ("extend_char_left", CommandSlot::Cmd(EditorCommand::ExtendLeft)),
    ("extend_char_right", CommandSlot::Cmd(EditorCommand::ExtendRight)),
    ("extend_line_up", CommandSlot::Cmd(EditorCommand::ExtendUp)),
    ("extend_line_down", CommandSlot::Cmd(EditorCommand::ExtendDown)),
    (
        "extend_next_word_start",
        CommandSlot::Cmd(EditorCommand::ExtendWordForward),
    ),
    (
        "extend_prev_word_start",
        CommandSlot::Cmd(EditorCommand::ExtendWordBackward),
    ),
    ("extend_next_word_end", CommandSlot::Cmd(EditorCommand::ExtendWordEnd)),
    (
        "extend_next_long_word_start",
        CommandSlot::Cmd(EditorCommand::ExtendLongWordForward),
    ),
    (
        "extend_prev_long_word_start",
        CommandSlot::Cmd(EditorCommand::ExtendLongWordBackward),
    ),
    (
        "extend_next_long_word_end",
        CommandSlot::Cmd(EditorCommand::ExtendLongWordEnd),
    ),
    ("extend_line_start", CommandSlot::Cmd(EditorCommand::ExtendLineStart)),
    ("extend_line_end", CommandSlot::Cmd(EditorCommand::ExtendLineEnd)),
    (
        "extend_to_first_nonwhitespace",
        CommandSlot::Cmd(EditorCommand::ExtendGotoFirstNonWhitespace),
    ),
    ("extend_goto_column", CommandSlot::Cmd(EditorCommand::ExtendGotoColumn)),
    (
        "extend_to_file_start",
        CommandSlot::Cmd(EditorCommand::ExtendToFirstLine),
    ),
    ("extend_to_file_end", CommandSlot::Cmd(EditorCommand::ExtendToLastLine)),
    ("extend_search_next", CommandSlot::Cmd(EditorCommand::ExtendSearchNext)),
    ("extend_search_prev", CommandSlot::Cmd(EditorCommand::ExtendSearchPrev)),
    // Multi-selection
    (
        "split_selection_on_newline",
        CommandSlot::Cmd(EditorCommand::SplitSelectionOnNewline),
    ),
    (
        "copy_selection_on_next_line",
        CommandSlot::Cmd(EditorCommand::CopySelectionOnNextLine),
    ),
    (
        "copy_selection_on_prev_line",
        CommandSlot::Cmd(EditorCommand::CopySelectionOnPrevLine),
    ),
    (
        "rotate_selections_forward",
        CommandSlot::Cmd(EditorCommand::RotateSelectionsForward),
    ),
    (
        "rotate_selections_backward",
        CommandSlot::Cmd(EditorCommand::RotateSelectionsBackward),
    ),
    // Clipboard
    ("yank", CommandSlot::Cmd(EditorCommand::Yank)),
    (
        "yank_main_selection_to_clipboard",
        CommandSlot::Cmd(EditorCommand::YankMainSelectionToClipboard),
    ),
    ("paste_after", CommandSlot::Cmd(EditorCommand::Paste)),
    ("paste_before", CommandSlot::Cmd(EditorCommand::PasteBefore)),
    ("delete_selection", CommandSlot::Cmd(EditorCommand::DeleteSelection)),
    (
        "delete_selection_noyank",
        CommandSlot::Cmd(EditorCommand::DeleteSelectionNoYank),
    ),
    (
        "replace_with_yanked",
        CommandSlot::Cmd(EditorCommand::ReplaceWithYanked),
    ),
    ("select_register", CommandSlot::AwaitChar(AwaitCharKind::SelectRegister)),
    // Surround
    (
        "select_textobject_inner",
        CommandSlot::AwaitChar(AwaitCharKind::SelectInsidePair),
    ),
    (
        "select_textobject_around",
        CommandSlot::AwaitChar(AwaitCharKind::SelectAroundPair),
    ),
    ("surround_add", CommandSlot::AwaitChar(AwaitCharKind::SurroundAdd)),
    ("surround_delete", CommandSlot::AwaitChar(AwaitCharKind::SurroundDelete)),
    (
        "surround_replace",
        CommandSlot::AwaitChar(AwaitCharKind::SurroundReplaceFrom),
    ),
    // Search
    (
        "search",
        CommandSlot::Cmd(EditorCommand::EnterSearchMode { backwards: false }),
    ),
    (
        "rsearch",
        CommandSlot::Cmd(EditorCommand::EnterSearchMode { backwards: true }),
    ),
    ("search_next", CommandSlot::Cmd(EditorCommand::SearchNext)),
    ("search_prev", CommandSlot::Cmd(EditorCommand::SearchPrevious)),
    // Regex
    (
        "select_regex",
        CommandSlot::Cmd(EditorCommand::EnterRegexMode { split: false }),
    ),
    (
        "split_selection",
        CommandSlot::Cmd(EditorCommand::EnterRegexMode { split: true }),
    ),
    // Command mode
    ("command_mode", CommandSlot::Cmd(EditorCommand::EnterCommandMode)),
    // Pickers
    ("file_picker", CommandSlot::Cmd(EditorCommand::ShowFilePicker)),
    (
        "file_picker_in_current_buffer_directory",
        CommandSlot::Cmd(EditorCommand::ShowFilePickerInBufferDir),
    ),
    ("file_explorer", CommandSlot::Cmd(EditorCommand::ShowFileExplorer)),
    (
        "file_explorer_in_current_buffer_directory",
        CommandSlot::Cmd(EditorCommand::ShowFileExplorerInBufferDir),
    ),
    ("buffer_picker", CommandSlot::Cmd(EditorCommand::ShowBufferPicker)),
    ("symbol_picker", CommandSlot::Cmd(EditorCommand::ShowDocumentSymbols)),
    (
        "workspace_symbol_picker",
        CommandSlot::Cmd(EditorCommand::ShowWorkspaceSymbols),
    ),
    (
        "diagnostics_picker",
        CommandSlot::Cmd(EditorCommand::ShowDocumentDiagnostics),
    ),
    (
        "workspace_diagnostics_picker",
        CommandSlot::Cmd(EditorCommand::ShowWorkspaceDiagnostics),
    ),
    (
        "changed_file_picker",
        CommandSlot::Cmd(EditorCommand::ShowChangedFilesPicker),
    ),
    ("global_search", CommandSlot::Cmd(EditorCommand::ShowGlobalSearch)),
    ("command_palette", CommandSlot::Cmd(EditorCommand::ShowCommandPanel)),
    ("jumplist_picker", CommandSlot::Cmd(EditorCommand::ShowJumpListPicker)),
    ("last_picker", CommandSlot::Cmd(EditorCommand::ShowLastPicker)),
    ("theme_picker", CommandSlot::Cmd(EditorCommand::ShowThemePicker)),
    ("emoji_picker", CommandSlot::Cmd(EditorCommand::ShowEmojiPicker)),
    // Buffer navigation
    ("goto_next_buffer", CommandSlot::Cmd(EditorCommand::NextBuffer)),
    ("goto_previous_buffer", CommandSlot::Cmd(EditorCommand::PreviousBuffer)),
    // LSP
    ("goto_definition", CommandSlot::Cmd(EditorCommand::GotoDefinition)),
    ("goto_declaration", CommandSlot::Cmd(EditorCommand::GotoDeclaration)),
    (
        "goto_type_definition",
        CommandSlot::Cmd(EditorCommand::GotoTypeDefinition),
    ),
    (
        "goto_implementation",
        CommandSlot::Cmd(EditorCommand::GotoImplementation),
    ),
    ("goto_reference", CommandSlot::Cmd(EditorCommand::GotoReferences)),
    ("goto_file", CommandSlot::Cmd(EditorCommand::GotoFileUnderCursor)),
    ("hover", CommandSlot::Cmd(EditorCommand::TriggerHover)),
    ("rename_symbol", CommandSlot::Cmd(EditorCommand::RenameSymbol)),
    ("code_action", CommandSlot::Cmd(EditorCommand::ShowCodeActions)),
    (
        "select_references_to_symbol_under_cursor",
        CommandSlot::Cmd(EditorCommand::SelectReferencesToSymbol),
    ),
    ("format_selections", CommandSlot::Cmd(EditorCommand::FormatSelections)),
    ("toggle_inlay_hints", CommandSlot::Cmd(EditorCommand::ToggleInlayHints)),
    ("completion", CommandSlot::Cmd(EditorCommand::TriggerCompletion)),
    ("signature_help", CommandSlot::Cmd(EditorCommand::TriggerSignatureHelp)),
    // Diagnostics
    ("goto_next_diag", CommandSlot::Cmd(EditorCommand::NextDiagnostic)),
    ("goto_prev_diag", CommandSlot::Cmd(EditorCommand::PrevDiagnostic)),
    ("goto_first_diag", CommandSlot::Cmd(EditorCommand::GotoFirstDiagnostic)),
    ("goto_last_diag", CommandSlot::Cmd(EditorCommand::GotoLastDiagnostic)),
    // Tree-sitter textobject navigation
    ("goto_next_function", CommandSlot::Cmd(EditorCommand::NextFunction)),
    ("goto_prev_function", CommandSlot::Cmd(EditorCommand::PrevFunction)),
    ("goto_next_class", CommandSlot::Cmd(EditorCommand::NextClass)),
    ("goto_prev_class", CommandSlot::Cmd(EditorCommand::PrevClass)),
    ("goto_next_parameter", CommandSlot::Cmd(EditorCommand::NextParameter)),
    ("goto_prev_parameter", CommandSlot::Cmd(EditorCommand::PrevParameter)),
    ("goto_next_comment", CommandSlot::Cmd(EditorCommand::NextComment)),
    ("goto_prev_comment", CommandSlot::Cmd(EditorCommand::PrevComment)),
    ("goto_next_paragraph", CommandSlot::Cmd(EditorCommand::NextParagraph)),
    ("goto_prev_paragraph", CommandSlot::Cmd(EditorCommand::PrevParagraph)),
    // VCS
    ("goto_next_change", CommandSlot::Cmd(EditorCommand::NextChange)),
    ("goto_prev_change", CommandSlot::Cmd(EditorCommand::PrevChange)),
    ("goto_first_change", CommandSlot::Cmd(EditorCommand::GotoFirstChange)),
    ("goto_last_change", CommandSlot::Cmd(EditorCommand::GotoLastChange)),
    // Shell
    (
        "shell_pipe",
        CommandSlot::Cmd(EditorCommand::EnterShellMode(ShellBehavior::Replace)),
    ),
    (
        "shell_pipe_to",
        CommandSlot::Cmd(EditorCommand::EnterShellMode(ShellBehavior::Ignore)),
    ),
    (
        "shell_insert_output",
        CommandSlot::Cmd(EditorCommand::EnterShellMode(ShellBehavior::Insert)),
    ),
    (
        "shell_append_output",
        CommandSlot::Cmd(EditorCommand::EnterShellMode(ShellBehavior::Append)),
    ),
    // Jump list
    ("jump_backward", CommandSlot::Cmd(EditorCommand::JumpBackward)),
    ("jump_forward", CommandSlot::Cmd(EditorCommand::JumpForward)),
    ("save_selection", CommandSlot::Cmd(EditorCommand::SaveSelection)),
    // Word jump (EasyMotion-style)
    ("goto_word", CommandSlot::Cmd(EditorCommand::GotoWord)),
    ("extend_to_word", CommandSlot::Cmd(EditorCommand::ExtendToWord)),
    // Macro
    ("record_macro", CommandSlot::Cmd(EditorCommand::ToggleMacroRecording)),
    ("replay_macro", CommandSlot::Cmd(EditorCommand::ReplayMacro)),
    // Repeat
    ("repeat_last_insert", CommandSlot::Cmd(EditorCommand::RepeatLastInsert)),
    // Insert mode operations
    ("insert_tab", CommandSlot::Cmd(EditorCommand::InsertTab)),
    ("insert_newline", CommandSlot::Cmd(EditorCommand::InsertNewline)),
    (
        "delete_char_backward",
        CommandSlot::Cmd(EditorCommand::DeleteCharBackward),
    ),
    (
        "delete_char_forward",
        CommandSlot::Cmd(EditorCommand::DeleteCharForward),
    ),
    (
        "delete_word_backward",
        CommandSlot::Cmd(EditorCommand::DeleteWordBackward),
    ),
    (
        "delete_word_forward",
        CommandSlot::Cmd(EditorCommand::DeleteWordForward),
    ),
    ("kill_to_line_start", CommandSlot::Cmd(EditorCommand::DeleteToLineStart)),
    ("kill_to_line_end", CommandSlot::Cmd(EditorCommand::KillToLineEnd)),
    ("insert_register", CommandSlot::AwaitChar(AwaitCharKind::InsertRegister)),
    // --- Helix-term compatibility aliases ---
    // These map helix-term command names to our EditorCommand variants so users
    // can share `[keys]` config between helix-term and helix-dioxus.
    //
    // Movement aliases
    ("goto_line_end_newline", CommandSlot::Cmd(EditorCommand::MoveLineEnd)),
    ("page_cursor_up", CommandSlot::Cmd(EditorCommand::PageUp)),
    ("page_cursor_down", CommandSlot::Cmd(EditorCommand::PageDown)),
    ("page_cursor_half_up", CommandSlot::Cmd(EditorCommand::HalfPageUp)),
    ("page_cursor_half_down", CommandSlot::Cmd(EditorCommand::HalfPageDown)),
    // Find/till aliases (helix-term uses different names)
    ("find_next_char", CommandSlot::AwaitChar(AwaitCharKind::FindForward)),
    ("find_prev_char", CommandSlot::AwaitChar(AwaitCharKind::FindBackward)),
    ("till_prev_char", CommandSlot::AwaitChar(AwaitCharKind::TillBackward)),
    // Extend find/till (select-mode aware via AwaitChar)
    ("extend_next_char", CommandSlot::AwaitChar(AwaitCharKind::FindForward)),
    ("extend_prev_char", CommandSlot::AwaitChar(AwaitCharKind::FindBackward)),
    ("extend_till_char", CommandSlot::AwaitChar(AwaitCharKind::TillForward)),
    (
        "extend_till_prev_char",
        CommandSlot::AwaitChar(AwaitCharKind::TillBackward),
    ),
    // Mode aliases
    ("exit_select_mode", CommandSlot::Cmd(EditorCommand::ExitSelectMode)),
    // Comment alias
    (
        "toggle_line_comments",
        CommandSlot::Cmd(EditorCommand::ToggleLineComment),
    ),
    // Extend movement aliases (helix-term uses `_to_` prefix)
    ("extend_to_line_start", CommandSlot::Cmd(EditorCommand::ExtendLineStart)),
    ("extend_to_line_end", CommandSlot::Cmd(EditorCommand::ExtendLineEnd)),
    (
        "extend_to_line_end_newline",
        CommandSlot::Cmd(EditorCommand::ExtendLineEnd),
    ),
    ("extend_to_column", CommandSlot::Cmd(EditorCommand::ExtendGotoColumn)),
    ("extend_visual_line_up", CommandSlot::Cmd(EditorCommand::ExtendUp)),
    ("extend_visual_line_down", CommandSlot::Cmd(EditorCommand::ExtendDown)),
    // History aliases
    ("earlier", CommandSlot::Cmd(EditorCommand::Earlier(1))),
    ("later", CommandSlot::Cmd(EditorCommand::Later(1))),
    // Picker aliases
    (
        "file_picker_in_current_directory",
        CommandSlot::Cmd(EditorCommand::ShowFilePicker),
    ),
    (
        "file_explorer_in_current_directory",
        CommandSlot::Cmd(EditorCommand::ShowFileExplorer),
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_find_forward_normal_mode() {
        let cmds = resolve_await(AwaitCharKind::FindForward, 'x', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::FindCharForward('x')));
    }

    #[test]
    fn resolve_find_forward_select_mode() {
        let cmds = resolve_await(AwaitCharKind::FindForward, 'x', Mode::Select);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendFindCharForward('x')));
    }

    #[test]
    fn resolve_find_backward_normal_mode() {
        let cmds = resolve_await(AwaitCharKind::FindBackward, 'a', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::FindCharBackward('a')));
    }

    #[test]
    fn resolve_find_backward_select_mode() {
        let cmds = resolve_await(AwaitCharKind::FindBackward, 'a', Mode::Select);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendFindCharBackward('a')));
    }

    #[test]
    fn resolve_till_forward_normal_mode() {
        let cmds = resolve_await(AwaitCharKind::TillForward, 'b', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::TillCharForward('b')));
    }

    #[test]
    fn resolve_till_forward_select_mode() {
        let cmds = resolve_await(AwaitCharKind::TillForward, 'b', Mode::Select);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendTillCharForward('b')));
    }

    #[test]
    fn resolve_till_backward_normal_mode() {
        let cmds = resolve_await(AwaitCharKind::TillBackward, 'c', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::TillCharBackward('c')));
    }

    #[test]
    fn resolve_till_backward_select_mode() {
        let cmds = resolve_await(AwaitCharKind::TillBackward, 'c', Mode::Select);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendTillCharBackward('c')));
    }

    #[test]
    fn resolve_replace_char() {
        let cmds = resolve_await(AwaitCharKind::ReplaceChar, 'z', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ReplaceChar('z')));
    }

    #[test]
    fn resolve_select_register() {
        let cmds = resolve_await(AwaitCharKind::SelectRegister, 'a', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SetSelectedRegister('a')));
    }

    #[test]
    fn resolve_select_inside_pair() {
        let cmds = resolve_await(AwaitCharKind::SelectInsidePair, '(', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SelectInsidePair('(')));
    }

    #[test]
    fn resolve_select_around_pair() {
        let cmds = resolve_await(AwaitCharKind::SelectAroundPair, '"', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SelectAroundPair('"')));
    }

    #[test]
    fn resolve_surround_add() {
        let cmds = resolve_await(AwaitCharKind::SurroundAdd, '(', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SurroundAdd('(')));
    }

    #[test]
    fn resolve_surround_delete() {
        let cmds = resolve_await(AwaitCharKind::SurroundDelete, '(', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SurroundDelete('(')));
    }

    #[test]
    fn resolve_surround_replace_from_produces_no_commands() {
        let cmds = resolve_await(AwaitCharKind::SurroundReplaceFrom, '(', Mode::Normal);
        assert!(cmds.is_empty());
    }

    #[test]
    fn resolve_surround_replace_to() {
        let cmds = resolve_await(AwaitCharKind::SurroundReplaceTo('('), '[', Mode::Normal);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SurroundReplace('(', '[')));
    }

    #[test]
    fn resolve_insert_register() {
        let cmds = resolve_await(AwaitCharKind::InsertRegister, '+', Mode::Insert);
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::InsertRegister('+')));
    }

    // --- command_from_name registry tests ---

    #[test]
    fn registry_move_char_left() {
        let slot = command_from_name("move_char_left").expect("should find move_char_left");
        assert!(matches!(slot, CommandSlot::Cmd(EditorCommand::MoveLeft)));
    }

    #[test]
    fn registry_goto_definition() {
        let slot = command_from_name("goto_definition").expect("should find goto_definition");
        assert!(matches!(slot, CommandSlot::Cmd(EditorCommand::GotoDefinition)));
    }

    #[test]
    fn registry_find_char() {
        let slot = command_from_name("find_char").expect("should find find_char");
        assert!(matches!(slot, CommandSlot::AwaitChar(AwaitCharKind::FindForward)));
    }

    #[test]
    fn registry_select_register() {
        let slot = command_from_name("select_register").expect("should find select_register");
        assert!(matches!(slot, CommandSlot::AwaitChar(AwaitCharKind::SelectRegister)));
    }

    #[test]
    fn registry_surround_replace() {
        let slot = command_from_name("surround_replace").expect("should find surround_replace");
        assert!(matches!(
            slot,
            CommandSlot::AwaitChar(AwaitCharKind::SurroundReplaceFrom)
        ));
    }

    #[test]
    fn registry_undo() {
        let slot = command_from_name("undo").expect("should find undo");
        assert!(matches!(slot, CommandSlot::Cmd(EditorCommand::Undo)));
    }

    #[test]
    fn registry_search() {
        let slot = command_from_name("search").expect("should find search");
        assert!(matches!(
            slot,
            CommandSlot::Cmd(EditorCommand::EnterSearchMode { backwards: false })
        ));
    }

    #[test]
    fn registry_shell_pipe() {
        let slot = command_from_name("shell_pipe").expect("should find shell_pipe");
        assert!(matches!(
            slot,
            CommandSlot::Cmd(EditorCommand::EnterShellMode(ShellBehavior::Replace))
        ));
    }

    #[test]
    fn registry_unknown_returns_none() {
        assert!(command_from_name("nonexistent_command").is_none());
    }

    #[test]
    fn registry_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for (name, _) in COMMAND_REGISTRY {
            assert!(seen.insert(*name), "duplicate command name in registry: {name}");
        }
    }
}
