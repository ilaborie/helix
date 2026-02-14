//! Normal mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use super::handle_move_keys;
use crate::state::{EditorCommand, ShellBehavior};

/// Handle view mode sub-keys (after `z` or `Z` prefix).
///
/// Returns commands for alignment, scrolling, and search operations.
/// Returns an empty vec for unrecognized keys (or Esc).
pub fn handle_view_prefix(key: &KeyEvent) -> Vec<EditorCommand> {
    // Handle Ctrl+key combinations
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('b') => vec![EditorCommand::PageUp],
            KeyCode::Char('f') => vec![EditorCommand::PageDown],
            KeyCode::Char('u') => vec![EditorCommand::HalfPageUp],
            KeyCode::Char('d') => vec![EditorCommand::HalfPageDown],
            _ => vec![],
        };
    }

    match key.code {
        // Alignment
        KeyCode::Char('z') | KeyCode::Char('c') | KeyCode::Char('m') => {
            vec![EditorCommand::AlignViewCenter]
        }
        KeyCode::Char('t') => vec![EditorCommand::AlignViewTop],
        KeyCode::Char('b') => vec![EditorCommand::AlignViewBottom],

        // Scroll by line
        KeyCode::Char('k') | KeyCode::Up => vec![EditorCommand::ScrollUp(1)],
        KeyCode::Char('j') | KeyCode::Down => vec![EditorCommand::ScrollDown(1)],

        // Page scroll
        KeyCode::PageUp => vec![EditorCommand::PageUp],
        KeyCode::PageDown => vec![EditorCommand::PageDown],
        KeyCode::Backspace => vec![EditorCommand::HalfPageUp],
        KeyCode::Char(' ') => vec![EditorCommand::HalfPageDown],

        // Search
        KeyCode::Char('/') => vec![EditorCommand::EnterSearchMode { backwards: false }],
        KeyCode::Char('?') => vec![EditorCommand::EnterSearchMode { backwards: true }],
        KeyCode::Char('n') => vec![EditorCommand::SearchNext],
        KeyCode::Char('N') => vec![EditorCommand::SearchPrevious],

        _ => vec![],
    }
}

