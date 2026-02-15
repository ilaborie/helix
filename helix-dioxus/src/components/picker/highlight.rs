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

    let chars: Vec<char> = text.chars().collect();
    let indices_set: std::collections::HashSet<usize> = indices.iter().copied().collect();

    let mut segments: Vec<Element> = Vec::new();
    let mut current_start = 0;
    let mut in_highlight = false;

    let emit_segment = |start: usize, slice: &[char], highlight: bool, segments: &mut Vec<Element>| {
        let segment_text: String = slice.iter().collect();
        if highlight {
            segments.push(rsx! {
                span { key: "{start}", class: "highlight-match", "{segment_text}" }
            });
        } else {
            segments.push(rsx! {
                span { key: "{start}", class: "highlight-text", style: "color: {base_color};", "{segment_text}" }
            });
        }
    };

    for (i, _) in chars.iter().enumerate() {
        let is_match = indices_set.contains(&i);

        if i == 0 {
            in_highlight = is_match;
        } else if is_match != in_highlight {
            if let Some(slice) = chars.get(current_start..i) {
                emit_segment(current_start, slice, in_highlight, &mut segments);
            }
            current_start = i;
            in_highlight = is_match;
        }
    }

    // Emit final segment
    if let Some(slice) = chars.get(current_start..) {
        emit_segment(current_start, slice, in_highlight, &mut segments);
    }

    rsx! {
        span {
            {segments.into_iter()}
        }
    }
}
