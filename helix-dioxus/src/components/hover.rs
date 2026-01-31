//! Hover popup component.
//!
//! Displays documentation and type information on hover.

use dioxus::prelude::*;

use crate::lsp::HoverSnapshot;

/// Hover popup that displays documentation and type info.
#[component]
pub fn HoverPopup(hover: HoverSnapshot, cursor_line: usize, cursor_col: usize) -> Element {
    // Position the popup above the cursor
    // TODO: Calculate actual pixel position based on cursor
    let top = cursor_line.saturating_sub(1) * 21 + 40; // Above cursor line
    let left = cursor_col * 8 + 60;

    let style = format!("top: {}px; left: {}px;", top.max(40), left.min(500));

    rsx! {
        div {
            class: "hover-popup",
            style: "{style}",

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
