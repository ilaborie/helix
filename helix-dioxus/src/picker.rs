//! Generic picker component.
//!
//! Supports files, directories, and buffers with fuzzy match highlighting.

use dioxus::prelude::*;
use lucide_dioxus::{ChevronRight, File, FileText, Folder, Search};

use crate::state::{PickerIcon, PickerItem, PickerMode};

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
            style: "
                position: absolute;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background-color: rgba(0, 0, 0, 0.5);
                display: flex;
                justify-content: center;
                align-items: flex-start;
                padding-top: 80px;
                z-index: 100;
            ",

            // Picker container
            div {
                class: "picker-container",
                style: "
                    background-color: #21252b;
                    border: 1px solid #3e4451;
                    border-radius: 6px;
                    width: 500px;
                    max-height: 450px;
                    overflow: hidden;
                    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
                ",

                // Header with title
                div {
                    style: "
                        padding: 8px 12px 4px 12px;
                        border-bottom: 1px solid #3e4451;
                    ",

                    // Title
                    div {
                        style: "
                            color: #abb2bf;
                            font-size: 12px;
                            font-weight: 500;
                            margin-bottom: 8px;
                        ",
                        "{title}"
                    }

                    // Search input display
                    div {
                        style: "
                            display: flex;
                            align-items: center;
                            background-color: #282c34;
                            border: 1px solid #3e4451;
                            border-radius: 4px;
                            padding: 6px 8px;
                            margin-bottom: 4px;
                        ",
                        span {
                            style: "width: 16px; height: 16px; margin-right: 8px; display: flex; align-items: center; color: #5c6370;",
                            Search { size: 16, color: "#5c6370" }
                        }
                        span {
                            style: "
                                color: #abb2bf;
                                font-size: 14px;
                                flex: 1;
                            ",
                            if filter.is_empty() {
                                span {
                                    style: "color: #5c6370;",
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
                                style: "
                                    color: #5c6370;
                                    font-size: 11px;
                                    margin-bottom: 4px;
                                    overflow: hidden;
                                    text-overflow: ellipsis;
                                    white-space: nowrap;
                                ",
                                "{path}"
                            }
                        }
                    }

                    // Help text row with count
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; padding: 2px;",

                        // Left: help text with kbd elements
                        span {
                            style: "color: #5c6370; font-size: 12px;",
                            kbd {
                                style: "
                                    background-color: #3e4451;
                                    border-radius: 3px;
                                    padding: 2px 5px;
                                    font-family: inherit;
                                    font-size: 11px;
                                ",
                                "\u{2191}\u{2193}"
                            }
                            " navigate \u{2022} "
                            kbd {
                                style: "
                                    background-color: #3e4451;
                                    border-radius: 3px;
                                    padding: 2px 5px;
                                    font-family: inherit;
                                    font-size: 11px;
                                ",
                                "Enter"
                            }
                            if mode == PickerMode::DirectoryBrowser {
                                " open/enter \u{2022} "
                            } else {
                                " select \u{2022} "
                            }
                            kbd {
                                style: "
                                    background-color: #3e4451;
                                    border-radius: 3px;
                                    padding: 2px 5px;
                                    font-family: inherit;
                                    font-size: 11px;
                                ",
                                "Esc"
                            }
                            " cancel"
                        }

                        // Right: count
                        span {
                            style: "color: #5c6370; font-size: 12px;",
                            "{filtered_count} / {total}"
                        }
                    }
                }

                // Item list
                div {
                    class: "picker-list",
                    style: "
                        overflow-y: auto;
                        max-height: 340px;
                    ",

                    if items.is_empty() {
                        div {
                            style: "
                                padding: 16px;
                                color: #5c6370;
                                text-align: center;
                            ",
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

/// Individual picker item row.
#[component]
fn PickerItemRow(item: PickerItem, is_selected: bool) -> Element {
    let bg_color = if is_selected {
        "#3e4451"
    } else {
        "transparent"
    };

    let text_color = match item.icon {
        PickerIcon::Folder => "#61afef",         // Blue for directories
        PickerIcon::BufferModified => "#e5c07b", // Yellow for modified buffers
        _ => "#abb2bf",                          // Default text color
    };

    let indicator_opacity = if is_selected { "1" } else { "0" };

    rsx! {
        div {
            class: "picker-item",
            style: "
                padding: 6px 8px;
                background-color: {bg_color};
                display: flex;
                align-items: center;
                cursor: pointer;
                transition: background-color 0.1s;
                min-height: 32px;
                box-sizing: border-box;
            ",

            // Selection indicator
            span {
                style: "width: 16px; height: 16px; display: flex; align-items: center; opacity: {indicator_opacity}; flex-shrink: 0;",
                ChevronRight { size: 16, color: "#98c379" }
            }

            // Icon based on type
            span {
                style: "width: 16px; height: 16px; margin-right: 8px; display: flex; align-items: center; flex-shrink: 0;",
                {match item.icon {
                    PickerIcon::Folder => rsx! { Folder { size: 16, color: text_color } },
                    PickerIcon::File => rsx! { File { size: 16, color: text_color } },
                    PickerIcon::Buffer | PickerIcon::BufferModified => rsx! { FileText { size: 16, color: text_color } },
                }}
            }

            // Main content area
            div {
                style: "flex: 1; overflow: hidden; display: flex; align-items: center; justify-content: space-between;",

                // Left side: text with highlighting
                div {
                    style: "overflow: hidden; flex: 1;",

                    // Primary text with highlighting
                    div {
                        style: "display: flex; align-items: center;",
                        HighlightedText {
                            text: item.display.clone(),
                            indices: item.match_indices.clone(),
                            base_color: text_color.to_string(),
                        }

                        // Modified indicator for buffers
                        if matches!(item.icon, PickerIcon::BufferModified) {
                            span {
                                style: "color: #e5c07b; margin-left: 4px;",
                                "\u{25cf}"
                            }
                        }
                    }

                    // Secondary text for files (path) - NOT for buffers or parent directory
                    if !matches!(item.icon, PickerIcon::Buffer | PickerIcon::BufferModified) && item.display != ".." {
                        if let Some(ref secondary) = item.secondary {
                            div {
                                style: "
                                    color: #5c6370;
                                    font-size: 11px;
                                    overflow: hidden;
                                    text-overflow: ellipsis;
                                    white-space: nowrap;
                                ",
                                "{secondary}"
                            }
                        }
                    }
                }

                // Right side badges
                // "current" badge for buffers
                if matches!(item.icon, PickerIcon::Buffer | PickerIcon::BufferModified) {
                    if item.secondary.as_deref() == Some("current") {
                        span {
                            style: "
                                color: #5c6370;
                                font-size: 11px;
                                margin-left: 8px;
                                flex-shrink: 0;
                            ",
                            "current"
                        }
                    }
                }

                // "Parent directory" badge for ".." entry
                if item.display == ".." {
                    span {
                        style: "
                            color: #5c6370;
                            font-size: 11px;
                            margin-left: 8px;
                            flex-shrink: 0;
                        ",
                        "Parent directory"
                    }
                }
            }
        }
    }
}

/// Text with highlighted match indices.
#[component]
fn HighlightedText(text: String, indices: Vec<usize>, base_color: String) -> Element {
    if indices.is_empty() {
        return rsx! {
            span {
                style: "color: {base_color}; font-size: 14px;",
                "{text}"
            }
        };
    }

    // Build spans for highlighted and non-highlighted segments
    let mut segments: Vec<Element> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let indices_set: std::collections::HashSet<usize> = indices.iter().copied().collect();

    let mut current_start = 0;
    let mut in_highlight = false;

    for (i, _ch) in chars.iter().enumerate() {
        let is_match = indices_set.contains(&i);

        if i == 0 {
            in_highlight = is_match;
            current_start = 0;
        } else if is_match != in_highlight {
            // Transition - emit previous segment
            if let Some(slice) = chars.get(current_start..i) {
                let segment_text: String = slice.iter().collect();
                if in_highlight {
                    segments.push(rsx! {
                        span {
                            key: "{current_start}",
                            style: "color: #e5c07b; font-weight: 600; font-size: 14px;",
                            "{segment_text}"
                        }
                    });
                } else {
                    segments.push(rsx! {
                        span {
                            key: "{current_start}",
                            style: "color: {base_color}; font-size: 14px;",
                            "{segment_text}"
                        }
                    });
                }
            }
            current_start = i;
            in_highlight = is_match;
        }
    }

    // Emit final segment
    if let Some(slice) = chars.get(current_start..) {
        let segment_text: String = slice.iter().collect();
        if in_highlight {
            segments.push(rsx! {
                span {
                    key: "{current_start}",
                    style: "color: #e5c07b; font-weight: 600; font-size: 14px;",
                    "{segment_text}"
                }
            });
        } else {
            segments.push(rsx! {
                span {
                    key: "{current_start}",
                    style: "color: {base_color}; font-size: 14px;",
                    "{segment_text}"
                }
            });
        }
    }

    rsx! {
        span {
            {segments.into_iter()}
        }
    }
}
