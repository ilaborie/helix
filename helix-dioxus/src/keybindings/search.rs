//! Search mode keybinding handler.

use helix_view::input::KeyEvent;

use super::handle_text_input_keys;
use crate::state::EditorCommand;

/// Handle keyboard input in Search mode.
pub fn handle_search_mode(key: &KeyEvent) -> Vec<EditorCommand> {
    handle_text_input_keys(
        key.code,
        EditorCommand::ExitSearchMode,
        EditorCommand::SearchExecute,
        EditorCommand::SearchBackspace,
        EditorCommand::SearchInput,
    )
}
