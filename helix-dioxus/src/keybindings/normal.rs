//! Normal mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::state::EditorCommand;

/// Handle keyboard input in Normal mode.
pub fn handle_normal_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Handle Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => vec![EditorCommand::ToggleLineComment],
            KeyCode::Char('r') => vec![EditorCommand::Redo],
            KeyCode::Char('h') => vec![EditorCommand::PreviousBuffer],
            KeyCode::Char('l') => vec![EditorCommand::NextBuffer],
            // Ctrl+Space or Ctrl+. - show code actions (quick fix)
            KeyCode::Char(' ') | KeyCode::Char('.') => vec![EditorCommand::ShowCodeActions],
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
        KeyCode::PageUp => vec![EditorCommand::PageUp],
        KeyCode::PageDown => vec![EditorCommand::PageDown],

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

        // LSP - Hover (K for documentation like vim)
        KeyCode::Char('K') => vec![EditorCommand::TriggerHover],

        _ => vec![],
    }
}

/// Handle two-key sequences starting with 'g' (goto).
pub fn handle_g_prefix(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // gg - go to first line
        KeyCode::Char('g') => vec![EditorCommand::GotoFirstLine],
        // gd - go to definition
        KeyCode::Char('d') => vec![EditorCommand::GotoDefinition],
        // gr - go to references
        KeyCode::Char('r') => vec![EditorCommand::GotoReferences],
        // gy - go to type definition
        KeyCode::Char('y') => vec![EditorCommand::GotoTypeDefinition],
        // gi - go to implementation
        KeyCode::Char('i') => vec![EditorCommand::GotoImplementation],
        _ => vec![],
    }
}

/// Handle two-key sequences starting with ']' (next).
pub fn handle_bracket_next(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // ]d - next diagnostic
        KeyCode::Char('d') => vec![EditorCommand::NextDiagnostic],
        _ => vec![],
    }
}

/// Handle two-key sequences starting with '[' (previous).
pub fn handle_bracket_prev(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // [d - previous diagnostic
        KeyCode::Char('d') => vec![EditorCommand::PrevDiagnostic],
        _ => vec![],
    }
}

/// Handle two-key sequences starting with Space (leader).
pub fn handle_space_leader(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // Space a - show code actions
        KeyCode::Char('a') => vec![EditorCommand::ShowCodeActions],
        // Space c - toggle line comment
        KeyCode::Char('c') => vec![EditorCommand::ToggleLineComment],
        // Space C - toggle block comment
        KeyCode::Char('C') => vec![EditorCommand::ToggleBlockComment],
        // Space d - document diagnostics
        KeyCode::Char('d') => vec![EditorCommand::ShowDocumentDiagnostics],
        // Space D - workspace diagnostics
        KeyCode::Char('D') => vec![EditorCommand::ShowWorkspaceDiagnostics],
        // Space f - format document
        KeyCode::Char('f') => vec![EditorCommand::FormatDocument],
        // Space i - toggle inlay hints
        KeyCode::Char('i') => vec![EditorCommand::ToggleInlayHints],
        // Space r - rename symbol
        KeyCode::Char('r') => vec![EditorCommand::RenameSymbol],
        // Space s - document symbols
        KeyCode::Char('s') => vec![EditorCommand::ShowDocumentSymbols],
        // Space S - workspace symbols
        KeyCode::Char('S') => vec![EditorCommand::ShowWorkspaceSymbols],
        _ => vec![],
    }
}
