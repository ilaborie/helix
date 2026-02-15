//! Input dialog mode keybindings.
//!
//! Handles keyboard input when an input dialog is visible.

use helix_view::input::{KeyCode, KeyEvent};

use crate::state::EditorCommand;

/// Handle input in input dialog mode.
#[must_use]
pub fn handle_input_dialog_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc => vec![EditorCommand::InputDialogCancel],
        KeyCode::Enter => vec![EditorCommand::InputDialogConfirm],
        KeyCode::Backspace => vec![EditorCommand::InputDialogBackspace],
        KeyCode::Char(ch) => vec![EditorCommand::InputDialogInput(ch)],
        _ => vec![],
    }
}
