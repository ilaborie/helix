//! Regex select/split mode keybinding handler.

use helix_view::input::KeyEvent;

use super::handle_text_input_keys;
use crate::state::EditorCommand;

/// Handle keyboard input in Regex mode (select/split).
pub fn handle_regex_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    handle_text_input_keys(
        key.code,
        EditorCommand::ExitRegexMode,
        EditorCommand::RegexExecute,
        EditorCommand::RegexBackspace,
        EditorCommand::RegexInput,
    )
}
