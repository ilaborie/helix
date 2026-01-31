//! Status line component.
//!
//! Displays mode, file name, cursor position, and other editor state.

use dioxus::prelude::*;

use crate::AppState;

/// Status line component that shows editor state.
#[component]
pub fn StatusLine(version: ReadSignal<usize>) -> Element {
    // Read the signal to subscribe to changes
    let _ = version();

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

            // Mode indicator (dynamic colors)
            div {
                class: "statusline-mode",
                style: "background-color: {mode_bg}; color: {mode_fg};",
                "{mode}"
            }

            // File name
            div {
                class: "statusline-filename",
                "{file_name}{modified_indicator}"
            }

            // Spacer
            div {
                class: "statusline-spacer",
            }

            // Position info
            div {
                class: "statusline-position",
                "{line}:{col}"
            }

            // Line count / percentage
            div {
                class: "statusline-position",
                "{percentage}% of {total_lines}L"
            }
        }
    }
}
