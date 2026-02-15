//! Picker preview panel component.
//!
//! Renders syntax-highlighted file content with a focus line indicator,
//! displayed as the right panel in the picker overlay.

use dioxus::prelude::*;

use crate::state::{PickerPreview, PreviewLine, TokenSpan};

/// Render syntax-highlighted tokens for a preview line.
/// Simplified version of `render_styled_content` without cursor/selection logic.
#[allow(clippy::indexing_slicing)] // Char indices are bounded by len checks above each slice
fn render_preview_tokens(content: &str, tokens: &[TokenSpan]) -> Element {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();

    if tokens.is_empty() {
        return rsx! {
            span { "{content}" }
        };
    }

    let mut sorted_tokens = tokens.to_vec();
    sorted_tokens.sort_by_key(|t| t.start);

    let mut spans: Vec<Element> = Vec::new();
    let mut pos = 0;

    for token in &sorted_tokens {
        let start = token.start.min(len);
        let end = token.end.min(len);

        // Emit unstyled gap before this token
        if start > pos {
            let text: String = chars[pos..start].iter().collect();
            spans.push(rsx! { span { "{text}" } });
        }

        // Emit styled token
        if end > start {
            let text: String = chars[start..end].iter().collect();
            let color = &token.color;
            spans.push(rsx! {
                span { style: "color: {color};", "{text}" }
            });
        }

        pos = end;
    }

    // Emit remaining unstyled text
    if pos < len {
        let text: String = chars[pos..].iter().collect();
        spans.push(rsx! { span { "{text}" } });
    }

    rsx! {
        {spans.into_iter()}
    }
}

/// Render a preview line with optional search match highlighting.
fn render_line_content(line: &PreviewLine, search_pattern: Option<&str>) -> Element {
    // If there's a search pattern, highlight matches within the already-tokenized content
    if let Some(pattern) = search_pattern {
        if !pattern.is_empty() {
            return render_with_search_highlight(&line.content, &line.tokens, pattern);
        }
    }
    render_preview_tokens(&line.content, &line.tokens)
}

/// Render preview tokens with search matches highlighted.
#[allow(clippy::indexing_slicing)] // Char indices are bounded by len/min checks throughout
fn render_with_search_highlight(content: &str, tokens: &[TokenSpan], pattern: &str) -> Element {
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let pattern_lower = pattern.to_lowercase();

    // Find all search match ranges (case-insensitive)
    let content_lower = content.to_lowercase();
    let mut matches: Vec<(usize, usize)> = Vec::new();
    let mut search_start = 0;
    while let Some(byte_pos) = content_lower[search_start..].find(&pattern_lower) {
        let byte_start = search_start + byte_pos;
        let byte_end = byte_start + pattern_lower.len();
        // Convert byte positions to char positions
        let char_start = content[..byte_start].chars().count();
        let char_end = content[..byte_end].chars().count();
        matches.push((char_start, char_end));
        search_start = byte_end;
    }

    if matches.is_empty() {
        return render_preview_tokens(content, tokens);
    }

    // Build sorted tokens
    let mut sorted_tokens = tokens.to_vec();
    sorted_tokens.sort_by_key(|t| t.start);

    let mut spans: Vec<Element> = Vec::new();
    let mut pos = 0;

    // Walk through characters, emitting styled spans
    while pos < len {
        // Check if we're inside a search match
        let in_match = matches.iter().find(|&&(s, e)| pos >= s && pos < e);

        if let Some(&(_, match_end)) = in_match {
            // Emit the match range with highlight background
            let end = match_end.min(len);
            let text: String = chars[pos..end].iter().collect();
            spans.push(rsx! {
                span {
                    class: "picker-preview-search-match",
                    "{text}"
                }
            });
            pos = end;
        } else {
            // Find next match start
            let next_match_start = matches
                .iter()
                .filter(|&&(s, _)| s > pos)
                .map(|&(s, _)| s)
                .min()
                .unwrap_or(len);

            // Find active token at this position
            let active_token = sorted_tokens.iter().find(|t| t.start <= pos && pos < t.end);

            let segment_end = if let Some(token) = active_token {
                token.end.min(next_match_start).min(len)
            } else {
                // Find next token start
                let next_token = sorted_tokens.iter().find(|t| t.start > pos);
                match next_token {
                    Some(t) => t.start.min(next_match_start).min(len),
                    None => next_match_start.min(len),
                }
            };

            let text: String = chars[pos..segment_end].iter().collect();
            if let Some(token) = active_token {
                let color = &token.color;
                spans.push(rsx! {
                    span { style: "color: {color};", "{text}" }
                });
            } else {
                spans.push(rsx! { span { "{text}" } });
            }
            pos = segment_end;
        }
    }

    rsx! {
        {spans.into_iter()}
    }
}

/// File preview panel for the picker.
#[component]
pub fn PickerPreviewPanel(preview: PickerPreview) -> Element {
    rsx! {
        div {
            class: "picker-preview",

            // Header with file path
            div {
                class: "picker-preview-header",
                "{preview.file_path}"
            }

            // Content area with lines
            div {
                class: "picker-preview-content",

                for line in preview.lines.iter() {
                    {
                        let line_class = if line.is_focus_line {
                            "picker-preview-line picker-preview-line-focus"
                        } else {
                            "picker-preview-line"
                        };
                        let line_num = line.line_number;
                        rsx! {
                            div {
                                key: "{line_num}",
                                class: "{line_class}",

                                // Line number gutter
                                span {
                                    class: "picker-preview-gutter",
                                    "{line_num}"
                                }

                                // Code content
                                span {
                                    class: "picker-preview-code",
                                    {render_line_content(line, preview.search_pattern.as_deref())}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
