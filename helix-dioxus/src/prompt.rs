//! Command and search prompt components.
//!
//! Displays the command/search input at the bottom of the screen.

use dioxus::prelude::*;

/// Command prompt component that displays the command input.
#[component]
pub fn CommandPrompt(input: String) -> Element {
    rsx! {
        div {
            class: "command-prompt",
            style: "
                height: 24px;
                background-color: #21252b;
                border-top: 1px solid #181a1f;
                padding: 0 8px;
                display: flex;
                align-items: center;
                font-size: 14px;
                color: #abb2bf;
            ",

            // Colon prefix
            span {
                style: "color: #61afef;",
                ":"
            }

            // Input text
            span { "{input}" }

            // Cursor
            span {
                class: "cursor",
                style: "
                    display: inline-block;
                    width: 8px;
                    height: 16px;
                    background-color: #61afef;
                    animation: blink 1s step-end infinite;
                ",
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
            class: "search-prompt",
            style: "
                height: 24px;
                background-color: #21252b;
                border-top: 1px solid #181a1f;
                padding: 0 8px;
                display: flex;
                align-items: center;
                font-size: 14px;
                color: #abb2bf;
            ",

            // Search prefix (/ or ?)
            span {
                style: "color: #e5c07b;",
                "{prefix}"
            }

            // Input text
            span { "{input}" }

            // Cursor
            span {
                class: "cursor",
                style: "
                    display: inline-block;
                    width: 8px;
                    height: 16px;
                    background-color: #e5c07b;
                    animation: blink 1s step-end infinite;
                ",
            }
        }
    }
}
