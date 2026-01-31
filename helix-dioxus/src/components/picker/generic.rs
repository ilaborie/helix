//! Generic picker component.
//!
//! Supports files, directories, and buffers with fuzzy match highlighting.

use dioxus::prelude::*;
use lucide_dioxus::Search;

use crate::state::{PickerItem, PickerMode};

use super::item::PickerItemRow;

/// Generic picker component that displays items with filtering and highlighting.
#[component]
pub fn GenericPicker(
    items: Vec<PickerItem>,
    selected: usize,
    filter: String,
    total: usize,
    mode: PickerMode,
    current_path: Option<String>,
) -> Element {
    // Calculate visible window (show 15 items max, centered on selection)
    let window_size = 15usize;
    let half_window = window_size / 2;

    let start = if selected <= half_window {
        0
    } else if selected + half_window >= items.len() {
        items.len().saturating_sub(window_size)
    } else {
        selected - half_window
    };

    let end = (start + window_size).min(items.len());
    let visible_items: Vec<(usize, &PickerItem)> = items
        .get(start..end)
        .unwrap_or(&[])
        .iter()
        .enumerate()
        .map(|(i, item)| (start + i, item))
        .collect();

    let filtered_count = items.len();

    // Determine title based on mode
    let title = match mode {
        PickerMode::DirectoryBrowser => "Open File",
        PickerMode::FilesRecursive => "Find Files",
        PickerMode::Buffers => "Switch Buffer",
    };

    rsx! {
        // Overlay backdrop
        div {
            class: "picker-overlay",

            // Picker container
            div {
                class: "picker-container",

                // Header with title
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
                            if mode == PickerMode::DirectoryBrowser {
                                " open/enter \u{2022} "
                            } else {
                                " select \u{2022} "
                            }
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

                // Item list
                div {
                    class: "picker-list",

                    if items.is_empty() {
                        div {
                            class: "picker-empty",
                            if filter.is_empty() {
                                "No items"
                            } else {
                                "No matches found"
                            }
                        }
                    } else {
                        for (idx, item) in visible_items {
                            PickerItemRow {
                                key: "{idx}",
                                item: item.clone(),
                                is_selected: idx == selected,
                            }
                        }
                    }
                }
            }
        }
    }
}
