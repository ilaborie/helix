//! Buffer bar component for displaying open buffers as tabs.
//!
//! Shows a horizontal tab bar with overflow scroll buttons when needed.

use dioxus::prelude::*;
use lucide_dioxus::{ChevronLeft, ChevronRight, X};

use crate::hooks::use_editor_snapshot;
use crate::state::{BufferInfo, EditorCommand};
use crate::AppState;

use super::file_icons::FileTypeIcon;

/// Maximum number of visible tabs before scrolling is needed.
const MAX_VISIBLE_TABS: usize = 8;

/// Buffer bar component that displays open buffers as clickable tabs.
#[component]
pub fn BufferBar(version: ReadSignal<usize>, on_change: EventHandler<()>) -> Element {
    let (app_state, snapshot) = use_editor_snapshot(version);

    let buffers = &snapshot.open_buffers;
    let scroll_offset = snapshot.buffer_scroll_offset;

    // Only show if there are buffers
    if buffers.is_empty() {
        return rsx! {};
    }

    // Calculate visible range (auto-scroll to current buffer is handled in state.rs)
    let visible_start = scroll_offset.min(buffers.len().saturating_sub(1));
    let visible_end = (visible_start + MAX_VISIBLE_TABS).min(buffers.len());
    let visible_buffers: Vec<&BufferInfo> = buffers
        .get(visible_start..visible_end)
        .unwrap_or(&[])
        .iter()
        .collect();

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
                        app_state_left.process_commands_sync();
                        on_change.call(());
                    },
                }
            }

            // Buffer tabs
            div {
                class: "buffer-tabs",
                for buffer in visible_buffers {
                    BufferTab {
                        key: "{buffer.id:?}",
                        buffer: buffer.clone(),
                        on_action: move |_| {
                            on_change.call(());
                        },
                    }
                }
            }

            // Right scroll button
            if needs_right_scroll {
                ScrollButton {
                    direction: "right",
                    onclick: move |_| {
                        app_state_right.send_command(EditorCommand::BufferBarScrollRight);
                        app_state_right.process_commands_sync();
                        on_change.call(());
                    },
                }
            }
        }
    }
}

/// Individual buffer tab component.
#[component]
fn BufferTab(buffer: BufferInfo, on_action: EventHandler<()>) -> Element {
    let app_state = use_context::<AppState>();
    let app_state_switch = app_state.clone();
    let app_state_close = app_state.clone();

    let on_action_switch = on_action.clone();
    let on_action_close = on_action.clone();

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
                app_state_switch.send_command(EditorCommand::SwitchToBuffer(doc_id));
                app_state_switch.process_commands_sync();
                on_action_switch.call(());
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
                    app_state_close.process_commands_sync();
                    on_action_close.call(());
                },
                span {
                    class: "icon-wrapper",
                    X { size: 12, color: text_color }
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
                    ChevronLeft { size: 16, color: "currentColor" }
                } else {
                    ChevronRight { size: 16, color: "currentColor" }
                }
            }
        }
    }
}
