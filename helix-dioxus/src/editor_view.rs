//! Document rendering component.
//!
//! Renders the document content with syntax highlighting and cursor display.

use dioxus::prelude::*;

use crate::state::LineSnapshot;
use crate::AppState;

/// Editor view component that renders the document content.
#[component]
pub fn EditorView(version: ReadSignal<usize>) -> Element {
    // Read the signal to subscribe to changes
    let version = version();

    let app_state = use_context::<AppState>();
    let snapshot = app_state.get_snapshot();

    let mode = &snapshot.mode;

    // Scroll cursor into view after each render when version changes
    // Use requestAnimationFrame to ensure DOM is updated before scrolling
    use_effect(move || {
        // version is used to trigger the effect on each state change
        let _ = version;
        document::eval(
            r#"
            requestAnimationFrame(() => {
                const cursor = document.getElementById('editor-cursor');
                if (cursor) {
                    cursor.scrollIntoView({ block: 'nearest', inline: 'nearest' });
                }
            });
        "#,
        );
    });

    rsx! {
        div {
            class: "editor-view",
            style: "display: flex; height: 100%; overflow: hidden; font-size: 14px; line-height: 1.5;",

            // Line numbers gutter
            div {
                class: "gutter",
                style: "
                    padding: 8px 12px 8px 8px;
                    text-align: right;
                    color: #5c6370;
                    background-color: #21252b;
                    user-select: none;
                    min-width: 40px;
                ",
                for line in &snapshot.lines {
                    div {
                        key: "{line.line_number}",
                        style: if line.is_cursor_line {
                            "color: #abb2bf;"
                        } else {
                            ""
                        },
                        "{line.line_number}"
                    }
                }
            }

            // Document content
            div {
                class: "content",
                style: "
                    flex: 1;
                    padding: 8px;
                    overflow-x: auto;
                    overflow-y: auto;
                    white-space: pre;
                    user-select: none;
                ",
                for line in &snapshot.lines {
                    // Include version and selection state in key to force re-render
                    {
                        let has_sel = line.selection_range.is_some();
                        let key = format!("{}-{}-{}", line.line_number, version, has_sel);
                        rsx! {
                            Line {
                                key: "{key}",
                                line: line.clone(),
                                mode: mode.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Individual line component with cursor and syntax highlighting rendering.
#[component]
fn Line(line: LineSnapshot, mode: String) -> Element {
    // Remove trailing newline for display
    let display_content = line.content.trim_end_matches('\n');
    let chars: Vec<char> = display_content.chars().collect();

    let cursor_style = match mode.as_str() {
        "INSERT" => "background-color: transparent; box-shadow: -2px 0 0 0 #61afef;",
        "SELECT" => "background-color: #c678dd; color: #282c34;",
        _ => "background-color: #61afef; color: #282c34;", // Normal mode
    };

    // Determine line background:
    // - Apply selection background at the LINE level to avoid gaps between lines
    // - Non-selected parts of the line will be masked with normal background in render_styled_content
    // - Cursor line gets highlight only if no selection
    let has_selection = line.selection_range.is_some();
    let line_style = if has_selection {
        "background-color: #3e4451; min-height: 1.5em;"
    } else if line.is_cursor_line {
        "background-color: #2c313a; min-height: 1.5em;"
    } else {
        "min-height: 1.5em;"
    };

    // Build sorted list of styling events (token starts/ends and cursor position)
    let cursor_pos = if line.is_cursor_line {
        line.cursor_col
    } else {
        None
    };

    // Render the line content with tokens, cursor, and selection highlighting
    rsx! {
        div {
            class: "line",
            style: "{line_style}",
            {render_styled_content(&chars, &line.tokens, cursor_pos, cursor_style, line.selection_range)}
        }
    }
}

/// Render content with syntax highlighting tokens, cursor, and selection.
fn render_styled_content(
    chars: &[char],
    tokens: &[crate::state::TokenSpan],
    cursor_pos: Option<usize>,
    cursor_style: &str,
    selection_range: Option<(usize, usize)>,
) -> Element {
    // Build a list of spans to render
    let mut spans: Vec<Element> = Vec::new();
    let mut pos = 0;
    let len = chars.len();

    // Sort tokens by start position
    let mut sorted_tokens = tokens.to_vec();
    sorted_tokens.sort_by_key(|t| t.start);

    let mut token_idx = 0;

    while pos <= len {
        // Find the next boundary (token start, token end, cursor, selection bounds, or end of line)
        let mut next_pos = len;

        // Check for token boundaries
        for token in &sorted_tokens[token_idx..] {
            if token.start > pos {
                next_pos = next_pos.min(token.start);
                break;
            }
            if token.end > pos {
                next_pos = next_pos.min(token.end);
            }
        }

        // Check for cursor position
        if let Some(cursor) = cursor_pos {
            if cursor > pos && cursor < next_pos {
                next_pos = cursor;
            } else if cursor == pos {
                next_pos = next_pos.min(pos + 1);
            }
        }

        // Check for selection boundaries
        if let Some((sel_start, sel_end)) = selection_range {
            if sel_start > pos && sel_start < next_pos {
                next_pos = sel_start;
            }
            if sel_end > pos && sel_end < next_pos {
                next_pos = sel_end;
            }
        }

        if next_pos == pos {
            if pos >= len {
                break;
            }
            next_pos = pos + 1;
        }

        // Find active token at this position
        let active_token = sorted_tokens.iter().find(|t| t.start <= pos && pos < t.end);

        // Determine if this is the cursor position
        let is_cursor = cursor_pos == Some(pos);

        // Determine if this position is within the selection
        let is_selected = selection_range
            .map(|(sel_start, sel_end)| pos >= sel_start && pos < sel_end)
            .unwrap_or(false);

        // Build the text content for this span
        let text: String = chars[pos..next_pos.min(len)].iter().collect();
        let text = if text.is_empty() && is_cursor {
            " ".to_string()
        } else {
            text
        };

        // Build style string
        let mut style = String::new();

        // For lines with selection, non-selected parts need normal background to "mask" the line-level selection
        // This approach: line has selection bg, non-selected spans get normal bg to hide it
        let line_has_selection = selection_range.is_some();
        if !is_selected && !is_cursor && line_has_selection {
            // Mask the line-level selection background with normal background
            style.push_str("background-color: #282c34;");
        }
        // Selected spans don't need explicit background since line already has it

        if let Some(token) = active_token {
            style.push_str(&format!("color: {};", token.color));
        }
        if is_cursor {
            style.push_str(cursor_style);
        }

        // Add the span (with id for cursor to enable scrollIntoView)
        if is_cursor {
            spans.push(
                rsx! { span { key: "{pos}", id: "editor-cursor", style: "{style}", "{text}" } },
            );
        } else if style.is_empty() {
            spans.push(rsx! { span { key: "{pos}", "{text}" } });
        } else {
            spans.push(rsx! { span { key: "{pos}", style: "{style}", "{text}" } });
        }

        pos = next_pos;

        // Advance token index past completed tokens
        while token_idx < sorted_tokens.len() && sorted_tokens[token_idx].end <= pos {
            token_idx += 1;
        }
    }

    // Handle cursor at end of line
    if let Some(cursor) = cursor_pos {
        if cursor >= len {
            let style = cursor_style.to_string();
            let cursor_key = "cursor-end";
            spans.push(
                rsx! { span { key: "{cursor_key}", id: "editor-cursor", style: "{style}", " " } },
            );
        }
    }

    rsx! { {spans.into_iter()} }
}
