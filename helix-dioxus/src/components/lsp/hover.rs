//! Hover popup component.
//!
//! Displays documentation and type information on hover.

use dioxus::prelude::*;

use super::markdown::markdown_to_html;
use crate::components::inline_dialog::{DialogConstraints, DialogPosition, InlineDialogContainer};
use crate::lsp::HoverSnapshot;

/// Hover popup that displays documentation and type info.
#[component]
pub fn HoverPopup(hover: HoverSnapshot, cursor_line: usize, cursor_col: usize) -> Element {
    let constraints = DialogConstraints {
        min_width: None,
        max_width: Some(600),
        max_height: Some(400),
    };

    let html = markdown_to_html(&hover.contents);

    rsx! {
        InlineDialogContainer {
            cursor_line,
            cursor_col,
            position: DialogPosition::Above,
            class: "hover-popup",
            constraints,

            div {
                class: "hover-markdown",
                dangerous_inner_html: "{html}",
            }
        }
    }
}