/// Handle keyboard input in Normal mode.
pub fn handle_normal_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Handle Alt+key combinations
    if key.modifiers.contains(KeyModifiers::ALT) {
        return match key.code {
            // Alt+. = repeat last motion (find/till)
            KeyCode::Char('.') => vec![EditorCommand::RepeatLastFind],
            // Alt+; = flip selections (swap anchor and head)
            KeyCode::Char(';') => vec![EditorCommand::FlipSelections],
            // Alt+` = convert to uppercase
            KeyCode::Char('`') => vec![EditorCommand::ToUppercase],
            // Alt+c = change selection without yanking
            KeyCode::Char('c') => vec![EditorCommand::ChangeSelectionNoYank],
            // Alt+C = copy selection to previous line
            KeyCode::Char('C') => vec![EditorCommand::CopySelectionOnPrevLine],
            // Alt+d = delete selection without yanking
            KeyCode::Char('d') => vec![EditorCommand::DeleteSelectionNoYank],
            // Alt+s = split selection on newlines
            KeyCode::Char('s') => vec![EditorCommand::SplitSelectionOnNewline],
            // Alt+x = shrink selection to line bounds
            KeyCode::Char('x') => vec![EditorCommand::ShrinkToLineBounds],
            // Alt+o = expand selection to parent syntax node
            KeyCode::Char('o') => vec![EditorCommand::ExpandSelection],
            // Alt+i = shrink selection to child syntax node
            KeyCode::Char('i') => vec![EditorCommand::ShrinkSelection],
            // Alt+| = pipe selection to shell, discard output
            KeyCode::Char('|') => {
                vec![EditorCommand::EnterShellMode(ShellBehavior::Ignore)]
            }
            // Alt+! = append shell output after selection
            KeyCode::Char('!') => {
                vec![EditorCommand::EnterShellMode(ShellBehavior::Append)]
            }
            _ => vec![],
        };
    }

    // Handle Ctrl+key combinations
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('b') => vec![EditorCommand::PageUp],
            KeyCode::Char('c') => vec![EditorCommand::ToggleLineComment],
            KeyCode::Char('d') => vec![EditorCommand::HalfPageDown],
            KeyCode::Char('f') => vec![EditorCommand::PageDown],
            KeyCode::Char('h') => vec![EditorCommand::PreviousBuffer],
            KeyCode::Char('l') => vec![EditorCommand::NextBuffer],
            KeyCode::Char('a') => vec![EditorCommand::Increment],
            KeyCode::Char('i') => vec![EditorCommand::JumpForward],
            KeyCode::Char('o') => vec![EditorCommand::JumpBackward],
            KeyCode::Char('r') => vec![EditorCommand::Redo],
            KeyCode::Char('s') => vec![EditorCommand::SaveSelection],
            KeyCode::Char('u') => vec![EditorCommand::HalfPageUp],
            KeyCode::Char('x') => vec![EditorCommand::Decrement],
            // Ctrl+Space or Ctrl+. - show code actions (quick fix)
            KeyCode::Char(' ') | KeyCode::Char('.') => vec![EditorCommand::ShowCodeActions],
            _ => vec![],
        };
    }

    // Direction keys (hjkl + arrows)
    if let Some(cmds) = handle_move_keys(key.code) {
        return cmds;
    }

    match key.code {
        // Word movement
        KeyCode::Char('w') => vec![EditorCommand::MoveWordForward],
        KeyCode::Char('b') => vec![EditorCommand::MoveWordBackward],
        KeyCode::Char('e') => vec![EditorCommand::MoveWordEnd],

        // WORD movement (long words)
        KeyCode::Char('W') => vec![EditorCommand::MoveLongWordForward],
        KeyCode::Char('B') => vec![EditorCommand::MoveLongWordBackward],
        KeyCode::Char('E') => vec![EditorCommand::MoveLongWordEnd],

        // Line movement
        KeyCode::Char('0') | KeyCode::Home => vec![EditorCommand::MoveLineStart],
        KeyCode::Char('$') | KeyCode::End => vec![EditorCommand::MoveLineEnd],

        // File navigation
        KeyCode::Char('G') => vec![EditorCommand::GotoLastLine],
        KeyCode::PageUp => vec![EditorCommand::PageUp],
        KeyCode::PageDown => vec![EditorCommand::PageDown],

        // Mode changes
        KeyCode::Char('i') => vec![EditorCommand::EnterInsertMode],
        KeyCode::Char('I') => vec![EditorCommand::EnterInsertModeLineStart],
        KeyCode::Char('a') => vec![EditorCommand::EnterInsertModeAfter],
        KeyCode::Char('A') => vec![EditorCommand::EnterInsertModeLineEnd],
        KeyCode::Char('o') => vec![EditorCommand::OpenLineBelow],
        KeyCode::Char('O') => vec![EditorCommand::OpenLineAbove],

        // Change selection (delete + enter insert)
        KeyCode::Char('c') => vec![EditorCommand::ChangeSelection],

        // Regex select/split
        KeyCode::Char('s') => vec![EditorCommand::EnterRegexMode { split: false }],
        KeyCode::Char('S') => vec![EditorCommand::EnterRegexMode { split: true }],

        // Copy selection on next line
        KeyCode::Char('C') => vec![EditorCommand::CopySelectionOnNextLine],

        // Rotate selections
        KeyCode::Char(')') => vec![EditorCommand::RotateSelectionsForward],
        KeyCode::Char('(') => vec![EditorCommand::RotateSelectionsBackward],

        // History
        KeyCode::Char('u') => vec![EditorCommand::Undo],
        KeyCode::Char('U') => vec![EditorCommand::Redo], // Shift+U also redoes (helix convention)

        // Visual selection mode
        KeyCode::Char('v') => vec![EditorCommand::EnterSelectMode],

        // Line selection (helix x/X)
        KeyCode::Char('x') => vec![EditorCommand::SelectLine],
        KeyCode::Char('X') => vec![EditorCommand::ExtendToLineBounds],

        // Delete selection (works in normal mode due to selection-first model)
        KeyCode::Char('d') => vec![EditorCommand::DeleteSelection],

        // Replace with yanked text
        KeyCode::Char('R') => vec![EditorCommand::ReplaceWithYanked],

        // Clipboard
        KeyCode::Char('p') => vec![EditorCommand::Paste],
        KeyCode::Char('P') => vec![EditorCommand::PasteBefore],
        KeyCode::Char('y') => vec![EditorCommand::Yank],

        // Search
        KeyCode::Char('/') => vec![EditorCommand::EnterSearchMode { backwards: false }],
        KeyCode::Char('?') => vec![EditorCommand::EnterSearchMode { backwards: true }],
        KeyCode::Char('n') => vec![EditorCommand::SearchNext],
        KeyCode::Char('N') => vec![EditorCommand::SearchPrevious],

        // Command mode
        KeyCode::Char(':') => vec![EditorCommand::EnterCommandMode],

        // Selection operations
        KeyCode::Char(';') => vec![EditorCommand::CollapseSelection],
        KeyCode::Char(',') => vec![EditorCommand::KeepPrimarySelection],

        // Indent/unindent
        KeyCode::Char('>') => vec![EditorCommand::IndentLine],
        KeyCode::Char('<') => vec![EditorCommand::UnindentLine],

        // Search word under cursor
        KeyCode::Char('*') => vec![EditorCommand::SearchWordUnderCursor],

        // Select all
        KeyCode::Char('%') => vec![EditorCommand::SelectAll],

        // Join lines
        KeyCode::Char('J') => vec![EditorCommand::JoinLines],

        // Align selections
        KeyCode::Char('&') => vec![EditorCommand::AlignSelections],

        // Trim selections
        KeyCode::Char('_') => vec![EditorCommand::TrimSelections],

        // Format selections via LSP
        KeyCode::Char('=') => vec![EditorCommand::FormatSelections],

        // Case operations
        KeyCode::Char('~') => vec![EditorCommand::ToggleCase],
        KeyCode::Char('`') => vec![EditorCommand::ToLowercase],

        // LSP - Hover (K for documentation like vim)
        KeyCode::Char('K') => vec![EditorCommand::TriggerHover],

        // Shell integration
        KeyCode::Char('|') => vec![EditorCommand::EnterShellMode(ShellBehavior::Replace)],
        KeyCode::Char('!') => vec![EditorCommand::EnterShellMode(ShellBehavior::Insert)],

        _ => vec![],
    }
}

