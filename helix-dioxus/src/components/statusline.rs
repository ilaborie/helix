//! Status line component.
//!
//! Displays mode, file name, cursor position, and other editor state.

use dioxus::prelude::*;
use lucide_dioxus::{CircleCheck, CircleX, LoaderCircle, Plug, TriangleAlert};

use crate::hooks::use_editor_snapshot;
use crate::lsp::{LspServerSnapshot, LspServerStatus};
use crate::state::EditorCommand;

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
                Plug { size: 14, color: "currentColor" }
            }
            span {
                class: match status {
                    LspServerStatus::Starting => "icon-wrapper lsp-icon-blinking",
                    LspServerStatus::Indexing => "icon-wrapper lsp-icon-spinning",
                    _ => "icon-wrapper",
                },
                style: "margin-right: 4px;",
                match status {
                    LspServerStatus::Running => rsx! { CircleCheck { size: 12, color: "currentColor" } },
                    LspServerStatus::Starting | LspServerStatus::Indexing => rsx! { LoaderCircle { size: 12, color: "currentColor" } },
                    LspServerStatus::Stopped => rsx! { CircleX { size: 12, color: "currentColor" } },
                }
            }
            "{server_count}"
        }
    }
}

/// Status line component that shows editor state.
#[component]
pub fn StatusLine(version: ReadSignal<usize>, on_change: EventHandler<()>) -> Element {
    let (app_state, snapshot) = use_editor_snapshot(version);

    let mode = &snapshot.mode;
    let file_name = &snapshot.file_name;
    let line = snapshot.cursor_line;
    let col = snapshot.cursor_col;
    let total_lines = snapshot.total_lines;
    let error_count = snapshot.error_count;
    let warning_count = snapshot.warning_count;
    let lsp_servers = snapshot.lsp_servers.clone();
    let selected_register = snapshot.selected_register;
    let macro_recording = snapshot.macro_recording;

    // Mode-specific colors (from CSS custom properties)
    let (mode_bg, mode_fg) = match mode.as_str() {
        "INSERT" => ("var(--mode-insert-bg)", "var(--mode-insert-fg)"),
        "SELECT" => ("var(--mode-select-bg)", "var(--mode-select-fg)"),
        _ => ("var(--mode-normal-bg)", "var(--mode-normal-fg)"),
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
                            style: "color: var(--error); display: flex; align-items: center;",
                            span {
                                class: "icon-wrapper",
                                style: "margin-right: 4px;",
                                CircleX { size: 14, color: "currentColor" }
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
                            style: "color: var(--warning); display: flex; align-items: center;",
                            span {
                                class: "icon-wrapper",
                                style: "margin-right: 4px;",
                                TriangleAlert { size: 14, color: "currentColor" }
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

            // Macro recording indicator
            if let Some(reg) = macro_recording {
                div {
                    class: "statusline-recording",
                    "REC [@{reg}]"
                }
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
