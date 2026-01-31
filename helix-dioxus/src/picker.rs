//! File picker component.
//!
//! Displays a modal file list with selection highlighting and keyboard navigation.

use dioxus::prelude::*;
use lucide_dioxus::{ChevronRight, File, Folder};

/// File picker component that displays a scrollable list of files.
#[component]
pub fn FilePicker(items: Vec<String>, selected: usize) -> Element {
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
    let visible_items: Vec<(usize, &String)> = items[start..end]
        .iter()
        .enumerate()
        .map(|(i, item)| (start + i, item))
        .collect();

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
                align-items: center;
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
                    max-height: 400px;
                    overflow: hidden;
                    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
                ",

                // Header
                div {
                    class: "picker-header",
                    style: "
                        padding: 12px 16px;
                        border-bottom: 1px solid #3e4451;
                        font-size: 14px;
                        color: #abb2bf;
                    ",
                    "Open file"
                    span {
                        style: "
                            margin-left: 8px;
                            color: #5c6370;
                            font-size: 12px;
                        ",
                        "(j/k to navigate, Enter to open, Esc to cancel)"
                    }
                }

                // File list
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
                            "No files found"
                        }
                    } else {
                        for (idx, item) in visible_items {
                            PickerItem {
                                key: "{idx}",
                                name: item.clone(),
                                is_selected: idx == selected,
                                is_directory: item.ends_with('/'),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual picker item.
#[component]
fn PickerItem(name: String, is_selected: bool, is_directory: bool) -> Element {
    let bg_color = if is_selected {
        "#3e4451"
    } else {
        "transparent"
    };

    let text_color = if is_directory {
        "#61afef" // Blue for directories
    } else {
        "#abb2bf" // Default text color for files
    };

    let indicator_opacity = if is_selected { "1" } else { "0" };

    rsx! {
        div {
            class: "picker-item",
            style: "
                padding: 8px 16px;
                background-color: {bg_color};
                display: flex;
                align-items: center;
                cursor: pointer;
                transition: background-color 0.1s;
                height: 36px;
                box-sizing: border-box;
            ",

            // Selection indicator
            span {
                style: "width: 16px; height: 16px; display: flex; align-items: center; opacity: {indicator_opacity};",
                ChevronRight { size: 16, color: "#98c379" }
            }

            // File/directory icon
            span {
                style: "width: 16px; height: 16px; margin-right: 8px; display: flex; align-items: center;",
                if is_directory {
                    Folder { size: 16, color: text_color }
                } else {
                    File { size: 16, color: text_color }
                }
            }

            // File name
            span {
                style: "
                    color: {text_color};
                    font-size: 14px;
                ",
                "{name}"
            }
        }
    }
}
