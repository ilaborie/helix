//! Buffer bar component for displaying open buffers as tabs.
//!
//! Shows a horizontal tab bar with overflow scroll buttons when needed.

use crate::icons::{lucide, Icon};
use dioxus::prelude::*;
use helix_view::DocumentId;

use crate::hooks::{use_snapshot, use_snapshot_signal};
use crate::state::{BufferInfo, EditorCommand};
use crate::AppState;

use super::file_icons::FileTypeIcon;
use super::popup_menu::{PopupMenu, PopupMenuEntry};

/// Maximum number of visible tabs before scrolling is needed.
const MAX_VISIBLE_TABS: usize = 8;

/// State for the right-click context menu on a buffer tab.
struct ContextMenuState {
    x: f64,
    y: f64,
    doc_id: DocumentId,
    path: Option<String>,
    is_modified: bool,
    buffer_count: usize,
}

/// Buffer bar component that displays open buffers as clickable tabs.
#[component]
pub fn BufferBar() -> Element {
    let app_state = use_context::<AppState>();
    let snapshot = use_snapshot();
    let mut snapshot_signal = use_snapshot_signal();

    let buffers = &snapshot.open_buffers;
    let scroll_offset = snapshot.buffer_scroll_offset;

    // Context menu state (local to BufferBar)
    let mut context_menu: Signal<Option<ContextMenuState>> = use_signal(|| None);

    // Only show if there are buffers
    if buffers.is_empty() {
        return rsx! {};
    }

    // Calculate visible range (auto-scroll to current buffer is handled in state.rs)
    let visible_start = scroll_offset.min(buffers.len().saturating_sub(1));
    let visible_end = (visible_start + MAX_VISIBLE_TABS).min(buffers.len());
    let visible_buffers: Vec<&BufferInfo> = buffers.get(visible_start..visible_end).unwrap_or(&[]).iter().collect();

    // Determine if we need scroll buttons
    let needs_left_scroll = scroll_offset > 0;
    let needs_right_scroll = visible_end < buffers.len();

    // Clone for closures
    let app_state_left = app_state.clone();
    let app_state_right = app_state.clone();

    rsx! {
        div {
            class: "buffer-bar",

            // Left scroll button
            if needs_left_scroll {
                ScrollButton {
                    direction: "left",
                    onclick: move |_| {
                        app_state_left.send_command(EditorCommand::BufferBarScrollLeft);
                        app_state_left.process_and_notify(&mut snapshot_signal);
                    },
                }
            }

            // Buffer tabs
            div {
                class: "buffer-tabs",
                {
                    let buffer_count = buffers.len();
                    rsx! {
                        for buffer in visible_buffers {
                            BufferTab {
                                key: "{buffer.id:?}",
                                buffer: buffer.clone(),
                                buffer_count,
                                on_context_menu: move |state: ContextMenuState| {
                                    context_menu.set(Some(state));
                                },
                            }
                        }
                    }
                }
            }

            // Right scroll button
            if needs_right_scroll {
                ScrollButton {
                    direction: "right",
                    onclick: move |_| {
                        app_state_right.send_command(EditorCommand::BufferBarScrollRight);
                        app_state_right.process_and_notify(&mut snapshot_signal);
                    },
                }
            }
        }

        // Context menu (rendered outside buffer-bar to avoid clipping)
        if let Some(ref state) = *context_menu.read() {
            {
                let entries = build_context_menu_entries(state, &app_state, &mut snapshot_signal);
                rsx! {
                    PopupMenu {
                        x: state.x,
                        y: state.y,
                        entries,
                        on_close: move |()| context_menu.set(None),
                    }
                }
            }
        }
    }
}

