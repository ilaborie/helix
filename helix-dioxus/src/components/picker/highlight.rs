//! Text highlighting component for fuzzy match results.

use dioxus::prelude::*;

/// Text with highlighted match indices.
#[component]
pub fn HighlightedText(text: String, indices: Vec<usize>, base_color: String) -> Element {
    if indices.is_empty() {
        return rsx! {
            span {
                class: "highlight-text",
                style: "color: {base_color};",
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
                            class: "highlight-match",
                            "{segment_text}"
                        }
                    });
                } else {
                    segments.push(rsx! {
                        span {
                            key: "{current_start}",
                            class: "highlight-text",
                            style: "color: {base_color};",
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
                    class: "highlight-match",
                    "{segment_text}"
                }
            });
        } else {
            segments.push(rsx! {
                span {
                    key: "{current_start}",
                    class: "highlight-text",
                    style: "color: {base_color};",
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
