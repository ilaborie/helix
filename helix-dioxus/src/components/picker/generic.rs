//! Generic picker component.
//!
//! Supports files, directories, and buffers with fuzzy match highlighting.
//! When a preview is available, displays a two-column layout with the item
//! list on the left and a syntax-highlighted file preview on the right.

use dioxus::prelude::*;
use lucide_dioxus::Search;

use crate::state::{centered_window, EditorCommand, PickerItem, PickerMode, PickerPreview};
use crate::AppState;

use super::item::PickerItemRow;
use super::preview::PickerPreviewPanel;

/// Generic picker component that displays items with filtering and highlighting.
#[component]
pub fn GenericPicker(
    items: Vec<PickerItem>,
    selected: usize,
    filter: String,
    total: usize,
    mode: PickerMode,
    current_path: Option<String>,
    preview: Option<PickerPreview>,
) -> Element {
    let app_state = use_context::<AppState>();

    // Calculate visible window (show 15 items max, centered on selection)
    let (start, end) = centered_window(selected, items.len(), 15);
    let visible_items: Vec<(usize, &PickerItem)> = items
        .get(start..end)
        .unwrap_or(&[])
        .iter()
        .enumerate()
        .map(|(i, item)| (start + i, item))
        .collect();

    let filtered_count = items.len();

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
        // Overlay backdrop
        div {
            class: "picker-overlay",

            // Picker container
            div {
                class: "{container_class}",

                // Header with title (spans full width, above body)
                div {
                    class: "picker-header",

                    // Title
                    div {
                        class: "picker-title",
                        "{title}"
                    }

                    // Search input display
                    div {
                        class: "picker-search",
                        span {
                            class: "icon-wrapper",
                            style: "width: 16px; height: 16px; margin-right: 8px; color: #5c6370;",
                            Search { size: 16, color: "#5c6370" }
                        }
                        span {
                            class: "picker-search-input",
                            if filter.is_empty() {
                                span {
                                    class: "picker-search-placeholder",
                                    "Type to filter..."
                                }
                            } else {
                                "{filter}"
                            }
                        }
                    }

                    // Current path (for directory browser)
                    if let Some(ref path) = current_path {
                        if mode == PickerMode::DirectoryBrowser {
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
                            kbd { "\u{2191}\u{2193}" }
                            " navigate \u{2022} "
                            kbd { "Enter" }
                            {mode.enter_hint()}
                            kbd { "Esc" }
                            " cancel"
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
                                            item_app_state.process_commands_sync();
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
}
