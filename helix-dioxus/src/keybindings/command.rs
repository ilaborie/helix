//! Command mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent};

use crate::state::EditorCommand;

/// Handle keyboard input in Command mode.
pub fn handle_command_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc => vec![EditorCommand::ExitCommandMode],
        KeyCode::Enter => vec![EditorCommand::CommandExecute],
        KeyCode::Backspace => vec![EditorCommand::CommandBackspace],
        KeyCode::Char(c) => vec![EditorCommand::CommandInput(c)],
        _ => vec![],
    }
}
