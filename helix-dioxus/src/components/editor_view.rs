//! Document rendering component.
//!
//! Renders the document content with syntax highlighting and cursor display.

use std::fmt::Write as _;

use crate::icons::{lucide, Icon};
use dioxus::prelude::*;

use crate::components::{
    diagnostics_for_line, first_diagnostic_for_line, highest_severity_for_line, DiagnosticMarker, DiagnosticUnderline,
    ErrorLens, Scrollbar,
};
use crate::hooks::{use_snapshot, use_snapshot_signal};
use crate::lsp::{DiagnosticSnapshot, InlayHintKind, InlayHintSnapshot};
use crate::state::{DiffLineType, EditorCommand, LineSnapshot, TokenSpan, WhitespaceSnapshot, WordJumpLabel};
use crate::AppState;

/// Editor view component that renders the document content.
#[component]
pub fn EditorView() -> Element {
    let snapshot = use_snapshot();
    let version = snapshot.snapshot_version;
    let app_state = use_context::<AppState>();
    let mut snapshot_signal = use_snapshot_signal();

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

    let soft_wrap_class = if snapshot.soft_wrap {
        "editor-view soft-wrap"
    } else {
        "editor-view"
    };

    rsx! {
        // Wrapper: positions the scrollbar alongside the grid
        div {
            class: "editor-view-wrapper",

            div {
                class: "{soft_wrap_class}",

                // CSS Grid: each line produces 3 cells (indicator, gutter, content)
                // Grid rows auto-size so wrapped content keeps gutter aligned
                for (idx, line) in snapshot.lines.iter().enumerate() {
                    {
                        let line_num = line.line_number;

                        // --- Cell 1: Indicator gutter ---
                        let show_lightbulb = has_code_actions && line_num == cursor_line;
                        let severity = highest_severity_for_line(diagnostics, line_num);
                        let has_jump = snapshot.jump_lines.contains(&line_num);
                        let lightbulb_color = severity.map_or("var(--warning)", |s| s.css_color());

                        // --- Cell 2: Line number gutter ---
                        let is_cursor = line.is_cursor_line;
                        let diff_type = snapshot.diff_lines.iter().find(|(l, _)| *l == line_num).map(|(_, dt)| *dt);
                        let gutter_class = if is_cursor { "gutter-cell gutter-line-active" } else { "gutter-cell gutter-line" };

                        // --- Cell 3: Content ---
                        let has_sel = !line.selection_ranges.is_empty();
                        let mut line_diags: Vec<_> = diagnostics_for_line(diagnostics, line_num)
                            .into_iter()
                            .cloned()
                            .collect();
                        line_diags.sort_by_key(|d| d.severity);
                        let primary_diag = first_diagnostic_for_line(diagnostics, line_num).cloned();

                        #[allow(clippy::indexing_slicing)]
                        let next_line_diag = if idx + 1 < snapshot.lines.len() {
                            let next_line = &snapshot.lines[idx + 1];
                            let next_content = next_line.content.trim();
                            if next_content.is_empty() {
                                first_diagnostic_for_line(diagnostics, next_line.line_number).cloned()
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        let error_lens_diag = primary_diag.or(next_line_diag);

                        let line_jump_labels: Vec<WordJumpLabel> = snapshot.word_jump_labels
                            .iter()
                            .filter(|l| l.line == line_num)
                            .cloned()
                            .collect();

                        let has_jump_labels = !line_jump_labels.is_empty();
                        let has_hints = !line.inlay_hints.is_empty();

                        // Single key on first cell — Dioxus uses it to diff the entire block
                        let row_key = format!("row-{line_num}-{version}-{show_lightbulb}-{is_cursor}-{has_sel}-{has_jump_labels}-{has_hints}-{diff_type:?}");

                        rsx! {
                            // Cell 1: Indicator
                            div {
                                key: "{row_key}",
                                class: "indicator-cell",

                                if show_lightbulb {
                                    span {
                                        class: "indicator-diagnostic icon-wrapper",
                                        style: "color: {lightbulb_color};",
                                        title: "Code actions available (Ctrl+Space)",
                                        Icon { data: lucide::Lightbulb, size: "10", fill: "currentColor" }
                                    }
                                } else if let Some(sev) = severity {
                                    span {
                                        class: "indicator-diagnostic",
                                        DiagnosticMarker { severity: sev }
                                    }
                                }

                                if has_jump {
                                    span {
                                        class: "indicator-jumplist icon-wrapper",
                                        style: "color: var(--orange);",
                                        title: "Jump list entry (C-o/C-i)",
                                        Icon { data: lucide::Bookmark, size: "10", fill: "currentColor" }
                                    }
                                }
                            }

                            // Cell 2: Line number or wrap indicator
                            div {
                                class: "{gutter_class}",
                                if let Some(ref indicator) = line.wrap_indicator {
                                    span { class: "wrap-indicator", "{indicator}" }
                                } else {
                                    "{line_num}"
                                    if let Some(dt) = diff_type {
                                        {
                                            let app_state_hover = app_state.clone();
                                            let app_state_leave = app_state.clone();
                                            let hover_line = line_num;
                                            rsx! {
                                                span {
                                                    class: "gutter-diff-zone",
                                                    "data-diff-line": "{hover_line}",
                                                    onmouseenter: move |_| {
                                                        app_state_hover.send_command(EditorCommand::ShowGitDiffHover(hover_line));
                                                        app_state_hover.process_and_notify(&mut snapshot_signal);
                                                    },
                                                    onmouseleave: move |_| {
                                                        app_state_leave.send_command(EditorCommand::CloseGitDiffHover);
                                                        app_state_leave.process_and_notify(&mut snapshot_signal);
                                                    },
                                                    if dt == DiffLineType::Deleted {
                                                        span { class: "gutter-diff-deleted" }
                                                    } else {
                                                        span {
                                                            class: "gutter-diff-bar",
                                                            style: "background-color: {dt.css_color()};",
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Cell 3: Content
                            div {
                                class: "content-cell",

                                Line {
                                    line: line.clone(),
                                    mode: mode.clone(),
                                    whitespace: snapshot.whitespace.clone(),
                                    diagnostics: line_diags,
                                    error_lens_diagnostic: error_lens_diag,
                                    jump_labels: line_jump_labels,
                                }
                            }
                        }
                    }
                }

                // Rulers (vertical column guides) — absolutely positioned over content area
                // Offset accounts for indicator(18px) + gutter(~60px) + content padding(8px)
                for &col in &snapshot.rulers {
                    div {
                        key: "ruler-{col}",
                        class: "ruler",
                        style: "left: calc(86px + {col}ch);",
                    }
                }
            }

            // Scrollbar: sibling of the grid, positioned by the flex wrapper
            Scrollbar {
                total_lines: snapshot.total_lines,
                visible_start: snapshot.visible_start,
                viewport_lines: snapshot.viewport_lines,
                all_diagnostics: snapshot.all_diagnostics_summary.clone(),
                search_match_lines: snapshot.search_match_lines.clone(),
            }
        }
    }
}

/// Individual line component with cursor and syntax highlighting rendering.
///
/// - `diagnostics`: All diagnostics for THIS line (used for underlines)
/// - `error_lens_diagnostic`: The diagnostic to show as `ErrorLens` (may be from next empty line)
#[component]
fn Line(
    line: LineSnapshot,
    mode: String,
    #[props(default)] whitespace: WhitespaceSnapshot,
    diagnostics: Vec<DiagnosticSnapshot>,
    error_lens_diagnostic: Option<DiagnosticSnapshot>,
    #[props(default)] jump_labels: Vec<WordJumpLabel>,
) -> Element {
    // Remove trailing newline for display
    let display_content = line.content.trim_end_matches('\n');
    let chars: Vec<char> = display_content.chars().collect();

    let cursor_class = match mode.as_str() {
        "INSERT" => "cursor-line-insert",
        "SELECT" => "cursor-block-select",
        _ => "cursor-block-normal",
    };

    // Determine line class:
    // - Apply selection background at the LINE level to avoid gaps between lines
    // - Non-selected parts of the line will be masked with normal background in render_styled_content
    // - Cursor line gets highlight only if no selection
    let has_selection = !line.selection_ranges.is_empty();
    let line_class = if has_selection {
        "line line-selected"
    } else if line.is_cursor_line {
        "line line-cursor"
    } else {
        "line"
    };

    let selection_ranges = &line.selection_ranges;

    let cursor_cols = &line.cursor_cols;
    let primary_cursor_col = line.primary_cursor_col;

    let secondary_cursor_class = match mode.as_str() {
        "INSERT" => "cursor-secondary-insert",
        "SELECT" => "cursor-secondary-select",
        _ => "cursor-secondary-normal",
    };

    // Only show ErrorLens if this line has content (not empty/whitespace)
    // This prevents showing ErrorLens on empty lines - it will be shown on the previous line instead
    let show_error_lens = !display_content.trim().is_empty();

    let inlay_hints = &line.inlay_hints;

    // Render the line content with tokens, cursor, and selection highlighting
    rsx! {
        div {
            class: "{line_class}",
            {render_styled_content(&chars, &line.tokens, cursor_cols, primary_cursor_col, cursor_class, secondary_cursor_class, selection_ranges, &whitespace, inlay_hints)}
            // Visible newline character
            if let Some(nl_char) = whitespace.newline {
                if line.content.ends_with('\n') {
                    span { class: "whitespace-char", "{nl_char}" }
                }
            }
            // Diagnostic underlines (wavy lines under errors/warnings)
            // Use visual_col to offset for inline inlay hints
            for (idx, diag) in diagnostics.iter().enumerate() {
                DiagnosticUnderline {
                    key: "{idx}",
                    start_col: visual_col(diag.start_col, inlay_hints),
                    end_col: visual_col(diag.end_col, inlay_hints),
                    severity: diag.severity,
                }
            }
            // Error Lens: Show diagnostic message at end of line (only if line has content)
            if show_error_lens {
                if let Some(diag) = error_lens_diagnostic {
                    ErrorLens { diagnostic: diag }
                }
            }
            // Word jump labels overlay
            for label in jump_labels.iter() {
                {
                    let left_ch = visual_col(label.col, inlay_hints);
                    let label_text = format!("{}{}", label.label[0], label.label[1]);
                    let class = if label.dimmed { "jump-label-dimmed" } else { "jump-label" };
                    // Position using ch units relative to content start
                    let style = format!("position: absolute; left: {left_ch}ch; z-index: 10;");
                    rsx! {
                        span {
                            class: "{class}",
                            style: "{style}",
                            "{label_text}"
                        }
                    }
                }
            }
            // Color swatches overlay
            for (idx, swatch) in line.color_swatches.iter().enumerate() {
                {
                    let swatch_left = visual_col(swatch.col, inlay_hints);
                    rsx! {
                        span {
                            key: "swatch-{idx}",
                            class: "color-swatch",
                            style: "left: {swatch_left}ch; background-color: {swatch.color};",
                        }
                    }
                }
            }
        }
    }
}

/// Compute visual column accounting for inline inlay hint characters.
///
/// Hints inserted before or at a logical column shift all subsequent positions
/// to the right. Each hint contributes its label length plus any padding.
#[must_use]
fn visual_col(logical_col: usize, hints: &[InlayHintSnapshot]) -> usize {
    let offset: usize = hints
        .iter()
        .filter(|h| h.column <= logical_col)
        .map(|h| h.label.chars().count() + usize::from(h.padding_left) + usize::from(h.padding_right))
        .sum();
    logical_col + offset
}

/// Render content with syntax highlighting tokens, cursors, selection, and inlay hints.
#[allow(clippy::indexing_slicing, clippy::too_many_arguments)]
fn render_styled_content(
    chars: &[char],
    tokens: &[TokenSpan],
    cursor_cols: &[usize],
    primary_cursor_col: Option<usize>,
    primary_cursor_class: &str,
    secondary_cursor_class: &str,
    selection_ranges: &[(usize, usize)],
    whitespace: &WhitespaceSnapshot,
    inlay_hints: &[InlayHintSnapshot],
) -> Element {
    // Build a list of spans to render
    let mut spans: Vec<Element> = Vec::new();
    let mut pos = 0;
    let len = chars.len();

    // Sort tokens by start position
    let mut sorted_tokens = tokens.to_vec();
    sorted_tokens.sort_by_key(|t| t.start);

    // Sort inlay hints by column for insertion
    let mut sorted_hints = inlay_hints.to_vec();
    sorted_hints.sort_by_key(|h| h.column);

    let mut token_idx = 0;
    let mut hint_idx = 0;

    while pos <= len {
        // Insert any inlay hints positioned at the current column
        while hint_idx < sorted_hints.len() && sorted_hints[hint_idx].column == pos {
            let hint = &sorted_hints[hint_idx];
            let kind_class = match hint.kind {
                InlayHintKind::Type => "inlay-hint-type",
                InlayHintKind::Parameter => "inlay-hint-param",
            };
            let mut hint_text = String::new();
            if hint.padding_left {
                hint_text.push(' ');
            }
            hint_text.push_str(&hint.label);
            if hint.padding_right {
                hint_text.push(' ');
            }
            let hint_key = format!("hint-{pos}-{hint_idx}");
            spans.push(rsx! {
                span {
                    key: "{hint_key}",
                    class: "inlay-hint {kind_class}",
                    "{hint_text}"
                }
            });
            hint_idx += 1;
        }

        // Find the next boundary (token start, token end, cursor, selection bounds, hint column, or end of line)
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

        // Check for cursor positions (all cursors create boundaries)
        for &cursor in cursor_cols {
            if cursor > pos && cursor < next_pos {
                next_pos = cursor;
            } else if cursor == pos {
                next_pos = next_pos.min(pos + 1);
            }
        }

        // Check for selection boundaries
        for &(sel_start, sel_end) in selection_ranges {
            if sel_start > pos && sel_start < next_pos {
                next_pos = sel_start;
            }
            if sel_end > pos && sel_end < next_pos {
                next_pos = sel_end;
            }
        }

        // Check for next inlay hint column
        if hint_idx < sorted_hints.len() {
            let hint_col = sorted_hints[hint_idx].column;
            if hint_col > pos && hint_col < next_pos {
                next_pos = hint_col;
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

        // Determine if this is any cursor position
        let is_cursor = cursor_cols.contains(&pos);
        let is_primary = primary_cursor_col == Some(pos);

        // Determine if this position is within any selection range
        let is_selected = selection_ranges
            .iter()
            .any(|&(sel_start, sel_end)| pos >= sel_start && pos < sel_end);

        // Build the text content for this span, replacing whitespace chars
        let span_chars = &chars[pos..next_pos.min(len)];
        let has_visible_ws = span_chars.iter().any(|&ch| {
            (ch == ' ' && whitespace.space.is_some())
                || (ch == '\t' && whitespace.tab.is_some())
                || (ch == '\u{00A0}' && whitespace.nbsp.is_some())
        });
        let text: String = span_chars
            .iter()
            .map(|&ch| match ch {
                ' ' => whitespace.space.unwrap_or(ch),
                '\t' => whitespace.tab.unwrap_or(ch),
                '\u{00A0}' => whitespace.nbsp.unwrap_or(ch),
                _ => ch,
            })
            .collect();
        let text = if text.is_empty() && is_cursor {
            " ".to_string()
        } else {
            text
        };

        // Build style string
        let mut style = String::new();

        // For lines with selection, non-selected parts need normal background to "mask" the line-level selection
        // This approach: line has selection bg, non-selected spans get normal bg to hide it
        let line_has_selection = !selection_ranges.is_empty();
        if !is_selected && !is_cursor && line_has_selection {
            // Mask the line-level selection background with normal background
            style.push_str("background-color: var(--bg-primary);");
        }
        // Selected spans don't need explicit background since line already has it

        if let Some(token) = active_token {
            let _ = write!(style, "color: {};", token.color);
        }

        // Add the span (with id and class for cursor to enable scrollIntoView + CSS animation)
        if is_cursor {
            if is_primary {
                spans.push(
                    rsx! { span { key: "{pos}", id: "editor-cursor", class: "{primary_cursor_class}", style: "{style}", "{text}" } },
                );
            } else {
                spans.push(
                    rsx! { span { key: "{pos}", class: "{secondary_cursor_class}", style: "{style}", "{text}" } },
                );
            }
        } else if has_visible_ws {
            spans.push(rsx! { span { key: "{pos}", class: "whitespace-char", style: "{style}", "{text}" } });
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

    // Insert any remaining hints at end of line
    while hint_idx < sorted_hints.len() {
        let hint = &sorted_hints[hint_idx];
        let kind_class = match hint.kind {
            InlayHintKind::Type => "inlay-hint-type",
            InlayHintKind::Parameter => "inlay-hint-param",
        };
        let mut hint_text = String::new();
        if hint.padding_left {
            hint_text.push(' ');
        }
        hint_text.push_str(&hint.label);
        if hint.padding_right {
            hint_text.push(' ');
        }
        let hint_key = format!("hint-end-{hint_idx}");
        spans.push(rsx! {
            span {
                key: "{hint_key}",
                class: "inlay-hint {kind_class}",
                "{hint_text}"
            }
        });
        hint_idx += 1;
    }

    // Handle cursors at end of line
    for &cursor in cursor_cols {
        if cursor >= len {
            let is_primary = primary_cursor_col == Some(cursor);
            let cursor_end_key = format!("cursor-end-{cursor}");
            if is_primary {
                spans.push(
                    rsx! { span { key: "{cursor_end_key}", id: "editor-cursor", class: "{primary_cursor_class}", " " } },
                );
            } else {
                spans.push(rsx! { span { key: "{cursor_end_key}", class: "{secondary_cursor_class}", " " } });
            }
        }
    }

    rsx! { {spans.into_iter()} }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_hint(column: usize, label: &str, padding_left: bool, padding_right: bool) -> InlayHintSnapshot {
        InlayHintSnapshot {
            line: 1,
            column,
            label: label.to_string(),
            kind: InlayHintKind::Type,
            padding_left,
            padding_right,
        }
    }

    fn param_hint(column: usize, label: &str, padding_left: bool, padding_right: bool) -> InlayHintSnapshot {
        InlayHintSnapshot {
            line: 1,
            column,
            label: label.to_string(),
            kind: InlayHintKind::Parameter,
            padding_left,
            padding_right,
        }
    }

    #[test]
    fn visual_col_no_hints() {
        assert_eq!(visual_col(0, &[]), 0);
        assert_eq!(visual_col(5, &[]), 5);
        assert_eq!(visual_col(100, &[]), 100);
    }

    #[test]
    fn visual_col_single_type_hint() {
        // `let x = 42;` → hint `: i32` at column 5 (after `x`), padding_left=true
        let hints = vec![type_hint(5, ": i32", true, false)];
        // Columns before hint: unchanged
        assert_eq!(visual_col(0, &hints), 0);
        assert_eq!(visual_col(4, &hints), 4);
        // Column at hint position: shifted by label len (5) + padding_left (1)
        assert_eq!(visual_col(5, &hints), 5 + 5 + 1);
        // Column after hint: also shifted
        assert_eq!(visual_col(10, &hints), 10 + 5 + 1);
    }

    #[test]
    fn visual_col_single_param_hint() {
        // `foo(42)` → hint `value:` at column 4, padding_right=true
        let hints = vec![param_hint(4, "value:", false, true)];
        assert_eq!(visual_col(3, &hints), 3);
        // At column 4: shifted by "value:" (6) + padding_right (1) = 7
        assert_eq!(visual_col(4, &hints), 4 + 6 + 1);
    }

    #[test]
    fn visual_col_multiple_hints() {
        // Two hints on the same line
        let hints = vec![
            type_hint(3, ": u8", true, false),  // 4 chars + 1 padding = 5
            param_hint(8, "key:", false, true), // 4 chars + 1 padding = 5
        ];
        // Before first hint
        assert_eq!(visual_col(2, &hints), 2);
        // At first hint column: +5
        assert_eq!(visual_col(3, &hints), 3 + 5);
        // Between hints: only first applies
        assert_eq!(visual_col(5, &hints), 5 + 5);
        // At second hint: both apply
        assert_eq!(visual_col(8, &hints), 8 + 5 + 5);
        // After both
        assert_eq!(visual_col(12, &hints), 12 + 5 + 5);
    }

    #[test]
    fn visual_col_hint_at_column_zero() {
        let hints = vec![param_hint(0, "x:", false, true)];
        // Column 0 is at/after hint position → shifted
        assert_eq!(visual_col(0, &hints), 0 + 2 + 1);
        assert_eq!(visual_col(3, &hints), 3 + 2 + 1);
    }

    #[test]
    fn visual_col_hint_with_both_padding() {
        let hints = vec![type_hint(5, ": T", true, true)];
        // ": T" = 3 chars + padding_left (1) + padding_right (1) = 5
        assert_eq!(visual_col(5, &hints), 5 + 3 + 1 + 1);
    }

    #[test]
    fn visual_col_hint_no_padding() {
        let hints = vec![type_hint(5, ": i32", false, false)];
        // ": i32" = 5 chars, no padding
        assert_eq!(visual_col(5, &hints), 5 + 5);
        assert_eq!(visual_col(4, &hints), 4);
    }
}
