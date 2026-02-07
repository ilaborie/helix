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
    handle_space_leader,
};
pub use picker::handle_picker_mode;
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
