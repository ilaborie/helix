//! Search mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent};

use crate::state::EditorCommand;

/// Handle keyboard input in Search mode.
pub fn handle_search_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc => vec![EditorCommand::ExitSearchMode],
        KeyCode::Enter => vec![EditorCommand::SearchExecute],
        KeyCode::Backspace => vec![EditorCommand::SearchBackspace],
        KeyCode::Char(ch) => vec![EditorCommand::SearchInput(ch)],
        _ => vec![],
    }
}
