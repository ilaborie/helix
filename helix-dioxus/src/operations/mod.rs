//! Editor operations organized by functionality.
//!
//! This module provides extension traits that add operation methods to `EditorContext`.
//! Operations are grouped by category for better organization and maintainability.

mod buffer;
mod cli;
mod clipboard;
mod editing;
mod jump;
mod lsp;
mod movement;
mod picker_ops;
mod search;
mod selection;
mod shell;
mod word_jump;

pub use buffer::BufferOps;
pub use cli::CliOps;
pub use clipboard::ClipboardOps;
pub use editing::EditingOps;
pub use jump::JumpOps;
pub use lsp::LspOps;
pub use movement::MovementOps;
pub use picker_ops::PickerOps;
pub use search::{collect_search_match_lines, SearchOps};
pub use selection::SelectionOps;
pub use shell::ShellOps;
pub use word_jump::WordJumpOps;