/// Handle two-key sequences starting with 'g' (goto).
pub fn handle_g_prefix(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // g. - go to last modification in current document
        KeyCode::Char('.') => vec![EditorCommand::GotoLastModification],
        // ga - go to last accessed file (alternate file)
        KeyCode::Char('a') => vec![EditorCommand::GotoLastAccessedFile],
        // gb - go to window bottom
        KeyCode::Char('b') => vec![EditorCommand::GotoWindowBottom],
        // gc - go to window center
        KeyCode::Char('c') => vec![EditorCommand::GotoWindowCenter],
        // gD - go to declaration
        KeyCode::Char('D') => vec![EditorCommand::GotoDeclaration],
        // gd - go to definition
        KeyCode::Char('d') => vec![EditorCommand::GotoDefinition],
        // ge - go to last line
        KeyCode::Char('e') => vec![EditorCommand::GotoLastLine],
        // gf - open file under cursor
        KeyCode::Char('f') => vec![EditorCommand::GotoFileUnderCursor],
        // gg - go to first line
        KeyCode::Char('g') => vec![EditorCommand::GotoFirstLine],
        // gh - go to line start
        KeyCode::Char('h') => vec![EditorCommand::MoveLineStart],
        // gi - go to implementation
        KeyCode::Char('i') => vec![EditorCommand::GotoImplementation],
        // gj - move down (visual line, no soft-wrap = same as j)
        KeyCode::Char('j') => vec![EditorCommand::MoveDown],
        // gk - move up (visual line, no soft-wrap = same as k)
        KeyCode::Char('k') => vec![EditorCommand::MoveUp],
        // gl - go to line end
        KeyCode::Char('l') => vec![EditorCommand::MoveLineEnd],
        // gm - go to last modified file
        KeyCode::Char('m') => vec![EditorCommand::GotoLastModifiedFile],
        // gn - next buffer
        KeyCode::Char('n') => vec![EditorCommand::NextBuffer],
        // gp - previous buffer
        KeyCode::Char('p') => vec![EditorCommand::PreviousBuffer],
        // gr - go to references
        KeyCode::Char('r') => vec![EditorCommand::GotoReferences],
        // gs - go to first non-whitespace character on line
        KeyCode::Char('s') => vec![EditorCommand::GotoFirstNonWhitespace],
        // gt - go to window top
        KeyCode::Char('t') => vec![EditorCommand::GotoWindowTop],
        // gw - word jump (EasyMotion-style)
        KeyCode::Char('w') => vec![EditorCommand::GotoWord],
        // gy - go to type definition
        KeyCode::Char('y') => vec![EditorCommand::GotoTypeDefinition],
        // g| - go to column 1
        KeyCode::Char('|') => vec![EditorCommand::GotoColumn],
        _ => vec![],
    }
}

