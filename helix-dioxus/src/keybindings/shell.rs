//! Shell mode keybinding handler.

use helix_view::input::KeyEvent;

use super::handle_text_input_keys;
use crate::state::EditorCommand;

/// Handle keyboard input in Shell prompt mode.
pub fn handle_shell_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    handle_text_input_keys(
        key.code,
        EditorCommand::ExitShellMode,
        EditorCommand::ShellExecute,
        EditorCommand::ShellBackspace,
        EditorCommand::ShellInput,
    )
}
