//! Picker item row component.

use dioxus::prelude::*;
use lucide_dioxus::{
    Blocks, Bookmark, Braces, ChevronRight, CircleX, Code, Component, FileCode, FileDiff, FileMinus, FilePen, FilePlus,
    FileX, Hash, Info, Layers, Lightbulb, Link2, Package, Palette, Smile, SquareFunction, Terminal, TextSearch,
    TriangleAlert, Variable,
};

use crate::components::file_icons::{FileTypeIcon, FolderTypeIcon};
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
        PickerIcon::Folder
        | PickerIcon::FolderOpen
        | PickerIcon::SymbolFunction
        | PickerIcon::SymbolMethod
        | PickerIcon::Reference
        | PickerIcon::VcsRenamed => "var(--accent)",
        PickerIcon::BufferModified
        | PickerIcon::SymbolClass
        | PickerIcon::SymbolStruct
        | PickerIcon::SymbolEnum
        | PickerIcon::SymbolInterface
        | PickerIcon::DiagnosticWarning
        | PickerIcon::VcsModified
        | PickerIcon::Emoji => "var(--warning)",
        PickerIcon::SymbolVariable
        | PickerIcon::SymbolField
        | PickerIcon::DiagnosticError
        | PickerIcon::VcsConflict
        | PickerIcon::VcsDeleted => "var(--error)",
        PickerIcon::SymbolConstant | PickerIcon::JumpEntry => "var(--orange)",
        PickerIcon::SymbolModule | PickerIcon::Definition | PickerIcon::Register | PickerIcon::Theme => "var(--purple)",
        PickerIcon::SymbolOther | PickerIcon::File | PickerIcon::Buffer => "var(--text)",
        PickerIcon::DiagnosticInfo => "var(--info)",
        PickerIcon::DiagnosticHint | PickerIcon::Command => "var(--hint)",
        PickerIcon::SearchResult | PickerIcon::VcsAdded => "var(--success)",
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
    let indent_px = u32::from(item.depth) * 16;

    rsx! {
        div {
            class: "{item_class}",
            onclick: move |evt| {
                if let Some(handler) = &on_click {
                    handler.call(evt);
                }
            },

            // Depth indentation
            if indent_px > 0 {
                span {
                    style: "width: {indent_px}px; flex-shrink: 0;",
                }
            }

            // Selection indicator
            span {
                class: "icon-wrapper",
                style: "width: 16px; height: 16px; opacity: {indicator_opacity}; flex-shrink: 0; color: var(--success);",
                ChevronRight { size: 16, color: "currentColor" }
            }

            // Icon based on type
            // File/Folder/Buffer variants use Material Icon Theme SVGs (colorful)
            // All other variants keep Lucide icons (monochrome with color via CSS)
            {match item.icon {
                PickerIcon::Folder => rsx! {
                    span { style: "margin-right: 8px; flex-shrink: 0;",
                        FolderTypeIcon { size: 16 }
                    }
                },
                PickerIcon::FolderOpen => rsx! {
                    span { style: "margin-right: 8px; flex-shrink: 0;",
                        FolderTypeIcon { is_open: true, size: 16 }
                    }
                },
                PickerIcon::File => rsx! {
                    span { style: "margin-right: 8px; flex-shrink: 0;",
                        FileTypeIcon { name: item.display.clone(), size: 16 }
                    }
                },
                PickerIcon::Buffer | PickerIcon::BufferModified => rsx! {
                    span { style: "margin-right: 8px; flex-shrink: 0;",
                        FileTypeIcon { name: item.display.clone(), size: 16 }
                    }
                },
                _ => rsx! {
                    span {
                        class: "icon-wrapper",
                        style: "width: 16px; height: 16px; margin-right: 8px; flex-shrink: 0; color: {icon_color};",
                        {match item.icon {
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
                            PickerIcon::Register => rsx! { Code { size: 16, color: "currentColor" } },
                            PickerIcon::Command => rsx! { Terminal { size: 16, color: "currentColor" } },
                            PickerIcon::JumpEntry => rsx! { Bookmark { size: 16, color: "currentColor" } },
                            PickerIcon::Theme => rsx! { Palette { size: 16, color: "currentColor" } },
                            // VCS icons
                            PickerIcon::VcsAdded => rsx! { FilePlus { size: 16, color: "currentColor" } },
                            PickerIcon::VcsModified => rsx! { FilePen { size: 16, color: "currentColor" } },
                            PickerIcon::VcsConflict => rsx! { FileX { size: 16, color: "currentColor" } },
                            PickerIcon::VcsDeleted => rsx! { FileMinus { size: 16, color: "currentColor" } },
                            PickerIcon::VcsRenamed => rsx! { FileDiff { size: 16, color: "currentColor" } },
                            PickerIcon::Emoji => rsx! { Smile { size: 16, color: "currentColor" } },
                            // Already handled above
                            PickerIcon::Folder | PickerIcon::FolderOpen | PickerIcon::File
                            | PickerIcon::Buffer | PickerIcon::BufferModified => unreachable!(),
                        }}
                    }
                },
            }}

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
