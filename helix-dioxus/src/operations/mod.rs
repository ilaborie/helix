//! Editor operations organized by functionality.
//!
//! This module provides extension traits that add operation methods to `EditorContext`.
//! Operations are grouped by category for better organization and maintainability.

mod buffer;
mod cli;
mod clipboard;
mod editing;
mod movement;
mod picker_ops;
mod search;
mod selection;

pub use buffer::BufferOps;
pub use cli::CliOps;
pub use clipboard::ClipboardOps;
pub use editing::EditingOps;
pub use movement::MovementOps;
pub use picker_ops::PickerOps;
pub use search::SearchOps;
pub use selection::SelectionOps;
