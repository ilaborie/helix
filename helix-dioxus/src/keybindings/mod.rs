//! Keybinding handlers for different editor modes.
//!
//! This module contains handlers that translate keyboard events into editor commands
//! for each editor mode (normal, insert, select, command, picker, search).

mod command;
mod completion;
mod confirmation;
mod input_dialog;
mod insert;
mod normal;
mod picker;
mod regex;
mod search;
mod select;
mod translate;

use helix_view::input::KeyCode;

use crate::state::{Direction, EditorCommand};

pub use command::handle_command_mode;
pub use completion::{
    handle_code_actions_mode, handle_completion_mode, handle_location_picker_mode,
    handle_lsp_dialog_mode,
};
pub use confirmation::handle_confirmation_mode;
pub use input_dialog::handle_input_dialog_mode;
pub use insert::handle_insert_mode;
pub use normal::{
    handle_bracket_next, handle_bracket_prev, handle_g_prefix, handle_normal_mode,
    handle_space_leader, handle_view_prefix,
};
pub use picker::handle_picker_mode;
pub use regex::handle_regex_mode;
pub use search::handle_search_mode;
pub use select::handle_select_mode;
pub use translate::translate_key_event;

/// Map direction keys (hjkl + arrows) to a `Direction`.
fn direction_from_key(code: KeyCode) -> Option<Direction> {
    match code {
        KeyCode::Char('h') | KeyCode::Left => Some(Direction::Left),
        KeyCode::Char('l') | KeyCode::Right => Some(Direction::Right),
        KeyCode::Char('j') | KeyCode::Down => Some(Direction::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Direction::Up),
        _ => None,
    }
}

/// Map a direction to a movement command (for normal/insert mode).
fn move_command(direction: Direction) -> EditorCommand {
    match direction {
        Direction::Left => EditorCommand::MoveLeft,
        Direction::Right => EditorCommand::MoveRight,
        Direction::Down => EditorCommand::MoveDown,
        Direction::Up => EditorCommand::MoveUp,
    }
}

/// Map a direction to a selection-extend command (for select mode).
fn extend_command(direction: Direction) -> EditorCommand {
    match direction {
        Direction::Left => EditorCommand::ExtendLeft,
        Direction::Right => EditorCommand::ExtendRight,
        Direction::Down => EditorCommand::ExtendDown,
        Direction::Up => EditorCommand::ExtendUp,
    }
}

/// Handle direction keys and return the appropriate move command, if matched.
fn handle_move_keys(code: KeyCode) -> Option<Vec<EditorCommand>> {
    direction_from_key(code).map(|dir| vec![move_command(dir)])
}

/// Handle direction keys and return the appropriate extend command, if matched.
fn handle_extend_keys(code: KeyCode) -> Option<Vec<EditorCommand>> {
    direction_from_key(code).map(|dir| vec![extend_command(dir)])
}

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
    use super::*;

    // --- direction_from_key ---

    #[test]
    fn direction_from_key_hjkl() {
        assert!(matches!(direction_from_key(KeyCode::Char('h')), Some(Direction::Left)));
        assert!(matches!(direction_from_key(KeyCode::Char('j')), Some(Direction::Down)));
        assert!(matches!(direction_from_key(KeyCode::Char('k')), Some(Direction::Up)));
        assert!(matches!(direction_from_key(KeyCode::Char('l')), Some(Direction::Right)));
    }

    #[test]
    fn direction_from_key_arrows() {
        assert!(matches!(direction_from_key(KeyCode::Left), Some(Direction::Left)));
        assert!(matches!(direction_from_key(KeyCode::Right), Some(Direction::Right)));
        assert!(matches!(direction_from_key(KeyCode::Up), Some(Direction::Up)));
        assert!(matches!(direction_from_key(KeyCode::Down), Some(Direction::Down)));
    }

    #[test]
    fn direction_from_key_unrecognized() {
        assert!(direction_from_key(KeyCode::Char('a')).is_none());
        assert!(direction_from_key(KeyCode::Enter).is_none());
        assert!(direction_from_key(KeyCode::Esc).is_none());
    }

    // --- move_command ---

    #[test]
    fn move_command_maps_directions() {
        assert!(matches!(move_command(Direction::Left), EditorCommand::MoveLeft));
        assert!(matches!(move_command(Direction::Right), EditorCommand::MoveRight));
        assert!(matches!(move_command(Direction::Up), EditorCommand::MoveUp));
        assert!(matches!(move_command(Direction::Down), EditorCommand::MoveDown));
    }

    // --- extend_command ---

    #[test]
    fn extend_command_maps_directions() {
        assert!(matches!(extend_command(Direction::Left), EditorCommand::ExtendLeft));
        assert!(matches!(extend_command(Direction::Right), EditorCommand::ExtendRight));
        assert!(matches!(extend_command(Direction::Up), EditorCommand::ExtendUp));
        assert!(matches!(extend_command(Direction::Down), EditorCommand::ExtendDown));
    }

    // --- handle_move_keys ---

    #[test]
    fn handle_move_keys_returns_command_for_hjkl() {
        let cmds = handle_move_keys(KeyCode::Char('h')).expect("should match 'h'");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::MoveLeft));
    }

    #[test]
    fn handle_move_keys_returns_none_for_unrecognized() {
        assert!(handle_move_keys(KeyCode::Char('a')).is_none());
        assert!(handle_move_keys(KeyCode::Esc).is_none());
    }

    // --- handle_extend_keys ---

    #[test]
    fn handle_extend_keys_returns_command_for_arrows() {
        let cmds = handle_extend_keys(KeyCode::Right).expect("should match Right");
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExtendRight));
    }

    #[test]
    fn handle_extend_keys_returns_none_for_unrecognized() {
        assert!(handle_extend_keys(KeyCode::Enter).is_none());
    }

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
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::ExitSearchMode));
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
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SearchExecute));
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
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SearchBackspace));
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
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::SearchInput('x')));
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
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::LocationCancel));
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
        assert_eq!(up.len(), 1);
        assert!(matches!(up[0], EditorCommand::LocationUp));

        let down = handle_list_navigation_keys(
            KeyCode::Down,
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_eq!(down.len(), 1);
        assert!(matches!(down[0], EditorCommand::LocationDown));
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
        assert_eq!(k.len(), 1);
        assert!(matches!(k[0], EditorCommand::LocationUp));

        let j = handle_list_navigation_keys(
            KeyCode::Char('j'),
            EditorCommand::LocationCancel,
            EditorCommand::LocationUp,
            EditorCommand::LocationDown,
            EditorCommand::LocationConfirm,
            None,
            None,
        );
        assert_eq!(j.len(), 1);
        assert!(matches!(j[0], EditorCommand::LocationDown));
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
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::CodeActionFilterChar('x')));
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
        assert_eq!(cmds.len(), 1);
        assert!(matches!(cmds[0], EditorCommand::LocationConfirm));
    }
}
