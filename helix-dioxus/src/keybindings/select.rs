//! Select mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use super::handle_extend_keys;
use crate::state::EditorCommand;

/// Handle two-key sequences starting with 'g' in select mode.
///
/// Extend-aware variants: gg, ge, gs, g| preserve the anchor.
/// LSP gotos and window/buffer navigation reuse normal-mode commands.
pub fn handle_select_g_prefix(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        // Extend variants (preserve anchor)
        KeyCode::Char('g') => vec![EditorCommand::ExtendToFirstLine],
        KeyCode::Char('e') => vec![EditorCommand::ExtendToLastLine],
        KeyCode::Char('h') => vec![EditorCommand::ExtendLineStart],
        KeyCode::Char('l') => vec![EditorCommand::ExtendLineEnd],
        KeyCode::Char('s') => vec![EditorCommand::ExtendGotoFirstNonWhitespace],
        KeyCode::Char('|') => vec![EditorCommand::ExtendGotoColumn],

        // LSP gotos (jump, don't extend)
        KeyCode::Char('d') => vec![EditorCommand::GotoDefinition],
        KeyCode::Char('D') => vec![EditorCommand::GotoDeclaration],
        KeyCode::Char('y') => vec![EditorCommand::GotoTypeDefinition],
        KeyCode::Char('i') => vec![EditorCommand::GotoImplementation],
        KeyCode::Char('r') => vec![EditorCommand::GotoReferences],
        KeyCode::Char('f') => vec![EditorCommand::GotoFileUnderCursor],

        // Window / buffer navigation (same as normal mode)
        KeyCode::Char('t') => vec![EditorCommand::GotoWindowTop],
        KeyCode::Char('c') => vec![EditorCommand::GotoWindowCenter],
        KeyCode::Char('b') => vec![EditorCommand::GotoWindowBottom],
        KeyCode::Char('n') => vec![EditorCommand::NextBuffer],
        KeyCode::Char('p') => vec![EditorCommand::PreviousBuffer],
        KeyCode::Char('a') => vec![EditorCommand::GotoLastAccessedFile],
        KeyCode::Char('m') => vec![EditorCommand::GotoLastModifiedFile],
        KeyCode::Char('.') => vec![EditorCommand::GotoLastModification],

        _ => vec![],
    }
}

