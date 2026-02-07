//! Picker mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::EditorCommand;

/// Handle keyboard input in File picker mode.
pub fn handle_picker_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Ctrl+n/p for navigation (like many fuzzy finders)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('n') => vec![EditorCommand::PickerDown],
            KeyCode::Char('p') => vec![EditorCommand::PickerUp],
            _ => vec![],
        };
    }

    match key.code {
        KeyCode::Esc => vec![EditorCommand::PickerCancel],
        KeyCode::Enter => vec![EditorCommand::PickerConfirm],
        KeyCode::Down => vec![EditorCommand::PickerDown],
        KeyCode::Up => vec![EditorCommand::PickerUp],
        KeyCode::Home => vec![EditorCommand::PickerFirst],
        KeyCode::End => vec![EditorCommand::PickerLast],
        KeyCode::PageUp => vec![EditorCommand::PickerPageUp],
        KeyCode::PageDown => vec![EditorCommand::PickerPageDown],
        KeyCode::Backspace => vec![EditorCommand::PickerBackspace],
        KeyCode::Char(ch) => vec![EditorCommand::PickerInput(ch)],
        _ => vec![],
    }
}
