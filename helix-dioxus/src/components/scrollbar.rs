//! Custom scrollbar with diagnostic markers.

use dioxus::prelude::*;

use crate::lsp::DiagnosticSeverity;
use crate::state::{EditorCommand, ScrollbarDiagnostic};
use crate::AppState;

// Line height in pixels (1.5em at 14px font-size = 21px)
const LINE_HEIGHT: f64 = 21.0;
// Content padding in pixels (matches .content padding: 8px)
const CONTENT_PADDING: f64 = 8.0;

/// Information for displaying a marker tooltip.
#[derive(Debug, Clone, PartialEq)]
struct MarkerTooltip {
    /// The line number (0-indexed).
    line: usize,
    /// CSS top position string for the tooltip.
    top_position: String,
    /// The kind of marker.
    kind: MarkerKind,
}

#[derive(Debug, Clone, PartialEq)]
enum MarkerKind {
    Search,
    Diagnostic {
        severity: DiagnosticSeverity,
        message: String,
    },
}

/// Custom scrollbar component that displays diagnostic markers.
#[component]
pub fn Scrollbar(
    /// Total number of lines in the document.
    total_lines: usize,
    /// First visible line (0-indexed).
    visible_start: usize,
    /// Number of lines visible in the viewport.
    viewport_lines: usize,
    /// All diagnostics in the document (for markers).
    all_diagnostics: Vec<ScrollbarDiagnostic>,
    /// Line numbers with search matches.
    search_match_lines: Vec<usize>,
) -> Element {
    let app_state = use_context::<AppState>();

    // Store scrollbar height for click calculations
    let mut scrollbar_height = use_signal(|| 0.0_f64);

    // Track hovered marker for tooltip
    let mut hovered_marker = use_signal(|| None::<MarkerTooltip>);

    // Drag state for scrollbar thumb
    let mut is_dragging = use_signal(|| false);
    let mut drag_start_y = use_signal(|| 0.0_f64);
    let mut drag_start_scroll_ratio = use_signal(|| 0.0_f64);

    // Only show thumb if content exceeds viewport
    let needs_scrollbar = total_lines > viewport_lines;

    // Calculate thumb size as a percentage of track height
    #[allow(clippy::cast_precision_loss)]
    let thumb_height = if total_lines > 0 {
        ((viewport_lines as f64 / total_lines as f64) * 100.0).clamp(5.0, 100.0)
    } else {
        100.0
    };

    // Calculate thumb position as a percentage from top
    #[allow(clippy::cast_precision_loss)]
    let thumb_top = if total_lines > viewport_lines {
        (visible_start as f64 / (total_lines - viewport_lines) as f64) * (100.0 - thumb_height)
    } else {
        0.0
    };

    // Handle mousedown on track to scroll to that position
    let track_app_state = app_state.clone();
    let handle_mousedown = move |evt: MouseEvent| {
        // Get click position relative to the scrollbar element
        let click_y = evt.element_coordinates().y;

        // Use JavaScript to get editor-view height (parent container)
        let height = document::eval(r"document.querySelector('.editor-view')?.getBoundingClientRect().height || 0");

        // Clone for the async block
        let app_state_clone = track_app_state.clone();

        // TODO: This doesn't work - getBoundingClientRect returns 0 for scrollbar height
        // Need to investigate Dioxus desktop element sizing
        spawn(async move {
            if let Ok(val) = height.await {
                let scrollbar_h: f64 = val.as_f64().unwrap_or(0.0);

                if scrollbar_h > 16.0 {
                    // The track has 8px padding top and bottom
                    let track_top = 8.0;
                    let track_height = scrollbar_h - 16.0;

                    let relative_y = click_y - track_top;
                    let ratio = (relative_y / track_height).clamp(0.0, 1.0);
                    #[allow(
                        clippy::cast_precision_loss,
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss
                    )]
                    let target_line = (ratio * total_lines as f64) as usize;

                    app_state_clone.send_command(EditorCommand::GoToLine(target_line));
                    app_state_clone.process_commands_sync();
                }
            }
        });
    };

    // Handle mousedown on thumb to start dragging
    // TODO: Drag doesn't work - element height returns 0, see track mousedown TODO
    let handle_thumb_mousedown = move |evt: MouseEvent| {
        evt.prevent_default(); // Prevent text selection during drag
        evt.stop_propagation(); // Prevent track click
        is_dragging.set(true);
        drag_start_y.set(evt.page_coordinates().y);
        // Store current scroll position as ratio
        #[allow(clippy::cast_precision_loss)]
        let current_ratio = if total_lines > viewport_lines {
            visible_start as f64 / (total_lines - viewport_lines) as f64
        } else {
            0.0
        };
        drag_start_scroll_ratio.set(current_ratio);
    };

    // Handle mousemove on drag overlay for drag tracking
    let mousemove_app_state = app_state.clone();
    let handle_mousemove = move |evt: MouseEvent| {
        let current_y = evt.page_coordinates().y;
        let delta_y = current_y - drag_start_y();

        // Use JavaScript to get editor-view height (parent container)
        let height = document::eval(r"document.querySelector('.editor-view')?.getBoundingClientRect().height || 0");

        let app_state_clone = mousemove_app_state.clone();
        let start_ratio = drag_start_scroll_ratio();

        spawn(async move {
            if let Ok(val) = height.await {
                let scrollbar_h: f64 = val.as_f64().unwrap_or(0.0);
                let track_height = scrollbar_h - 16.0;

                if track_height > 0.0 {
                    // Convert pixel delta to ratio delta
                    let ratio_delta = delta_y / track_height;
                    let new_ratio = (start_ratio + ratio_delta).clamp(0.0, 1.0);

                    #[allow(
                        clippy::cast_precision_loss,
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss
                    )]
                    let target_line = (new_ratio * (total_lines.saturating_sub(viewport_lines)) as f64) as usize;

                    app_state_clone.send_command(EditorCommand::ScrollToLine(target_line));
                    app_state_clone.process_commands_sync();
                }
            }
        });
    };

    // Handle mouseup to end dragging
    let handle_mouseup = move |_evt: MouseEvent| {
        is_dragging.set(false);
    };

    // Use onmounted to capture the actual height of the scrollbar element
    // Note: This currently returns 0 - element not laid out yet at mount time
    let onmounted = move |evt: MountedEvent| async move {
        if let Ok(rect) = evt.get_client_rect().await {
            scrollbar_height.set(rect.height());
        }
    };

    // Sort diagnostics by severity (ascending) so errors render last (on top)
    let mut sorted_diagnostics = all_diagnostics.clone();
    sorted_diagnostics.sort_by_key(|d| d.severity);

    // Helper to calculate marker top position
    #[allow(clippy::cast_precision_loss)]
    let calc_marker_top = |line: usize| -> String {
        if needs_scrollbar {
            format!("{}%", (line as f64 / total_lines.max(1) as f64) * 100.0)
        } else {
            format!("{}px", CONTENT_PADDING + (line as f64 * LINE_HEIGHT))
        }
    };

    rsx! {
        // Full-screen overlay to capture mouse events during drag
        if is_dragging() {
            div {
                class: "scrollbar-drag-overlay",
                onmousemove: handle_mousemove,
                onmouseup: handle_mouseup,
            }
        }

        div {
            class: "editor-scrollbar",
            onmousedown: handle_mousedown,
            onmounted: onmounted,

            div {
                class: "scrollbar-track",

                // Search markers first (so diagnostics render on top)
                for (idx, &line) in search_match_lines.iter().enumerate() {
                    {
                        let marker_top = calc_marker_top(line);
                        let marker_top_for_tooltip = marker_top.clone();
                        let marker_app_state = app_state.clone();
                        let handle_click = move |evt: MouseEvent| {
                            evt.stop_propagation();
                            marker_app_state.send_command(EditorCommand::GoToLine(line));
                            marker_app_state.process_commands_sync();
                        };
                        let handle_mouseenter = move |_| {
                            hovered_marker.set(Some(MarkerTooltip {
                                line,
                                top_position: marker_top_for_tooltip.clone(),
                                kind: MarkerKind::Search,
                            }));
                        };
                        let handle_mouseleave = move |_| {
                            hovered_marker.set(None);
                        };
                        rsx! {
                            div {
                                key: "search-{idx}",
                                class: "scrollbar-marker scrollbar-marker-search",
                                style: "top: {marker_top};",
                                onclick: handle_click,
                                onmouseenter: handle_mouseenter,
                                onmouseleave: handle_mouseleave,
                            }
                        }
                    }
                }

                // Diagnostic markers (sorted by severity so errors render on top)
                for (idx, diag) in sorted_diagnostics.iter().enumerate() {
                    {
                        let marker_line = diag.line;
                        let marker_severity = diag.severity;
                        let marker_message = diag.message.clone();
                        let marker_top = calc_marker_top(marker_line);
                        let marker_top_for_tooltip = marker_top.clone();
                        let diag_app_state = app_state.clone();
                        let handle_click = move |evt: MouseEvent| {
                            evt.stop_propagation();
                            diag_app_state.send_command(EditorCommand::GoToLine(marker_line));
                            diag_app_state.process_commands_sync();
                        };
                        let tooltip_message = marker_message.clone();
                        let handle_mouseenter = move |_| {
                            hovered_marker.set(Some(MarkerTooltip {
                                line: marker_line,
                                top_position: marker_top_for_tooltip.clone(),
                                kind: MarkerKind::Diagnostic {
                                    severity: marker_severity,
                                    message: tooltip_message.clone(),
                                },
                            }));
                        };
                        let handle_mouseleave = move |_| {
                            hovered_marker.set(None);
                        };
                        let marker_class = match diag.severity {
                            DiagnosticSeverity::Error => "scrollbar-marker scrollbar-marker-error",
                            DiagnosticSeverity::Warning => "scrollbar-marker scrollbar-marker-warning",
                            DiagnosticSeverity::Info => "scrollbar-marker scrollbar-marker-info",
                            DiagnosticSeverity::Hint => "scrollbar-marker scrollbar-marker-hint",
                        };
                        rsx! {
                            div {
                                key: "diag-{idx}",
                                class: "{marker_class}",
                                style: "top: {marker_top};",
                                onclick: handle_click,
                                onmouseenter: handle_mouseenter,
                                onmouseleave: handle_mouseleave,
                            }
                        }
                    }
                }

                // Thumb (viewport indicator) - only show if content exceeds viewport
                if needs_scrollbar {
                    div {
                        class: if is_dragging() { "scrollbar-thumb scrollbar-thumb-dragging" } else { "scrollbar-thumb" },
                        style: "top: {thumb_top}%; height: {thumb_height}%;",
                        onmousedown: handle_thumb_mousedown,
                    }
                }
            }

            // Tooltip for hovered marker
            if let Some(tooltip) = hovered_marker() {
                {
                    let (severity_label, severity_class, message) = match &tooltip.kind {
                        MarkerKind::Search => {
                            ("Search match".to_string(), "scrollbar-tooltip-search", None)
                        }
                        MarkerKind::Diagnostic { severity, message } => {
                            let label = match severity {
                                DiagnosticSeverity::Error => "Error",
                                DiagnosticSeverity::Warning => "Warning",
                                DiagnosticSeverity::Info => "Info",
                                DiagnosticSeverity::Hint => "Hint",
                            };
                            let class = match severity {
                                DiagnosticSeverity::Error => "scrollbar-tooltip-error",
                                DiagnosticSeverity::Warning => "scrollbar-tooltip-warning",
                                DiagnosticSeverity::Info => "scrollbar-tooltip-info",
                                DiagnosticSeverity::Hint => "scrollbar-tooltip-hint",
                            };
                            (label.to_string(), class, Some(message.clone()))
                        }
                    };
                    // Truncate message for display
                    let display_message = message.map(|m| {
                        if m.len() > 80 {
                            format!("{}...", &m[..77])
                        } else {
                            m
                        }
                    });
                    // Display 1-indexed line number for user
                    let display_line = tooltip.line + 1;
                    let tooltip_top = &tooltip.top_position;
                    rsx! {
                        div {
                            class: "scrollbar-tooltip {severity_class}",
                            style: "top: {tooltip_top};",
                            div {
                                class: "scrollbar-tooltip-header",
                                span {
                                    class: "scrollbar-tooltip-severity",
                                    "{severity_label}"
                                }
                                span {
                                    class: "scrollbar-tooltip-line",
                                    "Line {display_line}"
                                }
                            }
                            if let Some(msg) = display_message {
                                div {
                                    class: "scrollbar-tooltip-message",
                                    "{msg}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
