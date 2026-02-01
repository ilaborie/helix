//! Custom scrollbar with diagnostic markers.

use dioxus::prelude::*;

use crate::lsp::DiagnosticSeverity;
use crate::state::{EditorCommand, ScrollbarDiagnostic};
use crate::AppState;

// Line height in pixels (1.5em at 14px font-size = 21px)
const LINE_HEIGHT: f64 = 21.0;
// Content padding in pixels (matches .content padding: 8px)
const CONTENT_PADDING: f64 = 8.0;

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

    // Handle click on track to scroll to that position
    // Use element_coordinates which gives position relative to the clicked element
    let handle_click = move |evt: MouseEvent| {
        // Get click position relative to the scrollbar element
        let click_y = evt.element_coordinates().y;

        // The track has 8px padding top and bottom
        // So track content starts at y=8 and ends at scrollbar_height - 8
        let track_top = 8.0;
        let track_height = scrollbar_height().max(16.0) - 16.0;

        if track_height > 0.0 {
            let relative_y = click_y - track_top;
            let ratio = (relative_y / track_height).clamp(0.0, 1.0);
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let target_line = (ratio * total_lines as f64) as usize;

            app_state.send_command(EditorCommand::ScrollToLine(target_line));
            app_state.process_commands_sync();
        }
    };

    // Use onmounted to capture the actual height of the scrollbar element
    let onmounted = move |evt: MountedEvent| async move {
        if let Ok(rect) = evt.get_client_rect().await {
            scrollbar_height.set(rect.height());
        }
    };

    // Sort diagnostics by severity (ascending) so errors render last (on top)
    let mut sorted_diagnostics = all_diagnostics.clone();
    sorted_diagnostics.sort_by_key(|d| d.severity);

    rsx! {
        div {
            class: "editor-scrollbar",
            onclick: handle_click,
            onmounted: onmounted,

            div {
                class: "scrollbar-track",

                // Search markers first (so diagnostics render on top)
                for (idx, &line) in search_match_lines.iter().enumerate() {
                    {
                        #[allow(clippy::cast_precision_loss)]
                        let marker_top = if needs_scrollbar {
                            format!("{}%", (line as f64 / total_lines.max(1) as f64) * 100.0)
                        } else {
                            format!("{}px", CONTENT_PADDING + (line as f64 * LINE_HEIGHT))
                        };
                        rsx! {
                            div {
                                key: "search-{idx}",
                                class: "scrollbar-marker scrollbar-marker-search",
                                style: "top: {marker_top};",
                            }
                        }
                    }
                }

                // Diagnostic markers (sorted by severity so errors render on top)
                for (idx, diag) in sorted_diagnostics.iter().enumerate() {
                    {
                        // Calculate marker position
                        // For small files: use pixel-based positioning aligned with line positions
                        // For large files: use percentage-based positioning
                        #[allow(clippy::cast_precision_loss)]
                        let marker_top = if needs_scrollbar {
                            // Percentage-based for large files
                            format!("{}%", (diag.line as f64 / total_lines.max(1) as f64) * 100.0)
                        } else {
                            // Pixel-based for small files (aligned with line)
                            format!("{}px", CONTENT_PADDING + (diag.line as f64 * LINE_HEIGHT))
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
                            }
                        }
                    }
                }

                // Thumb (viewport indicator) - only show if content exceeds viewport
                if needs_scrollbar {
                    div {
                        class: "scrollbar-thumb",
                        style: "top: {thumb_top}%; height: {thumb_height}%;",
                    }
                }
            }
        }
    }
}
