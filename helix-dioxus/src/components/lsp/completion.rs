//! Completion popup component.
//!
//! Displays an auto-complete menu with LSP completion items.

use dioxus::prelude::*;

use crate::components::inline_dialog::{DialogConstraints, InlineListDialog, InlineListItem};
use crate::lsp::CompletionItemSnapshot;

/// A single completion item in the menu.
#[component]
fn CompletionItem(item: CompletionItemSnapshot, is_selected: bool) -> Element {
    let label_class = if item.deprecated {
        "completion-item-label completion-item-deprecated"
    } else {
        "completion-item-label"
    };
    let kind_color = item.kind.css_color();
    let kind_text = item.kind.short_name();

    rsx! {
        InlineListItem {
            is_selected,
            class: "completion-row",

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

/// Completion popup that displays auto-complete suggestions.
#[component]
pub fn CompletionPopup(items: Vec<CompletionItemSnapshot>, selected: usize) -> Element {
    let constraints = DialogConstraints {
        min_width: Some(250),
        max_width: Some(500),
        max_height: Some(300),
    };

    rsx! {
        InlineListDialog {
            selected,
            empty_message: "No completions",
            class: "completion-popup",
            constraints,
            has_items: !items.is_empty(),

            for (idx, item) in items.iter().enumerate() {
                CompletionItem {
                    key: "{idx}",
                    item: item.clone(),
                    is_selected: idx == selected,
                }
            }
        }
    }
}
