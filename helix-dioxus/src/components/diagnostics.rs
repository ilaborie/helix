//! Diagnostic display components.
//!
//! This module provides components for displaying LSP diagnostics:
//! - Gutter icons showing severity (E/W/I/H)
//! - Error Lens style inline messages at end of lines

use dioxus::prelude::*;

use crate::lsp::{DiagnosticSeverity, DiagnosticSnapshot};

/// Renders a diagnostic marker icon in the gutter.
#[component]
pub fn DiagnosticMarker(severity: DiagnosticSeverity) -> Element {
    let color = severity.css_color();
    let icon = severity.gutter_icon();

    rsx! {
        span {
            class: "diagnostic-marker",
            style: "color: {color};",
            "{icon}"
        }
    }
}

/// Renders an Error Lens style inline diagnostic message.
/// This appears at the end of the line with the diagnostic.
#[component]
pub fn ErrorLens(diagnostic: DiagnosticSnapshot) -> Element {
    let color = diagnostic.severity.css_color();
    // Truncate long messages for inline display
    let message = if diagnostic.message.len() > 80 {
        format!("{}...", &diagnostic.message[..77])
    } else {
        diagnostic.message.clone()
    };

    // Replace newlines with spaces for inline display
    let message = message.replace('\n', " ");

    rsx! {
        span {
            class: "error-lens",
            style: "color: {color};",
            " ● {message}"
        }
    }
}

/// Renders an underline for a diagnostic range.
/// This is rendered as a decoration under the text.
#[component]
pub fn DiagnosticUnderline(
    start_col: usize,
    end_col: usize,
    severity: DiagnosticSeverity,
) -> Element {
    let color = severity.css_color();
    let width = end_col.saturating_sub(start_col).max(1);
    let left_offset = start_col;

    // Use CSS to position the underline
    let style =
        format!("left: {left_offset}ch; width: {width}ch; border-bottom: 2px wavy {color};");

    rsx! {
        span {
            class: "diagnostic-underline",
            style: "{style}",
        }
    }
}

/// Get the highest severity diagnostic for a line.
/// Used to determine which icon to show in the gutter when multiple diagnostics exist.
pub fn highest_severity_for_line(
    diagnostics: &[DiagnosticSnapshot],
    line: usize,
) -> Option<DiagnosticSeverity> {
    diagnostics
        .iter()
        .filter(|diag| diag.line == line)
        .map(|diag| diag.severity)
        .max()
}

/// Get the first diagnostic message for a line (for Error Lens).
/// Returns the highest severity diagnostic message.
pub fn first_diagnostic_for_line(
    diagnostics: &[DiagnosticSnapshot],
    line: usize,
) -> Option<&DiagnosticSnapshot> {
    diagnostics
        .iter()
        .filter(|diag| diag.line == line)
        .max_by_key(|diag| diag.severity)
}