/// Handle two-key sequences starting with ']' (next).
pub fn handle_bracket_next(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // ]a - next parameter
        KeyCode::Char('a') => vec![EditorCommand::NextParameter],
        // ]c - next comment
        KeyCode::Char('c') => vec![EditorCommand::NextComment],
        // ]d - next diagnostic
        KeyCode::Char('d') => vec![EditorCommand::NextDiagnostic],
        // ]f - next function
        KeyCode::Char('f') => vec![EditorCommand::NextFunction],
        // ]p - next paragraph
        KeyCode::Char('p') => vec![EditorCommand::NextParagraph],
        // ]t - next class/type
        KeyCode::Char('t') => vec![EditorCommand::NextClass],
        // ]D - last diagnostic
        KeyCode::Char('D') => vec![EditorCommand::GotoLastDiagnostic],
        // ] Space - add newline below
        KeyCode::Char(' ') => vec![EditorCommand::AddNewlineBelow],
        _ => vec![],
    }
}

/// Handle two-key sequences starting with '[' (previous).
pub fn handle_bracket_prev(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // [a - previous parameter
        KeyCode::Char('a') => vec![EditorCommand::PrevParameter],
        // [c - previous comment
        KeyCode::Char('c') => vec![EditorCommand::PrevComment],
        // [d - previous diagnostic
        KeyCode::Char('d') => vec![EditorCommand::PrevDiagnostic],
        // [f - previous function
        KeyCode::Char('f') => vec![EditorCommand::PrevFunction],
        // [p - previous paragraph
        KeyCode::Char('p') => vec![EditorCommand::PrevParagraph],
        // [t - previous class/type
        KeyCode::Char('t') => vec![EditorCommand::PrevClass],
        // [D - first diagnostic
        KeyCode::Char('D') => vec![EditorCommand::GotoFirstDiagnostic],
        // [ Space - add newline above
        KeyCode::Char(' ') => vec![EditorCommand::AddNewlineAbove],
        _ => vec![],
    }
}

