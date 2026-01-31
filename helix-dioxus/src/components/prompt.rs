//! Command and search prompt components.
//!
//! Displays the command/search input at the bottom of the screen.

use dioxus::prelude::*;

/// Command prompt component that displays the command input.
#[component]
pub fn CommandPrompt(input: String) -> Element {
    rsx! {
        div {
            class: "prompt",

            // Colon prefix
            span {
                style: "color: #61afef;",
                ":"
            }

            // Input text
            span { "{input}" }

            // Cursor
            span {
                class: "prompt-cursor prompt-cursor-command",
            }
        }
    }
}

/// Search prompt component that displays the search input.
#[component]
pub fn SearchPrompt(input: String, backwards: bool) -> Element {
    let prefix = if backwards { "?" } else { "/" };

    rsx! {
        div {
            class: "prompt",

            // Search prefix (/ or ?)
            span {
                style: "color: #e5c07b;",
                "{prefix}"
            }

            // Input text
            span { "{input}" }

            // Cursor
            span {
                class: "prompt-cursor prompt-cursor-search",
            }
        }
    }
}
