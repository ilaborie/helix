//! Select mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent};

use crate::state::EditorCommand;

/// Handle keyboard input in Select mode.
pub fn handle_select_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // Exit select mode
        KeyCode::Esc => vec![EditorCommand::ExitSelectMode],

        // Character movement - extends selection
        KeyCode::Char('h') | KeyCode::Left => vec![EditorCommand::ExtendLeft],
        KeyCode::Char('l') | KeyCode::Right => vec![EditorCommand::ExtendRight],
        KeyCode::Char('j') | KeyCode::Down => vec![EditorCommand::ExtendDown],
        KeyCode::Char('k') | KeyCode::Up => vec![EditorCommand::ExtendUp],

        // Word movement - extends selection
        KeyCode::Char('w') => vec![EditorCommand::ExtendWordForward],
        KeyCode::Char('b') => vec![EditorCommand::ExtendWordBackward],

        // Line movement - extends selection
        KeyCode::Char('0') | KeyCode::Home => vec![EditorCommand::ExtendLineStart],
        KeyCode::Char('$') | KeyCode::End => vec![EditorCommand::ExtendLineEnd],

        // Line selection
        KeyCode::Char('x') => vec![EditorCommand::SelectLine],
        KeyCode::Char('X') => vec![EditorCommand::ExtendLine],

        // Clipboard operations
        KeyCode::Char('y') => vec![EditorCommand::Yank, EditorCommand::ExitSelectMode],
        KeyCode::Char('d') => vec![EditorCommand::DeleteSelection],

        // Paste replaces selection
        KeyCode::Char('p') => vec![EditorCommand::DeleteSelection, EditorCommand::Paste],

        _ => vec![],
    }
}
