//! Generic inline dialog components.
//!
//! Provides reusable components for cursor-positioned popups like:
//! - Completion menus
//! - Code actions menus
//! - Hover information
//! - Signature help
//!
//! # Architecture
//!
//! - [`InlineDialogContainer`]: Base container handling positioning and styling
//! - [`InlineListDialog`]: List-based dialog with selection support
//!
//! # Usage
//!
//! ```rust,ignore
//! // Simple content dialog
//! InlineDialogContainer {
//!     cursor_line: 10,
//!     cursor_col: 5,
//!     position: DialogPosition::Below,
//!     // content as children
//! }
//!
//! // List dialog with selection
//! InlineListDialog {
//!     cursor_line: 10,
//!     cursor_col: 5,
//!     selected: 0,
//!     empty_message: "No items",
//!     // InlineListItem children
//! }
//! ```

mod container;
mod list;

pub use container::{DialogConstraints, DialogPosition, InlineDialogContainer};
pub use list::{InlineListDialog, InlineListItem};
