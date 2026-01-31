//! Keybinding handlers for different editor modes.
//!
//! This module contains handlers that translate keyboard events into editor commands
//! for each editor mode (normal, insert, select, command, picker, search).

mod command;
mod insert;
mod normal;
mod picker;
mod search;
mod select;

pub use command::handle_command_mode;
pub use insert::handle_insert_mode;
pub use normal::handle_normal_mode;
pub use picker::handle_picker_mode;
pub use search::handle_search_mode;
pub use select::handle_select_mode;
