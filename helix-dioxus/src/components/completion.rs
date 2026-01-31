//! Completion popup component.
//!
//! Displays an auto-complete menu with LSP completion items.

use dioxus::prelude::*;

use crate::lsp::CompletionItemSnapshot;

/// Completion popup that displays auto-complete suggestions.
#[component]
pub fn CompletionPopup(
    items: Vec<CompletionItemSnapshot>,
    selected: usize,
    cursor_line: usize,
    cursor_col: usize,
) -> Element {
    // Position the popup near the cursor
    // TODO: Calculate actual pixel position based on cursor
    let top = (cursor_line + 1) * 21 + 40; // Approximate: line height * line + buffer bar
    let left = cursor_col * 8 + 60; // Approximate: char width * col + gutter

    let style = format!(
        "top: {}px; left: {}px;",
        top.min(400), // Cap to avoid going off screen
        left.min(600)
    );

    rsx! {
        div {
            class: "completion-popup",
            style: "{style}",

            for (idx, item) in items.iter().enumerate() {
                {
                    let is_selected = idx == selected;
                    let item_class = if is_selected {
                        "completion-item completion-item-selected"
                    } else {
                        "completion-item"
                    };
                    let label_class = if item.deprecated {
                        "completion-item-label completion-item-deprecated"
                    } else {
                        "completion-item-label"
                    };
                    let kind_color = item.kind.css_color();
                    let kind_text = item.kind.short_name();

                    rsx! {
                        div {
                            key: "{idx}",
                            class: "{item_class}",

                            // Kind badge
                            span {
                                class: "completion-item-kind",
                                style: "color: {kind_color};",
                                "{kind_text}"
                            }

                            // Label
                            span {
                                class: "{label_class}",
                                "{item.label}"
                            }

                            // Detail (type signature, etc.)
                            if let Some(ref detail) = item.detail {
                                span {
                                    class: "completion-item-detail",
                                    "{detail}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
