//! Command mode keybinding handler.

use helix_view::input::KeyEvent;

use super::handle_text_input_keys;
use crate::state::EditorCommand;

/// Handle keyboard input in Command mode.
pub fn handle_command_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    handle_text_input_keys(
        key.code,
        EditorCommand::ExitCommandMode,
        EditorCommand::CommandExecute,
        EditorCommand::CommandBackspace,
        EditorCommand::CommandInput,
    )
}
