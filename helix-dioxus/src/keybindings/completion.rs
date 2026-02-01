//! Completion mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::EditorCommand;

/// Handle keyboard input when completion popup is visible.
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
        KeyCode::Char(ch) => vec![
            EditorCommand::CompletionCancel,
            EditorCommand::InsertChar(ch),
        ],
        KeyCode::Backspace => vec![
            EditorCommand::CompletionCancel,
            EditorCommand::DeleteCharBackward,
        ],
        _ => vec![],
    }
}

/// Handle keyboard input when location picker is visible.
pub fn handle_location_picker_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc => vec![EditorCommand::LocationCancel],
        KeyCode::Up | KeyCode::Char('k') => vec![EditorCommand::LocationUp],
        KeyCode::Down | KeyCode::Char('j') => vec![EditorCommand::LocationDown],
        KeyCode::Enter => vec![EditorCommand::LocationConfirm],
        _ => vec![],
    }
}

/// Handle keyboard input when code actions menu is visible.
/// Navigation uses arrow keys only; typing adds to the filter.
pub fn handle_code_actions_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc => vec![EditorCommand::CodeActionCancel],
        KeyCode::Up => vec![EditorCommand::CodeActionUp],
        KeyCode::Down => vec![EditorCommand::CodeActionDown],
        KeyCode::Enter => vec![EditorCommand::CodeActionConfirm],
        KeyCode::Backspace => vec![EditorCommand::CodeActionFilterBackspace],
        KeyCode::Char(ch) => vec![EditorCommand::CodeActionFilterChar(ch)],
        _ => vec![],
    }
}

/// Handle keyboard input when LSP dialog is visible.
pub fn handle_lsp_dialog_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc => vec![EditorCommand::CloseLspDialog],
        KeyCode::Up | KeyCode::Char('k') => vec![EditorCommand::LspDialogUp],
        KeyCode::Down | KeyCode::Char('j') => vec![EditorCommand::LspDialogDown],
        KeyCode::Char('r') => vec![EditorCommand::RestartSelectedLsp],
        KeyCode::Enter => vec![EditorCommand::CloseLspDialog],
        _ => vec![],
    }
}
