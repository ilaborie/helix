//! Status line component.
//!
//! Displays mode, file name, cursor position, and other editor state.

use dioxus::prelude::*;

use crate::AppState;

/// Status line component that shows editor state.
#[component]
pub fn StatusLine(version: usize) -> Element {
    let app_state = use_context::<AppState>();
    let snapshot = app_state.get_snapshot();

    let mode = &snapshot.mode;
    let file_name = &snapshot.file_name;
    let line = snapshot.cursor_line;
    let col = snapshot.cursor_col;
    let total_lines = snapshot.total_lines;

    // Mode-specific colors
    let (mode_bg, mode_fg) = match mode.as_str() {
        "INSERT" => ("#98c379", "#282c34"), // Green
        "SELECT" => ("#c678dd", "#282c34"), // Purple
        _ => ("#61afef", "#282c34"),        // Blue for Normal
    };

    let modified_indicator = if snapshot.is_modified { " [+]" } else { "" };

    let percentage = if total_lines > 0 {
        (line * 100) / total_lines
    } else {
        0
    };

    rsx! {
        div {
            class: "statusline",
            style: "
                display: flex;
                align-items: center;
                height: 24px;
                background-color: #21252b;
                border-top: 1px solid #181a1f;
                font-size: 12px;
                user-select: none;
            ",

            // Mode indicator
            div {
                class: "mode",
                style: "
                    padding: 0 12px;
                    height: 100%;
                    display: flex;
                    align-items: center;
                    background-color: {mode_bg};
                    color: {mode_fg};
                    font-weight: 600;
                ",
                "{mode}"
            }

            // File name
            div {
                class: "filename",
                style: "
                    padding: 0 12px;
                    color: #abb2bf;
                ",
                "{file_name}{modified_indicator}"
            }

            // Spacer
            div {
                style: "flex: 1;",
            }

            // Position info
            div {
                class: "position",
                style: "
                    padding: 0 12px;
                    color: #5c6370;
                ",
                "{line}:{col}"
            }

            // Line count / percentage
            div {
                class: "lines",
                style: "
                    padding: 0 12px;
                    color: #5c6370;
                ",
                "{percentage}% of {total_lines}L"
            }
        }
    }
}
