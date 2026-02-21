//! Hover popup component.
//!
//! Displays documentation and type information on hover.

use dioxus::prelude::*;

use crate::components::inline_dialog::{DialogConstraints, DialogPosition, InlineDialogContainer};

/// Hover popup that displays documentation and type info.
#[component]
pub fn HoverPopup(hover_html: String) -> Element {
    let constraints = DialogConstraints {
        min_width: None,
        max_width: Some(600),
        max_height: Some(400),
    };

    rsx! {
        InlineDialogContainer {
            position: DialogPosition::Above,
            class: "hover-popup",
            constraints,

            div {
                class: "hover-markdown",
                dangerous_inner_html: "{hover_html}",
            }
        }
    }
}
