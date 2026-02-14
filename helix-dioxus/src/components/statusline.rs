//! Status line component.
//!
//! Displays mode, file name, cursor position, and other editor state.

use dioxus::prelude::*;
use lucide_dioxus::{CircleCheck, CircleX, LoaderCircle, Plug, TriangleAlert};

use crate::lsp::{LspServerSnapshot, LspServerStatus};
use crate::state::EditorCommand;
use crate::AppState;

/// Determine the aggregate status for display.
fn aggregate_lsp_status(servers: &[LspServerSnapshot]) -> LspServerStatus {
    // Priority: Starting > Indexing > Running > Stopped
    // If any server is starting, show starting
    // If any server is indexing, show indexing
    // If all are running, show running
    // Otherwise show stopped
    let mut has_starting = false;
    let mut has_indexing = false;
    let mut has_running = false;

    for server in servers {
        match server.status {
            LspServerStatus::Starting => has_starting = true,
            LspServerStatus::Indexing => has_indexing = true,
            LspServerStatus::Running => has_running = true,
            LspServerStatus::Stopped => {}
        }
    }

    if has_starting {
        LspServerStatus::Starting
    } else if has_indexing {
        LspServerStatus::Indexing
    } else if has_running {
        LspServerStatus::Running
    } else {
        LspServerStatus::Stopped
    }
}

/// LSP status block component for the status line.
#[component]
fn LspStatusBlock(servers: Vec<LspServerSnapshot>, on_click: EventHandler<MouseEvent>) -> Element {
    let server_count = servers.len();

    // Don't show anything if no servers are connected
    if server_count == 0 {
        return rsx! {};
    }

    // Determine overall status
    let status = aggregate_lsp_status(&servers);
    let color = status.css_color();

    rsx! {
        div {
            class: "statusline-lsp",
            style: "color: {color}; cursor: pointer;",
            onmousedown: move |evt| {
                evt.stop_propagation();
                on_click.call(evt);
            },
            span {
                class: "icon-wrapper",
                style: "margin-right: 4px;",
                Plug { size: 14, color: color }
            }
            span {
                class: match status {
                    LspServerStatus::Starting => "icon-wrapper lsp-icon-blinking",
                    LspServerStatus::Indexing => "icon-wrapper lsp-icon-spinning",
                    _ => "icon-wrapper",
                },
                style: "margin-right: 4px;",
                match status {
                    LspServerStatus::Running => rsx! { CircleCheck { size: 12, color: color } },
                    LspServerStatus::Starting | LspServerStatus::Indexing => rsx! { LoaderCircle { size: 12, color: color } },
                    LspServerStatus::Stopped => rsx! { CircleX { size: 12, color: color } },
                }
            }
            "{server_count}"
        }
    }
}

/// Status line component that shows editor state.
#[component]
pub fn StatusLine(version: ReadSignal<usize>, on_change: EventHandler<()>) -> Element {
    // Read the signal to subscribe to changes
    let _ = version();

    let app_state = use_context::<AppState>();
    let snapshot = app_state.get_snapshot();

    let mode = &snapshot.mode;
    let file_name = &snapshot.file_name;
    let line = snapshot.cursor_line;
    let col = snapshot.cursor_col;
    let total_lines = snapshot.total_lines;
    let error_count = snapshot.error_count;
    let warning_count = snapshot.warning_count;
    let lsp_servers = snapshot.lsp_servers.clone();
    let selected_register = snapshot.selected_register;

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

            // Diagnostic counts (if any)
            if error_count > 0 || warning_count > 0 {
                div {
                    class: "statusline-diagnostics",
                    if error_count > 0 {
                        span {
                            class: "statusline-errors",
                            style: "color: #e06c75; display: flex; align-items: center;",
                            span {
                                class: "icon-wrapper",
                                style: "margin-right: 4px;",
                                CircleX { size: 14, color: "#e06c75" }
                            }
                            "{error_count}"
                        }
                    }
                    if error_count > 0 && warning_count > 0 {
                        span { " " }
                    }
                    if warning_count > 0 {
                        span {
                            class: "statusline-warnings",
                            style: "color: #e5c07b; display: flex; align-items: center;",
                            span {
                                class: "icon-wrapper",
                                style: "margin-right: 4px;",
                                TriangleAlert { size: 14, color: "#e5c07b" }
                            }
                            "{warning_count}"
                        }
                    }
                }
            }

            // LSP status indicator
            LspStatusBlock {
                servers: lsp_servers,
                on_click: move |_| {
                    app_state.send_command(EditorCommand::ToggleLspDialog);
                    app_state.process_commands_sync();
                    on_change.call(());
                },
            }

            // Spacer
            div {
                class: "statusline-spacer",
            }

            // Selected register indicator
            if let Some(reg) = selected_register {
                div {
                    class: "statusline-register",
                    style: "color: var(--orange); padding: 0 6px;",
                    "reg={reg}"
                }
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
