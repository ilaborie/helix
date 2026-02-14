//! Insert mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::EditorCommand;

/// Handle keyboard input in Insert mode.
pub fn handle_insert_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Check for Escape first
    if key.code == KeyCode::Esc {
        return vec![EditorCommand::ExitInsertMode];
    }

    // Handle Alt+key combinations
    if key.modifiers.contains(KeyModifiers::ALT) {
        return match key.code {
            // Alt+d - delete word forward
            KeyCode::Char('d') => vec![EditorCommand::DeleteWordForward],
            _ => vec![],
        };
    }

    // Handle control key combinations
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            // Ctrl+c - toggle line comment
            KeyCode::Char('c') => vec![EditorCommand::ToggleLineComment],
            // Ctrl+d - delete char forward (alt delete)
            KeyCode::Char('d') => vec![EditorCommand::DeleteCharForward],
            // Ctrl+h - delete char backward (alt backspace)
            KeyCode::Char('h') => vec![EditorCommand::DeleteCharBackward],
            // Ctrl+j - insert newline (alt enter)
            KeyCode::Char('j') => vec![EditorCommand::InsertNewline],
            // Ctrl+k - kill to line end
            KeyCode::Char('k') => vec![EditorCommand::KillToLineEnd],
            // Ctrl+Space - trigger completion
            KeyCode::Char(' ') => vec![EditorCommand::TriggerCompletion],
            // Ctrl+. - show code actions (quick fix)
            KeyCode::Char('.') => vec![EditorCommand::ShowCodeActions],
            // Ctrl+s - commit undo checkpoint
            KeyCode::Char('s') => vec![EditorCommand::CommitUndoCheckpoint],
            // Ctrl+w - delete word backward
            KeyCode::Char('w') => vec![EditorCommand::DeleteWordBackward],
            // Ctrl+u - delete to line start
            KeyCode::Char('u') => vec![EditorCommand::DeleteToLineStart],
            _ => vec![],
        };
    }

    match key.code {
        KeyCode::Char(c) => vec![EditorCommand::InsertChar(c)],
        KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
            vec![EditorCommand::UnindentLine]
        }
        KeyCode::Tab => vec![EditorCommand::InsertTab],
        KeyCode::Enter => vec![EditorCommand::InsertNewline],
        KeyCode::Backspace => vec![EditorCommand::DeleteCharBackward],
        KeyCode::Delete => vec![EditorCommand::DeleteCharForward],
        KeyCode::Left => vec![EditorCommand::MoveLeft],
        KeyCode::Right => vec![EditorCommand::MoveRight],
        KeyCode::Up => vec![EditorCommand::MoveUp],
        KeyCode::Down => vec![EditorCommand::MoveDown],
        KeyCode::Home => vec![EditorCommand::MoveLineStart],
        KeyCode::End => vec![EditorCommand::MoveLineEnd],
        KeyCode::PageUp => vec![EditorCommand::PageUp],
        KeyCode::PageDown => vec![EditorCommand::PageDown],
        _ => vec![],
    }
}