/// Handle two-key sequences starting with Space (leader).
pub fn handle_space_leader(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // Space / - global search
        KeyCode::Char('/') => vec![EditorCommand::ShowGlobalSearch],
        // Space ? - command palette
        KeyCode::Char('?') => vec![EditorCommand::ShowCommandPanel],
        // Space a - show code actions
        KeyCode::Char('a') => vec![EditorCommand::ShowCodeActions],
        // Space c - toggle line comment
        KeyCode::Char('c') => vec![EditorCommand::ToggleLineComment],
        // Space C - toggle block comment
        KeyCode::Char('C') => vec![EditorCommand::ToggleBlockComment],
        // Space b - buffer picker
        KeyCode::Char('b') => vec![EditorCommand::ShowBufferPicker],
        // Space d - document diagnostics
        KeyCode::Char('d') => vec![EditorCommand::ShowDocumentDiagnostics],
        // Space D - workspace diagnostics
        KeyCode::Char('D') => vec![EditorCommand::ShowWorkspaceDiagnostics],
        // Space f - file picker
        KeyCode::Char('f') => vec![EditorCommand::ShowFilePicker],
        // Space F - file picker in buffer's directory
        KeyCode::Char('F') => vec![EditorCommand::ShowFilePickerInBufferDir],
        // Space h - select references to symbol (document highlights)
        KeyCode::Char('h') => vec![EditorCommand::SelectReferencesToSymbol],
        // Space i - toggle inlay hints (custom extension)
        KeyCode::Char('i') => vec![EditorCommand::ToggleInlayHints],
        // Space j - jump list picker
        KeyCode::Char('j') => vec![EditorCommand::ShowJumpListPicker],
        // Space k - hover
        KeyCode::Char('k') => vec![EditorCommand::TriggerHover],
        // Space p - paste from system clipboard
        KeyCode::Char('p') => {
            vec![
                EditorCommand::SetSelectedRegister('+'),
                EditorCommand::Paste,
            ]
        }
        // Space P - paste from system clipboard before
        KeyCode::Char('P') => {
            vec![
                EditorCommand::SetSelectedRegister('+'),
                EditorCommand::PasteBefore,
            ]
        }
        // Space r - rename symbol
        KeyCode::Char('r') => vec![EditorCommand::RenameSymbol],
        // Space R - replace selections with clipboard
        KeyCode::Char('R') => {
            vec![
                EditorCommand::SetSelectedRegister('+'),
                EditorCommand::ReplaceWithYanked,
            ]
        }
        // Space s - document symbols
        KeyCode::Char('s') => vec![EditorCommand::ShowDocumentSymbols],
        // Space S - workspace symbols
        KeyCode::Char('S') => vec![EditorCommand::ShowWorkspaceSymbols],
        // Space y - yank to system clipboard
        KeyCode::Char('y') => {
            vec![EditorCommand::SetSelectedRegister('+'), EditorCommand::Yank]
        }
        // Space Y - yank main selection to system clipboard
        KeyCode::Char('Y') => {
            vec![
                EditorCommand::SetSelectedRegister('+'),
                EditorCommand::YankMainSelectionToClipboard,
            ]
        }
        // Space ' - resume last picker
        KeyCode::Char('\'') => vec![EditorCommand::ShowLastPicker],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_single_command;
    use crate::test_helpers::{alt_key, ctrl_key, key};

    use super::*;

    #[test]
    fn ctrl_o_jumps_backward() {
        let cmds = handle_normal_mode(&ctrl_key('o'));
        assert_single_command!(cmds, EditorCommand::JumpBackward);
    }

    #[test]
    fn ctrl_i_jumps_forward() {
        let cmds = handle_normal_mode(&ctrl_key('i'));
        assert_single_command!(cmds, EditorCommand::JumpForward);
    }

    #[test]
    fn ctrl_s_saves_selection() {
        let cmds = handle_normal_mode(&ctrl_key('s'));
        assert_single_command!(cmds, EditorCommand::SaveSelection);
    }

    #[test]
    fn space_j_shows_jumplist_picker() {
        let cmds = handle_space_leader(&key('j'));
        assert_single_command!(cmds, EditorCommand::ShowJumpListPicker);
    }

    #[test]
    fn alt_o_expands_selection() {
        let cmds = handle_normal_mode(&alt_key('o'));
        assert_single_command!(cmds, EditorCommand::ExpandSelection);
    }

    #[test]
    fn alt_i_shrinks_selection() {
        let cmds = handle_normal_mode(&alt_key('i'));
        assert_single_command!(cmds, EditorCommand::ShrinkSelection);
    }
}
