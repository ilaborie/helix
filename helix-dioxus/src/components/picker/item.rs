//! Picker item row component.

use dioxus::prelude::*;
use lucide_dioxus::{
    Blocks, Bookmark, Braces, ChevronRight, CircleX, Code, Component, File, FileCode, FileText,
    Folder, Hash, Info, Layers, Lightbulb, Link2, Package, Palette, SquareFunction, Terminal,
    TextSearch, TriangleAlert, Variable,
};

use crate::state::{PickerIcon, PickerItem};

use super::highlight::HighlightedText;

/// Individual picker item row.
#[component]
pub fn PickerItemRow(
    item: PickerItem,
    is_selected: bool,
    #[props(default)] on_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    let item_class = if is_selected {
        "picker-item picker-item-selected"
    } else {
        "picker-item"
    };

    // Icon color based on type (using CSS variables)
    let icon_color = match item.icon {
        PickerIcon::Folder => "var(--accent)",
        PickerIcon::BufferModified => "var(--warning)",
        // Symbol colors
        PickerIcon::SymbolFunction | PickerIcon::SymbolMethod => "var(--accent)",
        PickerIcon::SymbolClass
        | PickerIcon::SymbolStruct
        | PickerIcon::SymbolEnum
        | PickerIcon::SymbolInterface => "var(--warning)",
        PickerIcon::SymbolVariable | PickerIcon::SymbolField => "var(--error)",
        PickerIcon::SymbolConstant => "var(--orange)",
        PickerIcon::SymbolModule => "var(--purple)",
        PickerIcon::SymbolOther => "var(--text)",
        // Diagnostic colors by severity
        PickerIcon::DiagnosticError => "var(--error)",
        PickerIcon::DiagnosticWarning => "var(--warning)",
        PickerIcon::DiagnosticInfo => "var(--info)",
        PickerIcon::DiagnosticHint => "var(--hint)",
        // Search result
        PickerIcon::SearchResult => "var(--success)",
        // Location icons
        PickerIcon::Reference => "var(--accent)",
        PickerIcon::Definition | PickerIcon::Register => "var(--purple)",
        // Command panel
        PickerIcon::Command => "var(--hint)",
        // Jump list
        PickerIcon::JumpEntry => "var(--orange)",
        // Theme
        PickerIcon::Theme => "var(--purple)",
        // Default colors
        PickerIcon::File | PickerIcon::Buffer => "var(--text)",
    };

    // Text color - use neutral for diagnostics so highlighting is visible
    let text_color = match item.icon {
        PickerIcon::DiagnosticError
        | PickerIcon::DiagnosticWarning
        | PickerIcon::DiagnosticInfo
        | PickerIcon::DiagnosticHint => "var(--text)",
        _ => icon_color,
    };

    let indicator_opacity = if is_selected { "1" } else { "0" };

    rsx! {
        div {
            class: "{item_class}",
            onclick: move |evt| {
                if let Some(handler) = &on_click {
                    handler.call(evt);
                }
            },

            // Selection indicator
            span {
                class: "icon-wrapper",
                style: "width: 16px; height: 16px; opacity: {indicator_opacity}; flex-shrink: 0; color: var(--success);",
                ChevronRight { size: 16, color: "currentColor" }
            }

            // Icon based on type
            span {
                class: "icon-wrapper",
                style: "width: 16px; height: 16px; margin-right: 8px; flex-shrink: 0; color: {icon_color};",
                {match item.icon {
                    PickerIcon::Folder => rsx! { Folder { size: 16, color: "currentColor" } },
                    PickerIcon::File => rsx! { File { size: 16, color: "currentColor" } },
                    PickerIcon::Buffer | PickerIcon::BufferModified => rsx! { FileText { size: 16, color: "currentColor" } },
                    // Symbol icons
                    PickerIcon::SymbolFunction => rsx! { SquareFunction { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolMethod => rsx! { Code { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolClass => rsx! { Blocks { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolStruct => rsx! { Braces { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolEnum => rsx! { Layers { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolInterface => rsx! { Component { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolVariable => rsx! { Variable { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolConstant => rsx! { Hash { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolField => rsx! { Code { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolModule => rsx! { Package { size: 16, color: "currentColor" } },
                    PickerIcon::SymbolOther => rsx! { Code { size: 16, color: "currentColor" } },
                    // Diagnostic icons
                    PickerIcon::DiagnosticError => rsx! { CircleX { size: 16, color: "currentColor" } },
                    PickerIcon::DiagnosticWarning => rsx! { TriangleAlert { size: 16, color: "currentColor" } },
                    PickerIcon::DiagnosticInfo => rsx! { Info { size: 16, color: "currentColor" } },
                    PickerIcon::DiagnosticHint => rsx! { Lightbulb { size: 16, color: "currentColor" } },
                    PickerIcon::SearchResult => rsx! { TextSearch { size: 16, color: "currentColor" } },
                    PickerIcon::Reference => rsx! { Link2 { size: 16, color: "currentColor" } },
                    PickerIcon::Definition => rsx! { FileCode { size: 16, color: "currentColor" } },
                    PickerIcon::Register => rsx! { FileText { size: 16, color: "currentColor" } },
                    PickerIcon::Command => rsx! { Terminal { size: 16, color: "currentColor" } },
                    PickerIcon::JumpEntry => rsx! { Bookmark { size: 16, color: "currentColor" } },
                    PickerIcon::Theme => rsx! { Palette { size: 16, color: "currentColor" } },
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
                                style: "color: var(--warning); margin-left: 4px;",
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

                // "current" badge for themes
                if matches!(item.icon, PickerIcon::Theme) {
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
