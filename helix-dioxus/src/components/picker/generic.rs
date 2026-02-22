//! Generic picker component.
//!
//! Supports files, directories, and buffers with fuzzy match highlighting.
//! When a preview is available, displays a two-column layout with the item
//! list on the left and a syntax-highlighted file preview on the right.

use crate::icons::{lucide, Icon};
use dioxus::prelude::*;

use crate::components::{KbdKey, ModalOverlay};
use crate::config::DialogSearchMode;
use crate::hooks::use_snapshot_signal;
use crate::state::{EditorCommand, PickerItem, PickerMode, PickerPreview, PICKER_WINDOW_SIZE};
use crate::AppState;

use super::item::PickerItemRow;
use super::preview::PickerPreviewPanel;
use super::scrollbar::PickerScrollbar;

/// Generic picker component that displays items with filtering and highlighting.
#[component]
pub fn GenericPicker(
    /// Pre-windowed items (already sliced to the visible window).
    items: Vec<PickerItem>,
    selected: usize,
    filter: String,
    total: usize,
    /// Number of items after filtering (may differ from `items.len()` due to windowing).
    filtered_count: usize,
    /// Start index of the windowed items in the full filtered list.
    window_offset: usize,
    mode: PickerMode,
    current_path: Option<String>,
    preview: Option<PickerPreview>,
    search_mode: DialogSearchMode,
    search_focused: bool,
) -> Element {
    let app_state = use_context::<AppState>();
    let mut snapshot_signal = use_snapshot_signal();

    // Items are pre-windowed — compute absolute indices using offset
    let visible_items: Vec<(usize, &PickerItem)> = items
        .iter()
        .enumerate()
        .map(|(i, item)| (window_offset + i, item))
        .collect();

    // Scroll selected item into view when selection changes.
    // Bridge prop → signal so use_effect re-runs when the value changes.
    let mut scroll_target = use_signal(|| selected);
    if *scroll_target.peek() != selected {
        scroll_target.set(selected);
    }
    use_effect(move || {
        scroll_target.read();
        document::eval("scrollSelectedPickerItem();");
    });

    let title = mode.title();

    // Use wide layout whenever the mode supports preview (even if current item has none),
    // so the container doesn't resize when navigating between folders and files.
    let wide_layout = mode.supports_preview();
    let container_class = if wide_layout {
        "picker-container picker-container-with-preview"
    } else {
        "picker-container"
    };

    rsx! {
        ModalOverlay {
            class: container_class,
            align_top: true,
            on_backdrop_click: EventHandler::new(|_| {}),

            // Header with title (spans full width, above body)
            div {
                class: "picker-header",

                // Title
                div {
                    class: "picker-title",
                    "{title}"
                }

                // Search input display
                {
                    let is_vim = search_mode == DialogSearchMode::VimStyle;
                    let search_class = if is_vim && search_focused {
                        "picker-search picker-search-focused"
                    } else {
                        "picker-search"
                    };
                    let placeholder = if is_vim && !search_focused {
                        "Press / to search..."
                    } else {
                        "Type to filter..."
                    };
                    rsx! {
                        div {
                            class: "{search_class}",
                            span {
                                class: "icon-wrapper",
                                style: "width: 16px; height: 16px; margin-right: 8px; color: var(--text-dim);",
                                Icon { data: lucide::Search, size: "16", fill: "currentColor" }
                            }
                            span {
                                class: "picker-search-input",
                                if filter.is_empty() {
                                    span {
                                        class: "picker-search-placeholder",
                                        "{placeholder}"
                                    }
                                } else {
                                    "{filter}"
                                }
                                if is_vim && search_focused {
                                    span { class: "prompt-cursor prompt-cursor-search" }
                                }
                            }
                        }
                    }
                }

                // Current path (for directory browser / file explorer)
                if let Some(ref path) = current_path {
                    if matches!(mode, PickerMode::DirectoryBrowser | PickerMode::FileExplorer) {
                        div {
                            class: "picker-path",
                            "{path}"
                        }
                    }
                }

                // Help text row with count
                div {
                    class: "picker-help-row",

                    // Left: help text with kbd elements
                    span {
                        class: "picker-help-text",
                        if search_mode == DialogSearchMode::VimStyle {
                            KbdKey { label: "j/k" }
                            " navigate \u{2022} "
                            KbdKey { label: "/" }
                            " search \u{2022} "
                            KbdKey { label: "Enter" }
                            {mode.enter_hint()}
                            KbdKey { label: "Esc" }
                            " cancel"
                        } else {
                            KbdKey { label: "\u{2191}\u{2193}" }
                            " navigate \u{2022} "
                            KbdKey { label: "Enter" }
                            {mode.enter_hint()}
                            KbdKey { label: "Esc" }
                            " cancel"
                        }
                    }

                    // Right: count
                    span {
                        class: "picker-help-text",
                        "{filtered_count} / {total}"
                    }
                }
            }

            // Body: two-column when preview, single-column otherwise
            div {
                class: "picker-body",

                // Left column: item list
                div {
                    class: if wide_layout { "picker-left" } else { "picker-left picker-left-full" },

                    // Item list
                    div {
                        class: "picker-list",

                        div {
                            class: "picker-list-items",

                            if items.is_empty() {
                                div {
                                    class: "picker-empty",
                                    if mode == PickerMode::GlobalSearch {
                                        if filter.is_empty() {
                                            "Type a pattern and press Enter to search"
                                        } else {
                                            "Press Enter to search"
                                        }
                                    } else if filter.is_empty() {
                                        "No items"
                                    } else {
                                        "No matches found"
                                    }
                                }
                            } else {
                                for (idx, item) in visible_items {
                                    {
                                        let item_app_state = app_state.clone();
                                        let handle_click = move |evt: MouseEvent| {
                                            evt.stop_propagation();
                                            item_app_state.send_command(EditorCommand::PickerConfirmItem(idx));
                                            item_app_state.process_and_notify(&mut snapshot_signal);
                                        };
                                        rsx! {
                                            PickerItemRow {
                                                key: "{idx}",
                                                item: item.clone(),
                                                is_selected: idx == selected,
                                                on_click: handle_click,
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        PickerScrollbar {
                            visible_count: PICKER_WINDOW_SIZE,
                            window_offset,
                            filtered_count,
                        }
                    }
                }

                // Right column: preview panel or placeholder
                if wide_layout {
                    if let Some(preview_data) = preview {
                        PickerPreviewPanel { preview: preview_data }
                    } else {
                        div {
                            class: "picker-preview-placeholder",
                            "No preview available"
                        }
                    }
                }
            }
        }
    }
}
