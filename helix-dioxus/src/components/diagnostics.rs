//! Diagnostic display components.
//!
//! This module provides components for displaying LSP diagnostics:
//! - Gutter icons showing severity (E/W/I/H)
//! - Error Lens style inline messages at end of lines

use dioxus::prelude::*;
use lucide_dioxus::{Circle, CircleX, Info, Lightbulb, TriangleAlert};

use crate::lsp::{DiagnosticSeverity, DiagnosticSnapshot};

/// Renders a diagnostic marker icon in the gutter.
/// Uses small icons (10px) to fit in the compact indicator gutter.
#[component]
pub fn DiagnosticMarker(severity: DiagnosticSeverity) -> Element {
    let color = severity.css_color();

    rsx! {
        span {
            class: "diagnostic-marker icon-wrapper",
            style: "color: {color};",
            match severity {
                DiagnosticSeverity::Error => rsx! { CircleX { size: 10, color: "currentColor" } },
                DiagnosticSeverity::Warning => rsx! { TriangleAlert { size: 10, color: "currentColor" } },
                DiagnosticSeverity::Info => rsx! { Info { size: 10, color: "currentColor" } },
                DiagnosticSeverity::Hint => rsx! { Lightbulb { size: 10, color: "currentColor" } },
            }
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
            // Separator to distinguish from code
            span {
                class: "error-lens-separator",
                style: "opacity: 0.5; margin-right: 6px;",
                "//"
            }
            span {
                class: "icon-wrapper",
                style: "margin-right: 4px;",
                Circle { size: 8, color: "currentColor" }
            }
            "{message}"
        }
    }
}

/// Renders an underline for a diagnostic range.
/// This is rendered as a decoration under the text.
#[component]
pub fn DiagnosticUnderline(start_col: usize, end_col: usize, severity: DiagnosticSeverity) -> Element {
    let width = end_col.saturating_sub(start_col).max(1);
    let left_offset = start_col;

    // Use CSS class for severity-specific underline style
    let severity_class = match severity {
        DiagnosticSeverity::Error => "diagnostic-underline-error",
        DiagnosticSeverity::Warning => "diagnostic-underline-warning",
        DiagnosticSeverity::Info => "diagnostic-underline-info",
        DiagnosticSeverity::Hint => "diagnostic-underline-hint",
    };

    // Position via inline style, color via CSS class
    let style = format!("left: {left_offset}ch; width: {width}ch;");

    rsx! {
        span {
            class: "diagnostic-underline {severity_class}",
            style: "{style}",
        }
    }
}

/// Get the highest severity diagnostic for a line.
/// Used to determine which icon to show in the gutter when multiple diagnostics exist.
#[must_use]
pub fn highest_severity_for_line(diagnostics: &[DiagnosticSnapshot], line: usize) -> Option<DiagnosticSeverity> {
    diagnostics
        .iter()
        .filter(|diag| diag.line == line)
        .map(|diag| diag.severity)
        .max()
}

/// Get the first diagnostic message for a line (for Error Lens).
/// Returns the highest severity diagnostic message.
#[must_use]
pub fn first_diagnostic_for_line(diagnostics: &[DiagnosticSnapshot], line: usize) -> Option<&DiagnosticSnapshot> {
    diagnostics
        .iter()
        .filter(|diag| diag.line == line)
        .max_by_key(|diag| diag.severity)
}

/// Get all diagnostics for a line (for underlines).
/// Returns all diagnostics on the given line.
#[must_use]
pub fn diagnostics_for_line(diagnostics: &[DiagnosticSnapshot], line: usize) -> Vec<&DiagnosticSnapshot> {
    diagnostics.iter().filter(|diag| diag.line == line).collect()
}