/// Build the context menu entries for a buffer tab.
fn build_context_menu_entries(
    state: &ContextMenuState,
    app_state: &AppState,
    snapshot_signal: &mut Signal<crate::state::EditorSnapshot>,
) -> Vec<PopupMenuEntry> {
    let doc_id = state.doc_id;
    let has_others = state.buffer_count > 1;
    let is_modified = state.is_modified;

    let mut entries = vec![
        // Close
        PopupMenuEntry::Item {
            label: "Close",
            disabled: false,
            on_click: {
                let app_state = app_state.clone();
                let mut snapshot_signal = *snapshot_signal;
                EventHandler::new(move |_| {
                    app_state.send_command(EditorCommand::CloseBuffer(doc_id));
                    app_state.process_and_notify(&mut snapshot_signal);
                })
            },
        },
        // Close Others
        PopupMenuEntry::Item {
            label: "Close Others",
            disabled: !has_others,
            on_click: {
                let app_state = app_state.clone();
                let mut snapshot_signal = *snapshot_signal;
                EventHandler::new(move |_| {
                    app_state.send_command(EditorCommand::SwitchToBuffer(doc_id));
                    app_state.send_command(EditorCommand::BufferCloseOthers);
                    app_state.process_and_notify(&mut snapshot_signal);
                })
            },
        },
        // Close All
        PopupMenuEntry::Item {
            label: "Close All",
            disabled: false,
            on_click: {
                let app_state = app_state.clone();
                let mut snapshot_signal = *snapshot_signal;
                EventHandler::new(move |_| {
                    app_state.send_command(EditorCommand::BufferCloseAll { force: false });
                    app_state.process_and_notify(&mut snapshot_signal);
                })
            },
        },
    ];

    // Copy Path (only for buffers with a file path)
    if let Some(ref file_path) = state.path {
        entries.push(PopupMenuEntry::Separator);
        let file_path = file_path.clone();
        entries.push(PopupMenuEntry::Item {
            label: "Copy Path",
            disabled: false,
            on_click: EventHandler::new(move |_: Event<MouseData>| {
                let escaped = file_path.replace('\\', "\\\\").replace('\'', "\\'");
                document::eval(&format!("navigator.clipboard.writeText('{escaped}')"));
            }),
        });
    }

    // Save
    entries.push(PopupMenuEntry::Separator);
    entries.push(PopupMenuEntry::Item {
        label: "Save",
        disabled: !is_modified,
        on_click: {
            let app_state = app_state.clone();
            let mut snapshot_signal = *snapshot_signal;
            EventHandler::new(move |_| {
                app_state.send_command(EditorCommand::SwitchToBuffer(doc_id));
                app_state.send_command(EditorCommand::CliCommand("w".into()));
                app_state.process_and_notify(&mut snapshot_signal);
            })
        },
    });

    entries
}

/// Individual buffer tab component.
#[component]
fn BufferTab(buffer: BufferInfo, buffer_count: usize, on_context_menu: EventHandler<ContextMenuState>) -> Element {
    let app_state = use_context::<AppState>();
    let mut snapshot_signal = use_snapshot_signal();

    let app_state_close = app_state.clone();
    let doc_id = buffer.id;
    let doc_id_close = buffer.id;

    let bg_color = if buffer.is_current {
        "var(--bg-primary)"
    } else {
        "transparent"
    };

    let text_color = if buffer.is_current {
        "var(--text)"
    } else {
        "var(--text-dim)"
    };

    let border_bottom = if buffer.is_current {
        "2px solid var(--accent)"
    } else {
        "2px solid transparent"
    };

    let modified_color = "var(--warning)";

    // Clone for context menu closure
    let ctx_path = buffer.path.clone();
    let ctx_is_modified = buffer.is_modified;

    rsx! {
        div {
            class: "buffer-tab",
            // Dynamic styles for active/inactive state
            style: "background-color: {bg_color}; border-bottom: {border_bottom};",
            // Tooltip with full name for truncated tabs
            title: "{buffer.name}",
            onmousedown: move |evt| {
                evt.stop_propagation();
                log::info!("Buffer tab clicked: {doc_id:?}");
                app_state.send_command(EditorCommand::SwitchToBuffer(doc_id));
                app_state.process_and_notify(&mut snapshot_signal);
            },
            oncontextmenu: move |evt: Event<MouseData>| {
                evt.prevent_default();
                evt.stop_propagation();
                let coords = evt.client_coordinates();
                on_context_menu.call(ContextMenuState {
                    x: coords.x,
                    y: coords.y,
                    doc_id,
                    path: ctx_path.clone(),
                    is_modified: ctx_is_modified,
                    buffer_count,
                });
            },

            // Modified indicator (before file icon for visibility)
            if buffer.is_modified {
                span {
                    style: "color: {modified_color}; margin-right: 4px; font-size: 10px;",
                    "\u{25cf}"
                }
            }

            // File icon
            span {
                style: "margin-right: 6px;",
                FileTypeIcon { name: buffer.name.clone(), size: 14 }
            }

            // File name (truncated)
            span {
                class: "buffer-tab-name",
                style: "color: {text_color};",
                "{buffer.name}"
            }

            // Close button
            div {
                style: "width: 16px; height: 16px; margin-left: 4px; border-radius: 3px; opacity: 0.6; cursor: pointer; display: flex; align-items: center; justify-content: center;",
                onmousedown: move |evt| {
                    evt.stop_propagation();
                    log::info!("Close button clicked: {doc_id_close:?}");
                    app_state_close.send_command(EditorCommand::CloseBuffer(doc_id_close));
                    app_state_close.process_and_notify(&mut snapshot_signal);
                },
                span {
                    class: "icon-wrapper",
                    Icon { data: lucide::X, size: "12", fill: text_color }
                }
            }
        }
    }
}

/// Scroll button for buffer bar overflow.
#[component]
fn ScrollButton(direction: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    let is_left = direction == "left";

    rsx! {
        div {
            class: "scroll-button",
            onmousedown: move |evt| {
                evt.stop_propagation();
                log::info!("Scroll button clicked: {direction}");
                onclick.call(evt);
            },
            span {
                class: "icon-wrapper",
                if is_left {
                    Icon { data: lucide::ChevronLeft, size: "16", fill: "currentColor" }
                } else {
                    Icon { data: lucide::ChevronRight, size: "16", fill: "currentColor" }
                }
            }
        }
    }
}
