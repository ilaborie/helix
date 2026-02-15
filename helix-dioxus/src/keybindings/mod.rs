//! Keybinding handlers for dialog and prompt modes.
//!
//! Normal, insert, and select mode dispatch is handled by the keymap system
//! (`crate::keymap`). This module contains handlers for overlay UIs:
//! command prompt, search prompt, regex prompt, shell prompt, picker,
//! completion, confirmation, input dialog, and LSP dialogs.

mod command;
mod completion;
mod confirmation;
mod input_dialog;
mod picker;
mod regex;
mod search;
mod shell;
mod translate;

use helix_view::input::KeyCode;

use crate::state::EditorCommand;

pub use command::handle_command_mode;
pub use completion::{
    handle_code_actions_mode, handle_completion_mode, handle_location_picker_mode,
    handle_lsp_dialog_mode,
};
pub use confirmation::handle_confirmation_mode;
pub use input_dialog::handle_input_dialog_mode;
pub use picker::handle_picker_mode;
pub use regex::handle_regex_mode;
pub use search::handle_search_mode;
pub use shell::handle_shell_mode;
pub use translate::translate_key_event;

/// Handle text input keys shared by search and command modes.
///
/// Returns commands for Esc (exit), Enter (execute), Backspace, and Char input.
/// `exit_cmd` is the command for Esc, `execute_cmd` for Enter,
/// `backspace_cmd` for Backspace, and `input_cmd` maps a char to a command.
fn handle_text_input_keys(
    code: KeyCode,
    exit_cmd: EditorCommand,
    execute_cmd: EditorCommand,
    backspace_cmd: EditorCommand,
    input_cmd: impl FnOnce(char) -> EditorCommand,
) -> Vec<EditorCommand> {
    match code {
        KeyCode::Esc => vec![exit_cmd],
        KeyCode::Enter => vec![execute_cmd],
        KeyCode::Backspace => vec![backspace_cmd],
        KeyCode::Char(ch) => vec![input_cmd(ch)],
        _ => vec![],
    }
}

/// Handle list navigation keys shared by location picker and code actions.
///
/// Handles Esc (cancel), Up/Down (navigate), Enter (confirm).
/// When `has_filter` is true, Backspace and Char keys map to filter commands
/// instead of j/k navigation.
fn handle_list_navigation_keys(
    code: KeyCode,
    cancel_cmd: EditorCommand,
    up_cmd: EditorCommand,
    down_cmd: EditorCommand,
    confirm_cmd: EditorCommand,
    filter_backspace_cmd: Option<EditorCommand>,
    filter_char_cmd: Option<&dyn Fn(char) -> EditorCommand>,
) -> Vec<EditorCommand> {
    match code {
        KeyCode::Esc => vec![cancel_cmd],
        KeyCode::Up => vec![up_cmd.clone()],
        KeyCode::Down => vec![down_cmd.clone()],
        KeyCode::Enter => vec![confirm_cmd],
        KeyCode::Char('k') if filter_char_cmd.is_none() => vec![up_cmd],
        KeyCode::Char('j') if filter_char_cmd.is_none() => vec![down_cmd],
        KeyCode::Backspace => match filter_backspace_cmd {
            Some(cmd) => vec![cmd],
            None => vec![],
        },
        KeyCode::Char(ch) => match filter_char_cmd {
            Some(f) => vec![f(ch)],
            None => vec![],
        },
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_single_command;

    use super::*;

    // --- handle_text_input_keys ---

    #[test]
    fn text_input_keys_esc() {
        let cmds = handle_text_input_keys(
            KeyCode::Esc,
            EditorCommand::ExitSearchMode,
            EditorCommand::SearchExecute,
            EditorCommand::SearchBackspace,
            EditorCommand::SearchInput,
        );
        assert_single_command!(cmds, EditorCommand::ExitSearchMode);
    }

    #[test]
    fn text_input_keys_enter() {
        let cmds = handle_text_input_keys(
            KeyCode::Enter,
            EditorCommand::ExitSearchMode,
            EditorCommand::SearchExecute,
            EditorCommand::SearchBackspace,
            EditorCommand::SearchInput,
        );
        assert_single_command!(cmds, EditorCommand::SearchExecute);
    }

    #[test]
    fn text_input_keys_backspace() {
        let cmds = handle_text_input_keys(
            KeyCode::Backspace,
            EditorCommand::ExitSearchMode,
            EditorCommand::SearchExecute,
            EditorCommand::SearchBackspace,
            EditorCommand::SearchInput,
        );
        assert_single_command!(cmds, EditorCommand::SearchBackspace);
    }

    #[test]
    fn text_input_keys_char() {
        let cmds = handle_text_input_keys(
            KeyCode::Char('x'),
            EditorCommand::ExitSearchMode,
            EditorCommand::SearchExecute,
            EditorCommand::SearchBackspace,
            |ch| EditorCommand::SearchInput(ch),
        );
        assert_single_command!(cmds, EditorCommand::SearchInput('x'));
    }

    #[test]
    fn text_input_keys_unrecognized() {
        let cmds = handle_text_input_keys(
            KeyCode::F(1),
            EditorCommand::ExitSearchMode,
            EditorCommand::SearchExecute,
            EditorCommand::SearchBackspace,
            EditorCommand::SearchInput,
        );
        assert!(cmds.is_empty());
    }

    // --- handle_list_navigation_keys ---

    #[test]
    fn list_nav_esc_returns_cancel() {
        let cmds = handle_list_navigation_keys(
            KeyCode::Esc,
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_single_command!(cmds, EditorCommand::LocationCancel);
    }

    #[test]
    fn list_nav_up_down() {
        let up = handle_list_navigation_keys(
            KeyCode::Up,
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_single_command!(up, EditorCommand::LocationUp);

        let down = handle_list_navigation_keys(
            KeyCode::Down,
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_single_command!(down, EditorCommand::LocationDown);
    }

    #[test]
    fn list_nav_jk_without_filter() {
        let k = handle_list_navigation_keys(
            KeyCode::Char('k'),
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_single_command!(k, EditorCommand::LocationUp);

        let j = handle_list_navigation_keys(
            KeyCode::Char('j'),
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_single_command!(j, EditorCommand::LocationDown);
    }

    #[test]
    fn list_nav_char_with_filter() {
        let filter_fn = |ch: char| EditorCommand::CodeActionFilterChar(ch);
        let cmds = handle_list_navigation_keys(
            KeyCode::Char('x'),
            EditorCommand::CodeActionCancel,
            EditorCommand::CodeActionUp,
            EditorCommand::CodeActionDown,
            EditorCommand::CodeActionConfirm,
            Some(EditorCommand::CodeActionFilterBackspace),
            Some(&filter_fn),
        );
        assert_single_command!(cmds, EditorCommand::CodeActionFilterChar('x'));
    }

    #[test]
    fn list_nav_enter() {
        let cmds = handle_list_navigation_keys(
            KeyCode::Enter,
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_single_command!(cmds, EditorCommand::LocationConfirm);
    }
}
