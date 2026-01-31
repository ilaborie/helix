//! Insert mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::EditorCommand;

/// Handle keyboard input in Insert mode.
pub fn handle_insert_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Check for Escape first
    if key.code == KeyCode::Esc {
        return vec![EditorCommand::ExitInsertMode];
    }

    // Handle control key combinations
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return vec![];
    }

    match key.code {
        KeyCode::Char(c) => vec![EditorCommand::InsertChar(c)],
        KeyCode::Enter => vec![EditorCommand::InsertNewline],
        KeyCode::Backspace => vec![EditorCommand::DeleteCharBackward],
        KeyCode::Delete => vec![EditorCommand::DeleteCharForward],
        KeyCode::Left => vec![EditorCommand::MoveLeft],
        KeyCode::Right => vec![EditorCommand::MoveRight],
        KeyCode::Up => vec![EditorCommand::MoveUp],
        KeyCode::Down => vec![EditorCommand::MoveDown],
        _ => vec![],
    }
}
