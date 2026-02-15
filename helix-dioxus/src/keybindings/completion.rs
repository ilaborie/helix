//! Completion mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::EditorCommand;

/// Handle keyboard input when completion popup is visible.
#[must_use]
pub fn handle_completion_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Handle control key combinations
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            // Ctrl+Space when already showing - close completion
            KeyCode::Char(' ') => vec![EditorCommand::CompletionCancel],
            // Ctrl+n - next item
            KeyCode::Char('n') => vec![EditorCommand::CompletionDown],
            // Ctrl+p - previous item
            KeyCode::Char('p') => vec![EditorCommand::CompletionUp],
            _ => vec![],
        };
    }

    match key.code {
        // Cancel completion
        KeyCode::Esc => vec![EditorCommand::CompletionCancel],
        // Navigate
        KeyCode::Up => vec![EditorCommand::CompletionUp],
        KeyCode::Down => vec![EditorCommand::CompletionDown],
        // Confirm selection
        KeyCode::Enter | KeyCode::Tab => vec![EditorCommand::CompletionConfirm],
        // Continue typing - forward to insert mode
        KeyCode::Char(ch) => vec![EditorCommand::CompletionCancel, EditorCommand::InsertChar(ch)],
        KeyCode::Backspace => vec![EditorCommand::CompletionCancel, EditorCommand::DeleteCharBackward],
        _ => vec![],
    }
}

/// Handle keyboard input when location picker is visible.
#[must_use]
pub fn handle_location_picker_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    super::handle_list_navigation_keys(
        key.code,
        EditorCommand::LocationCancel,
        EditorCommand::LocationUp,
        EditorCommand::LocationDown,
        EditorCommand::LocationConfirm,
        None,
        None,
    )
}

/// Handle keyboard input when code actions menu is visible.
/// Navigation uses arrow keys only; typing adds to the filter.
#[must_use]
pub fn handle_code_actions_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    super::handle_list_navigation_keys(
        key.code,
        EditorCommand::CodeActionCancel,
        EditorCommand::CodeActionUp,
        EditorCommand::CodeActionDown,
        EditorCommand::CodeActionConfirm,
        Some(EditorCommand::CodeActionFilterBackspace),
        Some(&EditorCommand::CodeActionFilterChar),
    )
}

/// Handle keyboard input when LSP dialog is visible.
#[must_use]
pub fn handle_lsp_dialog_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => vec![EditorCommand::CloseLspDialog],
        KeyCode::Up | KeyCode::Char('k') => vec![EditorCommand::LspDialogUp],
        KeyCode::Down | KeyCode::Char('j') => vec![EditorCommand::LspDialogDown],
        KeyCode::Char('r') => vec![EditorCommand::RestartSelectedLsp],
        _ => vec![],
    }
}
