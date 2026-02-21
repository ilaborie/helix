//! Picker item row component.

use dioxus::prelude::*;

use crate::components::file_icons::{FileTypeIcon, FolderTypeIcon};
use crate::icons::{lucide, Icon};
use crate::state::{PickerIcon, PickerItem};

use super::highlight::HighlightedText;

/// Split a shortcut string into renderable key segments for `<kbd>` display.
///
/// Examples: `"Ctrl+f"` → `["Ctrl", "f"]`, `"C-b"` → `["Ctrl", "b"]`,
/// `"Space f"` → `["Space", "f"]`, `"gg"` → `["g", "g"]`, `":reload"` → `[":reload"]`.
pub fn parse_shortcut_keys(s: &str) -> Vec<String> {
    if s.starts_with(':') {
        return vec![s.to_string()];
    }
    if s.contains('+') {
        return s.split('+').map(String::from).collect();
    }
    if let Some(rest) = s.strip_prefix("C-") {
        return vec!["Ctrl".to_string(), rest.to_string()];
    }
    if let Some(rest) = s.strip_prefix("A-") {
        return vec!["Alt".to_string(), rest.to_string()];
    }
    if s.contains(' ') {
        return s.split_whitespace().map(String::from).collect();
    }
    if s.chars().count() > 1 {
        return s.chars().map(|c| c.to_string()).collect();
    }
    vec![s.to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typable_command() {
        assert_eq!(parse_shortcut_keys(":reload"), vec![":reload"]);
        assert_eq!(parse_shortcut_keys(":write"), vec![":write"]);
    }

    #[test]
    fn modifier_plus_key() {
        assert_eq!(parse_shortcut_keys("Ctrl+f"), vec!["Ctrl", "f"]);
        assert_eq!(parse_shortcut_keys("Ctrl+Space"), vec!["Ctrl", "Space"]);
    }

    #[test]
    fn helix_c_prefix() {
        assert_eq!(parse_shortcut_keys("C-b"), vec!["Ctrl", "b"]);
    }

    #[test]
    fn helix_a_prefix() {
        assert_eq!(parse_shortcut_keys("A-o"), vec!["Alt", "o"]);
    }

    #[test]
    fn space_separated() {
        assert_eq!(parse_shortcut_keys("Space f"), vec!["Space", "f"]);
        assert_eq!(parse_shortcut_keys("g s"), vec!["g", "s"]);
    }

    #[test]
    fn multi_char_sequence() {
        assert_eq!(parse_shortcut_keys("gg"), vec!["g", "g"]);
        assert_eq!(parse_shortcut_keys("mm"), vec!["m", "m"]);
    }

    #[test]
    fn single_key() {
        assert_eq!(parse_shortcut_keys("/"), vec!["/"]);
        assert_eq!(parse_shortcut_keys("u"), vec!["u"]);
    }
}

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
                Icon { data: lucide::ChevronRight, size: "16", fill: "currentColor" }
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
                            PickerIcon::SymbolFunction => rsx! { Icon { data: lucide::SquareFunction, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolMethod => rsx! { Icon { data: lucide::Code, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolClass => rsx! { Icon { data: lucide::Blocks, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolStruct => rsx! { Icon { data: lucide::Braces, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolEnum => rsx! { Icon { data: lucide::Layers, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolInterface => rsx! { Icon { data: lucide::Component, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolVariable => rsx! { Icon { data: lucide::Variable, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolConstant => rsx! { Icon { data: lucide::Hash, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolField => rsx! { Icon { data: lucide::Code, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolModule => rsx! { Icon { data: lucide::Package, size: "16", fill: "currentColor" } },
                            PickerIcon::SymbolOther => rsx! { Icon { data: lucide::Code, size: "16", fill: "currentColor" } },
                            // Diagnostic icons
                            PickerIcon::DiagnosticError => rsx! { Icon { data: lucide::CircleX, size: "16", fill: "currentColor" } },
                            PickerIcon::DiagnosticWarning => rsx! { Icon { data: lucide::TriangleAlert, size: "16", fill: "currentColor" } },
                            PickerIcon::DiagnosticInfo => rsx! { Icon { data: lucide::Info, size: "16", fill: "currentColor" } },
                            PickerIcon::DiagnosticHint => rsx! { Icon { data: lucide::Lightbulb, size: "16", fill: "currentColor" } },
                            PickerIcon::SearchResult => rsx! { Icon { data: lucide::TextSearch, size: "16", fill: "currentColor" } },
                            PickerIcon::Reference => rsx! { Icon { data: lucide::Link2, size: "16", fill: "currentColor" } },
                            PickerIcon::Definition => rsx! { Icon { data: lucide::FileCode, size: "16", fill: "currentColor" } },
                            PickerIcon::Register => rsx! { Icon { data: lucide::Code, size: "16", fill: "currentColor" } },
                            PickerIcon::Command => rsx! { Icon { data: lucide::Terminal, size: "16", fill: "currentColor" } },
                            PickerIcon::JumpEntry => rsx! { Icon { data: lucide::Bookmark, size: "16", fill: "currentColor" } },
                            PickerIcon::Theme => rsx! { Icon { data: lucide::Palette, size: "16", fill: "currentColor" } },
                            // VCS icons
                            PickerIcon::VcsAdded => rsx! { Icon { data: lucide::FilePlus, size: "16", fill: "currentColor" } },
                            PickerIcon::VcsModified => rsx! { Icon { data: lucide::FilePen, size: "16", fill: "currentColor" } },
                            PickerIcon::VcsConflict => rsx! { Icon { data: lucide::FileX, size: "16", fill: "currentColor" } },
                            PickerIcon::VcsDeleted => rsx! { Icon { data: lucide::FileMinus, size: "16", fill: "currentColor" } },
                            PickerIcon::VcsRenamed => rsx! { Icon { data: lucide::FileDiff, size: "16", fill: "currentColor" } },
                            PickerIcon::Emoji => rsx! { Icon { data: lucide::Smile, size: "16", fill: "currentColor" } },
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

                    // Secondary text - NOT for buffers or parent directory
                    if !matches!(item.icon, PickerIcon::Buffer | PickerIcon::BufferModified) && item.display != ".." {
                        if let Some(ref secondary) = item.secondary {
                            if matches!(item.icon, PickerIcon::Command) {
                                // Render command shortcuts as styled <kbd> elements
                                div {
                                    class: "picker-item-shortcut",
                                    for key in parse_shortcut_keys(secondary) {
                                        kbd { class: "kbd-key-compact", "{key}" }
                                    }
                                }
                            } else {
                                div {
                                    class: "picker-item-secondary",
                                    "{secondary}"
                                }
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
