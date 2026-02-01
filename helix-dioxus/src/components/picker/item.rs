//! Picker item row component.

use dioxus::prelude::*;
use lucide_dioxus::{
    Blocks, Braces, ChevronRight, Code, Component, File, FileText, Folder, Hash, Layers, Package,
    SquareFunction, Variable,
};

use crate::state::{PickerIcon, PickerItem};

use super::highlight::HighlightedText;

/// Individual picker item row.
#[component]
pub fn PickerItemRow(item: PickerItem, is_selected: bool) -> Element {
    let item_class = if is_selected {
        "picker-item picker-item-selected"
    } else {
        "picker-item"
    };

    let text_color = match item.icon {
        PickerIcon::Folder => "#61afef",         // Blue for directories
        PickerIcon::BufferModified => "#e5c07b", // Yellow for modified buffers
        // Symbol colors (matching LSP completion icons in code_actions.rs and completion.rs)
        PickerIcon::SymbolFunction | PickerIcon::SymbolMethod => "#61afef", // Blue
        PickerIcon::SymbolClass
        | PickerIcon::SymbolStruct
        | PickerIcon::SymbolEnum
        | PickerIcon::SymbolInterface => "#e5c07b", // Yellow
        PickerIcon::SymbolVariable | PickerIcon::SymbolField => "#e06c75",  // Red
        PickerIcon::SymbolConstant => "#d19a66",                            // Orange
        PickerIcon::SymbolModule => "#c678dd",                              // Purple
        PickerIcon::SymbolOther => "#abb2bf",                               // Gray
        _ => "#abb2bf",                                                     // Default text color
    };

    let indicator_opacity = if is_selected { "1" } else { "0" };

    rsx! {
        div {
            class: "{item_class}",

            // Selection indicator
            span {
                class: "icon-wrapper",
                style: "width: 16px; height: 16px; opacity: {indicator_opacity}; flex-shrink: 0;",
                ChevronRight { size: 16, color: "#98c379" }
            }

            // Icon based on type
            span {
                class: "icon-wrapper",
                style: "width: 16px; height: 16px; margin-right: 8px; flex-shrink: 0;",
                {match item.icon {
                    PickerIcon::Folder => rsx! { Folder { size: 16, color: text_color } },
                    PickerIcon::File => rsx! { File { size: 16, color: text_color } },
                    PickerIcon::Buffer | PickerIcon::BufferModified => rsx! { FileText { size: 16, color: text_color } },
                    // Symbol icons
                    PickerIcon::SymbolFunction => rsx! { SquareFunction { size: 16, color: text_color } },
                    PickerIcon::SymbolMethod => rsx! { Code { size: 16, color: text_color } },
                    PickerIcon::SymbolClass => rsx! { Blocks { size: 16, color: text_color } },
                    PickerIcon::SymbolStruct => rsx! { Braces { size: 16, color: text_color } },
                    PickerIcon::SymbolEnum => rsx! { Layers { size: 16, color: text_color } },
                    PickerIcon::SymbolInterface => rsx! { Component { size: 16, color: text_color } },
                    PickerIcon::SymbolVariable => rsx! { Variable { size: 16, color: text_color } },
                    PickerIcon::SymbolConstant => rsx! { Hash { size: 16, color: text_color } },
                    PickerIcon::SymbolField => rsx! { Code { size: 16, color: text_color } },
                    PickerIcon::SymbolModule => rsx! { Package { size: 16, color: text_color } },
                    PickerIcon::SymbolOther => rsx! { Code { size: 16, color: text_color } },
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
                                class: "picker-item-secondary",
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
                            class: "picker-item-secondary",
                            style: "margin-left: 8px; flex-shrink: 0;",
                            "current"
                        }
                    }
                }

                // "Parent directory" badge for ".." entry
                if item.display == ".." {
                    span {
                        class: "picker-item-secondary",
                        style: "margin-left: 8px; flex-shrink: 0;",
                        "Parent directory"
                    }
                }
            }
        }
    }
}
