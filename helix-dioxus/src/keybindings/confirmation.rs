//! Confirmation dialog mode keybindings.
//!
//! Handles keyboard input when a confirmation dialog is visible.

use helix_view::input::{KeyCode, KeyEvent};

use crate::state::EditorCommand;

/// Handle input in confirmation dialog mode.
#[must_use]
pub fn handle_confirmation_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // y/Y or Enter confirms the primary action
        KeyCode::Char('y' | 'Y') | KeyCode::Enter => {
            vec![EditorCommand::ConfirmationDialogConfirm]
        }
        // n/N denies (e.g., "Don't Save")
        KeyCode::Char('n' | 'N') => vec![EditorCommand::ConfirmationDialogDeny],
        // Escape cancels (dismisses dialog without action)
        KeyCode::Esc => vec![EditorCommand::ConfirmationDialogCancel],
        _ => vec![],
    }
}
