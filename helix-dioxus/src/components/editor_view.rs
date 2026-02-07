//! Document rendering component.
//!
//! Renders the document content with syntax highlighting and cursor display.

use dioxus::prelude::*;
use lucide_dioxus::Lightbulb;

use crate::components::{
    diagnostics_for_line, first_diagnostic_for_line, highest_severity_for_line, DiagnosticMarker,
    DiagnosticUnderline, ErrorLens, Scrollbar,
};
use crate::lsp::DiagnosticSnapshot;
use crate::state::{LineSnapshot, TokenSpan};
use crate::AppState;

/// Editor view component that renders the document content.
#[component]
pub fn EditorView(version: ReadSignal<usize>) -> Element {
    // Read the signal to subscribe to changes
    let version = version();

    let app_state = use_context::<AppState>();
    let snapshot = app_state.get_snapshot();

    let mode = &snapshot.mode;
    let diagnostics = &snapshot.diagnostics;

    // Scroll cursor into view after each render when version changes
    use_effect(move || {
        // version is used to trigger the effect on each state change
        let _ = version;
        document::eval("scrollCursorIntoView();");
    });

    let has_code_actions = snapshot.has_code_actions;
    let cursor_line = snapshot.cursor_line;

    rsx! {
        div {
            class: "editor-view",

            // Unified indicator gutter (breakpoints, diagnostics, code actions)
            // Uses absolute positioning to allow multiple overlapping indicators
            div {
                class: "indicator-gutter",
                for line in &snapshot.lines {
                    {
                        let line_num = line.line_number;
                        let show_lightbulb = has_code_actions && line_num == cursor_line;
                        let severity = highest_severity_for_line(diagnostics, line_num);
                        let key = format!("ind-{}-{}-{}-{}", line_num, version, show_lightbulb, severity.is_some());
                        // Use diagnostic severity color if available, otherwise yellow
                        let lightbulb_color = severity
                            .map(|s| s.css_color())
                            .unwrap_or("#e5c07b");
                        rsx! {
                            div {
                                key: "{key}",
                                class: "indicator-gutter-line",

                                // Single indicator at bottom-right:
                                // - Lightbulb (severity-colored) when code actions available
                                // - Diagnostic marker when no code actions but has diagnostic
                                if show_lightbulb {
                                    span {
                                        class: "indicator-diagnostic icon-wrapper",
                                        title: "Code actions available (Ctrl+Space)",
                                        Lightbulb { size: 10, color: lightbulb_color }
                                    }
                                } else if let Some(sev) = severity {
                                    span {
                                        class: "indicator-diagnostic",
                                        DiagnosticMarker { severity: sev }
                                    }
                                }

                                // Future: Breakpoint indicator (center)
                                // Future: Jumplist number (bottom-left)
                            }
                        }
                    }
                }
            }

            // Line numbers gutter
            div {
                class: "gutter",
                for line in &snapshot.lines {
                    // Include version and cursor state in key to force re-render
                    {
                        let is_cursor = line.is_cursor_line;
                        let gutter_key = format!("{}-{}-{}", line.line_number, version, is_cursor);
                        let gutter_class = if is_cursor { "gutter-line-active" } else { "gutter-line" };
                        rsx! {
                            div {
                                key: "{gutter_key}",
                                class: "{gutter_class}",
                                "{line.line_number}"
                            }
                        }
                    }
                }
            }

            // Document content
            div {
                class: "content",
                for (idx, line) in snapshot.lines.iter().enumerate() {
                    // Include version and selection state in key to force re-render
                    {
                        let has_sel = line.selection_range.is_some();
                        let line_num = line.line_number;
                        // Get all diagnostics for this line (for underlines)
                        // Sort by severity ascending so higher severity renders last (on top)
                        let mut line_diags: Vec<_> = diagnostics_for_line(diagnostics, line_num)
                            .into_iter()
                            .cloned()
                            .collect();
                        line_diags.sort_by_key(|d| d.severity);
                        // Get highest severity diagnostic for ErrorLens
                        let primary_diag = first_diagnostic_for_line(diagnostics, line_num).cloned();

                        // Check if the next line is empty and has a diagnostic we should show here
                        let next_line_diag = if idx + 1 < snapshot.lines.len() {
                            let next_line = &snapshot.lines[idx + 1];
                            let next_content = next_line.content.trim();
                            // If next line is empty/whitespace and has a diagnostic, show it on this line
                            if next_content.is_empty() {
                                first_diagnostic_for_line(diagnostics, next_line.line_number).cloned()
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        // Use next line's diagnostic for ErrorLens if this line has content but no diagnostic
                        // and the next line is empty with a diagnostic
                        let error_lens_diag = primary_diag.or(next_line_diag);

                        let key = format!("{}-{}-{}", line_num, version, has_sel);
                        rsx! {
                            Line {
                                key: "{key}",
                                line: line.clone(),
                                mode: mode.clone(),
                                diagnostics: line_diags,
                                error_lens_diagnostic: error_lens_diag,
                            }
                        }
                    }
                }
            }

            // Scrollbar on the right edge
            Scrollbar {
                total_lines: snapshot.total_lines,
                visible_start: snapshot.visible_start,
                viewport_lines: 40,
                all_diagnostics: snapshot.all_diagnostics_summary.clone(),
                search_match_lines: snapshot.search_match_lines.clone(),
            }
        }
    }
}

/// Individual line component with cursor and syntax highlighting rendering.
///
/// - `diagnostics`: All diagnostics for THIS line (used for underlines)
/// - `error_lens_diagnostic`: The diagnostic to show as ErrorLens (may be from next empty line)
#[component]
fn Line(
    line: LineSnapshot,
    mode: String,
    diagnostics: Vec<DiagnosticSnapshot>,
    error_lens_diagnostic: Option<DiagnosticSnapshot>,
) -> Element {
    // Remove trailing newline for display
    let display_content = line.content.trim_end_matches('\n');
    let chars: Vec<char> = display_content.chars().collect();

    let cursor_style = match mode.as_str() {
        "INSERT" => "background-color: transparent; box-shadow: -2px 0 0 0 #61afef;",
        "SELECT" => "background-color: #c678dd; color: #282c34; position: relative; z-index: 1; box-shadow: 0 0 0 2px #c678dd;",
        _ => "background-color: #61afef; color: #282c34; position: relative; z-index: 1; box-shadow: 0 0 0 2px #61afef;",
    };

    // Determine line class:
    // - Apply selection background at the LINE level to avoid gaps between lines
    // - Non-selected parts of the line will be masked with normal background in render_styled_content
    // - Cursor line gets highlight only if no selection
    let has_selection = line.selection_range.is_some();
    let line_class = if has_selection {
        "line line-selected"
    } else if line.is_cursor_line {
        "line line-cursor"
    } else {
        "line"
    };

    // Build sorted list of styling events (token starts/ends and cursor position)
    let cursor_pos = if line.is_cursor_line {
        line.cursor_col
    } else {
        None
    };

    // Only show ErrorLens if this line has content (not empty/whitespace)
    // This prevents showing ErrorLens on empty lines - it will be shown on the previous line instead
    let show_error_lens = !display_content.trim().is_empty();

    // Render the line content with tokens, cursor, and selection highlighting
    rsx! {
        div {
            class: "{line_class}",
            {render_styled_content(&chars, &line.tokens, cursor_pos, cursor_style, line.selection_range)}
            // Diagnostic underlines (wavy lines under errors/warnings)
            for (idx, diag) in diagnostics.iter().enumerate() {
                DiagnosticUnderline {
                    key: "{idx}",
                    start_col: diag.start_col,
                    end_col: diag.end_col,
                    severity: diag.severity,
                }
            }
            // Error Lens: Show diagnostic message at end of line (only if line has content)
            if show_error_lens {
                if let Some(diag) = error_lens_diagnostic {
                    ErrorLens { diagnostic: diag }
                }
            }
        }
    }
}

/// Render content with syntax highlighting tokens, cursor, and selection.
fn render_styled_content(
    chars: &[char],
    tokens: &[TokenSpan],
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
