//! Hover popup component.
//!
//! Displays documentation and type information on hover.

use dioxus::prelude::*;

use super::inline_dialog::{DialogConstraints, DialogPosition, InlineDialogContainer};
use crate::lsp::HoverSnapshot;

/// Hover popup that displays documentation and type info.
#[component]
pub fn HoverPopup(hover: HoverSnapshot, cursor_line: usize, cursor_col: usize) -> Element {
    let constraints = DialogConstraints {
        min_width: None,
        max_width: Some(600),
        max_height: Some(400),
    };

    rsx! {
        InlineDialogContainer {
            cursor_line,
            cursor_col,
            position: DialogPosition::Above,
            class: "hover-popup",
            constraints,

            // Render the hover content
            // For now, just render as pre-formatted text
            // TODO: Add proper markdown rendering
            pre {
                style: "margin: 0; white-space: pre-wrap; word-wrap: break-word;",
                "{hover.contents}"
            }
        }
    }
}
