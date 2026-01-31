//! Code actions menu component.
//!
//! Displays available code actions (quick fixes, refactors) at cursor position.

use dioxus::prelude::*;

use crate::lsp::CodeActionSnapshot;

/// Code actions menu that displays available fixes and refactors.
#[component]
pub fn CodeActionsMenu(
    actions: Vec<CodeActionSnapshot>,
    selected: usize,
    cursor_line: usize,
    cursor_col: usize,
) -> Element {
    // Position the menu near the cursor
    let top = cursor_line * 21 + 60;
    let left = cursor_col * 8 + 60;

    let style = format!("top: {}px; left: {}px;", top.min(400), left.min(500));

    rsx! {
        div {
            class: "code-actions-menu",
            style: "{style}",

            for (idx, action) in actions.iter().enumerate() {
                {
                    let is_selected = idx == selected;
                    let is_disabled = action.disabled.is_some();

                    let mut item_class = String::from("code-action-item");
                    if is_selected {
                        item_class.push_str(" code-action-item-selected");
                    }
                    if is_disabled {
                        item_class.push_str(" code-action-disabled");
                    }

                    let title_class = if action.is_preferred {
                        "code-action-title code-action-preferred"
                    } else {
                        "code-action-title"
                    };

                    rsx! {
                        div {
                            key: "{idx}",
                            class: "{item_class}",

                            // Preferred indicator
                            if action.is_preferred {
                                span {
                                    style: "color: #e5c07b; margin-right: 4px;",
                                    "★"
                                }
                            }

                            // Action title
                            span {
                                class: "{title_class}",
                                "{action.title}"
                            }

                            // Kind indicator (if available)
                            if let Some(ref kind) = action.kind {
                                span {
                                    style: "color: #5c6370; font-size: 11px; margin-left: 8px;",
                                    "[{kind}]"
                                }
                            }
                        }
                    }
                }
            }

            // Empty state
            if actions.is_empty() {
                div {
                    style: "padding: 12px; color: #5c6370; text-align: center;",
                    "No code actions available"
                }
            }
        }
    }
}
