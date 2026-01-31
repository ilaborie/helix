//! Document rendering component.
//!
//! Renders the document content with syntax highlighting and cursor display.

use dioxus::prelude::*;

use crate::state::LineSnapshot;
use crate::AppState;

/// Editor view component that renders the document content.
#[component]
pub fn EditorView(version: usize) -> Element {
    let app_state = use_context::<AppState>();
    let snapshot = app_state.get_snapshot();

    let mode = &snapshot.mode;

    rsx! {
        div {
            class: "editor-view",
            style: "display: flex; height: 100%; overflow: hidden; font-size: 14px; line-height: 1.5;",

            // Line numbers gutter
            div {
                class: "gutter",
                style: "
                    padding: 8px 12px 8px 8px;
                    text-align: right;
                    color: #5c6370;
                    background-color: #21252b;
                    user-select: none;
                    min-width: 40px;
                ",
                for line in &snapshot.lines {
                    div {
                        key: "{line.line_number}",
                        style: if line.is_cursor_line {
                            "color: #abb2bf;"
                        } else {
                            ""
                        },
                        "{line.line_number}"
                    }
                }
            }

            // Document content
            div {
                class: "content",
                style: "
                    flex: 1;
                    padding: 8px;
                    overflow-x: auto;
                    overflow-y: auto;
                    white-space: pre;
                ",
                for line in &snapshot.lines {
                    Line {
                        key: "{line.line_number}",
                        line: line.clone(),
                        mode: mode.clone(),
                    }
                }
            }
        }
    }
}

/// Individual line component with cursor rendering.
#[component]
fn Line(line: LineSnapshot, mode: String) -> Element {
    // Remove trailing newline for display
    let display_content = line.content.trim_end_matches('\n');

    // If this is the cursor line, we need to render with cursor
    if line.is_cursor_line {
        if let Some(cursor_col) = line.cursor_col {
            let chars: Vec<char> = display_content.chars().collect();
            let cursor_pos = cursor_col; // Already 0-indexed

            // Split content around cursor position
            let before: String = chars.iter().take(cursor_pos).collect();
            let cursor_char = chars.get(cursor_pos).copied().unwrap_or(' ');
            let after: String = chars.iter().skip(cursor_pos + 1).collect();

            let cursor_style = match mode.as_str() {
                "INSERT" => "background-color: transparent; box-shadow: -2px 0 0 0 #61afef;",
                "SELECT" => "background-color: #c678dd; color: #282c34;",
                _ => "background-color: #61afef; color: #282c34;", // Normal mode
            };

            return rsx! {
                div {
                    class: "line",
                    style: "background-color: #2c313a; min-height: 1.5em;",
                    span { "{before}" }
                    span {
                        class: "cursor",
                        style: "{cursor_style}",
                        "{cursor_char}"
                    }
                    span { "{after}" }
                }
            };
        }
    }

    rsx! {
        div {
            class: "line",
            style: "min-height: 1.5em;",
            "{display_content}"
        }
    }
}
