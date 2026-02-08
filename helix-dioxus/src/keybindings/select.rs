//! Select mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use super::handle_extend_keys;
use crate::state::EditorCommand;

/// Handle keyboard input in Select mode.
pub fn handle_select_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Handle Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => vec![EditorCommand::ToggleLineComment],
            _ => vec![],
        };
    }

    // Direction keys - extends selection (hjkl + arrows)
    if let Some(cmds) = handle_extend_keys(key.code) {
        return cmds;
    }

    match key.code {
        // Exit select mode
        KeyCode::Esc => vec![EditorCommand::ExitSelectMode],

        // Word movement - extends selection
        KeyCode::Char('w') => vec![EditorCommand::ExtendWordForward],
        KeyCode::Char('b') => vec![EditorCommand::ExtendWordBackward],
        KeyCode::Char('e') => vec![EditorCommand::ExtendWordEnd],

        // WORD movement - extends selection
        KeyCode::Char('W') => vec![EditorCommand::ExtendLongWordForward],
        KeyCode::Char('B') => vec![EditorCommand::ExtendLongWordBackward],
        KeyCode::Char('E') => vec![EditorCommand::ExtendLongWordEnd],

        // Line movement - extends selection
        KeyCode::Char('0') | KeyCode::Home => vec![EditorCommand::ExtendLineStart],
        KeyCode::Char('$') | KeyCode::End => vec![EditorCommand::ExtendLineEnd],

        // Page movement (moves cursor, exits select mode for now)
        KeyCode::PageUp => vec![EditorCommand::PageUp],
        KeyCode::PageDown => vec![EditorCommand::PageDown],

        // Line selection
        KeyCode::Char('x') => vec![EditorCommand::SelectLine],
        KeyCode::Char('X') => vec![EditorCommand::ExtendLine],

        // Clipboard operations
        KeyCode::Char('y') => vec![EditorCommand::Yank, EditorCommand::ExitSelectMode],
        KeyCode::Char('d') => vec![EditorCommand::DeleteSelection],

        // Change selection (delete + enter insert)
        KeyCode::Char('c') => vec![EditorCommand::ChangeSelection],

        // Extend search
        KeyCode::Char('n') => vec![EditorCommand::ExtendSearchNext],
        KeyCode::Char('N') => vec![EditorCommand::ExtendSearchPrev],

        // Replace with yanked text / paste replaces selection
        KeyCode::Char('R' | 'p') => vec![EditorCommand::ReplaceWithYanked],

        // Toggle back to normal mode
        KeyCode::Char('v') => vec![EditorCommand::ExitSelectMode],

        // Indent/unindent
        KeyCode::Char('>') => vec![EditorCommand::IndentLine],
        KeyCode::Char('<') => vec![EditorCommand::UnindentLine],

        // Selection operations
        KeyCode::Char(';') => vec![EditorCommand::CollapseSelection],
        KeyCode::Char(',') => vec![EditorCommand::KeepPrimarySelection],

        _ => vec![],
    }
}
