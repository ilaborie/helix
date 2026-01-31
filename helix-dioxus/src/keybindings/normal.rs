//! Normal mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::EditorCommand;

/// Handle keyboard input in Normal mode.
pub fn handle_normal_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Handle Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('r') => vec![EditorCommand::Redo],
            KeyCode::Char('h') => vec![EditorCommand::PreviousBuffer],
            KeyCode::Char('l') => vec![EditorCommand::NextBuffer],
            _ => vec![],
        };
    }

    match key.code {
        // Movement
        KeyCode::Char('h') | KeyCode::Left => vec![EditorCommand::MoveLeft],
        KeyCode::Char('l') | KeyCode::Right => vec![EditorCommand::MoveRight],
        KeyCode::Char('j') | KeyCode::Down => vec![EditorCommand::MoveDown],
        KeyCode::Char('k') | KeyCode::Up => vec![EditorCommand::MoveUp],

        // Word movement
        KeyCode::Char('w') => vec![EditorCommand::MoveWordForward],
        KeyCode::Char('b') => vec![EditorCommand::MoveWordBackward],

        // Line movement
        KeyCode::Char('0') | KeyCode::Home => vec![EditorCommand::MoveLineStart],
        KeyCode::Char('$') | KeyCode::End => vec![EditorCommand::MoveLineEnd],

        // File navigation
        KeyCode::Char('G') => vec![EditorCommand::GotoLastLine],
        // TODO: 'gg' requires tracking previous key

        // Mode changes
        KeyCode::Char('i') => vec![EditorCommand::EnterInsertMode],
        KeyCode::Char('a') => vec![EditorCommand::EnterInsertModeAfter],
        KeyCode::Char('A') => vec![EditorCommand::EnterInsertModeLineEnd],
        KeyCode::Char('o') => vec![EditorCommand::OpenLineBelow],
        KeyCode::Char('O') => vec![EditorCommand::OpenLineAbove],

        // History
        KeyCode::Char('u') => vec![EditorCommand::Undo],
        KeyCode::Char('U') => vec![EditorCommand::Redo], // Shift+U also redoes (helix convention)

        // Visual selection mode
        KeyCode::Char('v') => vec![EditorCommand::EnterSelectMode],

        // Line selection (helix x/X)
        KeyCode::Char('x') => vec![EditorCommand::SelectLine],

        // Delete selection (works in normal mode due to selection-first model)
        KeyCode::Char('d') => vec![EditorCommand::DeleteSelection],

        // Clipboard
        KeyCode::Char('p') => vec![EditorCommand::Paste],
        KeyCode::Char('P') => vec![EditorCommand::PasteBefore],
        KeyCode::Char('y') => vec![EditorCommand::Yank],

        // Search
        KeyCode::Char('/') => vec![EditorCommand::EnterSearchMode { backwards: false }],
        KeyCode::Char('?') => vec![EditorCommand::EnterSearchMode { backwards: true }],
        KeyCode::Char('n') => vec![EditorCommand::SearchNext],
        KeyCode::Char('N') => vec![EditorCommand::SearchPrevious],

        // Command mode
        KeyCode::Char(':') => vec![EditorCommand::EnterCommandMode],

        _ => vec![],
    }
}
