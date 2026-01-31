//! Keybinding handlers for different editor modes.
//!
//! This module contains handlers that translate keyboard events into editor commands
//! for each editor mode (normal, insert, select, command, picker, search).

mod command;
mod completion;
mod insert;
mod normal;
mod picker;
mod search;
mod select;

pub use command::handle_command_mode;
pub use completion::{
    handle_code_actions_mode, handle_completion_mode, handle_location_picker_mode,
};
pub use insert::handle_insert_mode;
pub use normal::{
    handle_bracket_next, handle_bracket_prev, handle_g_prefix, handle_normal_mode,
    handle_space_leader,
};
pub use picker::handle_picker_mode;
pub use search::handle_search_mode;
pub use select::handle_select_mode;
