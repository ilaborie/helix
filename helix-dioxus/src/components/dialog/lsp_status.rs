//! LSP status dialog component.
//!
//! Displays a modal dialog with a list of active language servers,
//! their status, and actions to restart them.

use crate::icons::{lucide, Icon};
use dioxus::prelude::*;

use crate::components::KbdKey;
use crate::lsp::{LspServerSnapshot, LspServerStatus};
use crate::state::EditorCommand;
use crate::AppState;

/// LSP status dialog component.
#[component]
pub fn LspStatusDialog(servers: Vec<LspServerSnapshot>, selected: usize, on_change: EventHandler<()>) -> Element {
    let app_state = use_context::<AppState>();

    rsx! {
        // Overlay
        div {
            class: "lsp-dialog-overlay",
            onmousedown: {
                let app_state = app_state.clone();
                move |_| {
                    app_state.send_command(EditorCommand::CloseLspDialog);
                    app_state.process_commands_sync();
                    on_change.call(());
                }
            },

            // Dialog container
            div {
                class: "lsp-dialog-container",
                onmousedown: move |evt| evt.stop_propagation(),

                // Header
                div {
                    class: "lsp-dialog-header",
                    "Language Servers"
                    span {
                        class: "lsp-dialog-count",
                        " ({servers.len()})"
                    }
                }

                // Server list
                div {
                    class: "lsp-dialog-list",
                    if servers.is_empty() {
                        div {
                            class: "lsp-dialog-empty",
                            "No language servers connected"
                        }
                    } else {
                        for (idx, server) in servers.iter().enumerate() {
                            LspServerRow {
                                server: server.clone(),
                                is_selected: idx == selected,
                                on_restart: {
                                    let app_state = app_state.clone();
                                    let name = server.name.clone();
                                    move |_| {
                                        app_state.send_command(EditorCommand::RestartLspServer(name.clone()));
                                        app_state.process_commands_sync();
                                        on_change.call(());
                                    }
                                },
                            }
                        }
                    }
                }

                // Help row
                div {
                    class: "lsp-dialog-help",
                    span {
                        KbdKey { label: "j" }
                        KbdKey { label: "k" }
                        " navigate"
                    }
                    span {
                        KbdKey { label: "r" }
                        " restart"
                    }
                    span {
                        KbdKey { label: "Esc" }
                        " close"
                    }
                }
            }
        }
    }
}

/// A single row in the LSP server list.
#[component]
fn LspServerRow(server: LspServerSnapshot, is_selected: bool, on_restart: EventHandler<MouseEvent>) -> Element {
    let status_color = server.status.css_color();
    let row_class = if is_selected {
        "lsp-server-row lsp-server-row-selected"
    } else {
        "lsp-server-row"
    };

    rsx! {
        div {
            class: "{row_class}",

            // Status icon
            span {
                class: match server.status {
                    LspServerStatus::Starting => "lsp-server-status icon-wrapper lsp-icon-blinking",
                    LspServerStatus::Indexing => "lsp-server-status icon-wrapper lsp-icon-spinning",
                    _ => "lsp-server-status icon-wrapper",
                },
                match server.status {
                    LspServerStatus::Running => rsx! { Icon { data: lucide::CircleCheck, size: "14", fill: status_color } },
                    LspServerStatus::Starting => rsx! { Icon { data: lucide::LoaderCircle, size: "14", fill: status_color } },
                    LspServerStatus::Indexing => rsx! { Icon { data: lucide::LoaderCircle, size: "14", fill: status_color } },
                    LspServerStatus::Stopped => rsx! { Icon { data: lucide::CircleX, size: "14", fill: status_color } },
                }
            }

            // Server info (name + progress message)
            div {
                class: "lsp-server-info",

                // Server name
                span {
                    class: "lsp-server-name",
                    "{server.name}"
                }

                // Progress message (if indexing)
                if let Some(ref msg) = server.progress_message {
                    span {
                        class: "lsp-server-progress",
                        style: "color: {status_color};",
                        "{msg}"
                    }
                }
            }

            // Languages badge (if any)
            if !server.languages.is_empty() {
                span {
                    class: "lsp-server-languages",
                    "{server.languages.join(\", \")}"
                }
            }

            // Active indicator for current document
            if server.active_for_current {
                span {
                    class: "lsp-server-active icon-wrapper",
                    title: "Active for current document",
                    Icon { data: lucide::Circle, size: "10", fill: "currentColor" }
                }
            }

            // Restart button
            button {
                class: "lsp-restart-btn",
                title: "Restart server",
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    on_restart.call(evt);
                },
                span {
                    class: "icon-wrapper",
                    Icon { data: lucide::RefreshCw, size: "14", fill: "currentColor" }
                }
            }
        }
    }
}
