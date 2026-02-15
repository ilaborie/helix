//! Command completion popup component.
//!
//! Renders a filtered list of commands above the command prompt when in command mode.

use dioxus::prelude::*;

use crate::state::{centered_window, CommandCompletionItem};

/// Maximum visible items in the completion popup.
const MAX_VISIBLE: usize = 10;

/// Command completion popup shown above the command prompt.
#[allow(clippy::indexing_slicing)] // start..end bounds are computed from centered_window which guarantees valid range
#[component]
pub fn CommandCompletionPopup(items: Vec<CommandCompletionItem>, selected: usize) -> Element {
    let total = items.len();
    let (start, end) = centered_window(selected, total, MAX_VISIBLE);
    let visible_items = &items[start..end];

    rsx! {
        div {
            class: "command-completion",

            // Header with count
            div {
                class: "command-completion-header",
                "Commands ({total})"
            }

            // Scrollable list
            for (idx, item) in visible_items.iter().enumerate() {
                {
                    let abs_idx = start + idx;
                    let is_selected = abs_idx == selected;
                    let class = if is_selected {
                        "command-completion-item command-completion-item-selected"
                    } else {
                        "command-completion-item"
                    };

                    rsx! {
                        div {
                            key: "{abs_idx}",
                            class: "{class}",
                            id: if is_selected { "command-completion-active" },

                            // Command name with highlighted match characters
                            span {
                                class: "command-completion-name",
                                {render_highlighted_name(&item.name, &item.match_indices)}
                            }

                            // Description
                            span {
                                class: "command-completion-desc",
                                "{item.description}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render a command name with fuzzy-match indices highlighted.
#[allow(clippy::indexing_slicing)] // Char indices are bounded by the chars vec length
fn render_highlighted_name(name: &str, indices: &[usize]) -> Element {
    if indices.is_empty() {
        return rsx! { span { "{name}" } };
    }

    let chars: Vec<char> = name.chars().collect();
    let indices_set: std::collections::HashSet<usize> = indices.iter().copied().collect();

    let mut segments: Vec<Element> = Vec::new();
    let mut current_start = 0;
    let mut in_highlight = false;

    for (i, _) in chars.iter().enumerate() {
        let is_match = indices_set.contains(&i);

        if i == 0 {
            in_highlight = is_match;
        } else if is_match != in_highlight {
            let segment_text: String = chars[current_start..i].iter().collect();
            if in_highlight {
                segments.push(rsx! {
                    span { key: "{current_start}", class: "command-completion-match", "{segment_text}" }
                });
            } else {
                segments.push(rsx! {
                    span { key: "{current_start}", "{segment_text}" }
                });
            }
            current_start = i;
            in_highlight = is_match;
        }
    }

    // Emit final segment.
    let segment_text: String = chars[current_start..].iter().collect();
    if in_highlight {
        segments.push(rsx! {
            span { key: "{current_start}", class: "command-completion-match", "{segment_text}" }
        });
    } else {
        segments.push(rsx! {
            span { key: "{current_start}", "{segment_text}" }
        });
    }

    rsx! {
        span { {segments.into_iter()} }
    }
}