/// Handle keyboard input in Select mode.
pub fn handle_select_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    // Handle Alt+key combinations
    if key.modifiers.contains(KeyModifiers::ALT) {
        return match key.code {
            // Alt+; = flip selections
            KeyCode::Char(';') => vec![EditorCommand::FlipSelections],
            // Alt+C = copy selection to previous line
            KeyCode::Char('C') => vec![EditorCommand::CopySelectionOnPrevLine],
            // Alt+s = split selection on newlines
            KeyCode::Char('s') => vec![EditorCommand::SplitSelectionOnNewline],
            _ => vec![],
        };
    }

    // Handle Ctrl+key combinations
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => vec![EditorCommand::ToggleLineComment],
            _ => vec![],
        };
    }

    // Direction keys - extends selection (hjkl + arrows)
    if let Some(cmds) = handle_extend_keys(key.code) {
        return cmds;
    }

    match key.code {
        // Exit select mode
        KeyCode::Esc => vec![EditorCommand::ExitSelectMode],

        // Word movement - extends selection
        KeyCode::Char('w') => vec![EditorCommand::ExtendWordForward],
        KeyCode::Char('b') => vec![EditorCommand::ExtendWordBackward],
        KeyCode::Char('e') => vec![EditorCommand::ExtendWordEnd],

        // WORD movement - extends selection
        KeyCode::Char('W') => vec![EditorCommand::ExtendLongWordForward],
        KeyCode::Char('B') => vec![EditorCommand::ExtendLongWordBackward],
        KeyCode::Char('E') => vec![EditorCommand::ExtendLongWordEnd],

        // Line movement - extends selection
        KeyCode::Char('0') | KeyCode::Home => vec![EditorCommand::ExtendLineStart],
        KeyCode::Char('$') | KeyCode::End => vec![EditorCommand::ExtendLineEnd],

        // Page movement (moves cursor, exits select mode for now)
        KeyCode::PageUp => vec![EditorCommand::PageUp],
        KeyCode::PageDown => vec![EditorCommand::PageDown],

        // Line selection
        KeyCode::Char('x') => vec![EditorCommand::SelectLine],
        KeyCode::Char('X') => vec![EditorCommand::ExtendLine],

        // Clipboard operations
        KeyCode::Char('y') => vec![EditorCommand::Yank, EditorCommand::ExitSelectMode],
        KeyCode::Char('d') => vec![EditorCommand::DeleteSelection],

        // Change selection (delete + enter insert)
        KeyCode::Char('c') => vec![EditorCommand::ChangeSelection],

        // Extend search
        KeyCode::Char('n') => vec![EditorCommand::ExtendSearchNext],
        KeyCode::Char('N') => vec![EditorCommand::ExtendSearchPrev],

        // Replace with yanked text / paste replaces selection
        KeyCode::Char('R' | 'p') => vec![EditorCommand::ReplaceWithYanked],

        // Toggle back to normal mode
        KeyCode::Char('v') => vec![EditorCommand::ExitSelectMode],

        // Indent/unindent
        KeyCode::Char('>') => vec![EditorCommand::IndentLine],
        KeyCode::Char('<') => vec![EditorCommand::UnindentLine],

        // Selection operations
        KeyCode::Char(';') => vec![EditorCommand::CollapseSelection],
        KeyCode::Char(',') => vec![EditorCommand::KeepPrimarySelection],

        // Regex select/split
        KeyCode::Char('s') => vec![EditorCommand::EnterRegexMode { split: false }],
        KeyCode::Char('S') => vec![EditorCommand::EnterRegexMode { split: true }],

        // Copy selection on next line
        KeyCode::Char('C') => vec![EditorCommand::CopySelectionOnNextLine],

        // Rotate selections
        KeyCode::Char(')') => vec![EditorCommand::RotateSelectionsForward],
        KeyCode::Char('(') => vec![EditorCommand::RotateSelectionsBackward],

        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use helix_view::input::KeyEvent;

    use super::*;

    fn key(ch: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyModifiers::NONE,
        }
    }

    // --- handle_select_g_prefix ---

    #[test]
    fn select_g_prefix_gg_extends_to_first_line() {
        let cmds = handle_select_g_prefix(&key('g'));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendToFirstLine));
    }

    #[test]
    fn select_g_prefix_ge_extends_to_last_line() {
        let cmds = handle_select_g_prefix(&key('e'));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendToLastLine));
    }

    #[test]
    fn select_g_prefix_gh_extends_line_start() {
        let cmds = handle_select_g_prefix(&key('h'));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendLineStart));
    }

    #[test]
    fn select_g_prefix_gl_extends_line_end() {
        let cmds = handle_select_g_prefix(&key('l'));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendLineEnd));
    }

    #[test]
    fn select_g_prefix_gs_extends_first_nonwhitespace() {
        let cmds = handle_select_g_prefix(&key('s'));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendGotoFirstNonWhitespace));
    }

    #[test]
    fn select_g_prefix_g_pipe_extends_goto_column() {
        let cmds = handle_select_g_prefix(&key('|'));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendGotoColumn));
    }

    #[test]
    fn select_g_prefix_gd_goes_to_definition() {
        let cmds = handle_select_g_prefix(&key('d'));
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::GotoDefinition));
    }

    #[test]
    fn select_g_prefix_unrecognized_returns_empty() {
        let cmds = handle_select_g_prefix(&key('z'));
        assert!(cmds.is_empty());
    }
}
