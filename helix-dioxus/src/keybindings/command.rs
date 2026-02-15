//! Command mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent};

use crate::state::EditorCommand;

/// Handle keyboard input in Command mode.
///
/// Tab accepts the selected completion, Up/Down navigate the completion list,
/// and the remaining keys behave as before (Esc exits, Enter executes, etc.).
#[must_use]
pub fn handle_command_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    match key.code {
        KeyCode::Esc => vec![EditorCommand::ExitCommandMode],
        KeyCode::Enter => vec![EditorCommand::CommandExecute],
        KeyCode::Backspace => vec![EditorCommand::CommandBackspace],
        KeyCode::Tab => vec![EditorCommand::CommandCompletionAccept],
        KeyCode::Up => vec![EditorCommand::CommandCompletionUp],
        KeyCode::Down => vec![EditorCommand::CommandCompletionDown],
        KeyCode::Char(c) => vec![EditorCommand::CommandInput(c)],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }

    #[test]
    fn tab_accepts_completion() {
        let cmds = handle_command_mode(&key(KeyCode::Tab));
        assert!(cmds.len() == 1);
        assert!(matches!(cmds[0], EditorCommand::CommandCompletionAccept));
    }

    #[test]
    fn up_navigates_completion() {
        let cmds = handle_command_mode(&key(KeyCode::Up));
        assert!(cmds.len() == 1);
        assert!(matches!(cmds[0], EditorCommand::CommandCompletionUp));
    }

    #[test]
    fn down_navigates_completion() {
        let cmds = handle_command_mode(&key(KeyCode::Down));
        assert!(cmds.len() == 1);
        assert!(matches!(cmds[0], EditorCommand::CommandCompletionDown));
    }

    #[test]
    fn esc_exits_command_mode() {
        let cmds = handle_command_mode(&key(KeyCode::Esc));
        assert!(cmds.len() == 1);
        assert!(matches!(cmds[0], EditorCommand::ExitCommandMode));
    }

    #[test]
    fn enter_executes_command() {
        let cmds = handle_command_mode(&key(KeyCode::Enter));
        assert!(cmds.len() == 1);
        assert!(matches!(cmds[0], EditorCommand::CommandExecute));
    }

    #[test]
    fn char_input() {
        let cmds = handle_command_mode(&key(KeyCode::Char('w')));
        assert!(cmds.len() == 1);
        assert!(matches!(cmds[0], EditorCommand::CommandInput('w')));
    }

    #[test]
    fn backspace_deletes() {
        let cmds = handle_command_mode(&key(KeyCode::Backspace));
        assert!(cmds.len() == 1);
        assert!(matches!(cmds[0], EditorCommand::CommandBackspace));
    }
}
