//! Inlay hints rendering.
//!
//! Provides functionality to render inline type hints.
//!
//! Note: These functions are prepared for LSP client integration but not yet used.

use crate::lsp::{InlayHintKind, InlayHintSnapshot};

/// Get inlay hints for a specific line.
#[allow(dead_code)]
pub fn hints_for_line(hints: &[InlayHintSnapshot], line: usize) -> Vec<&InlayHintSnapshot> {
    hints.iter().filter(|hint| hint.line == line).collect()
}

/// Format an inlay hint for display.
/// Returns the hint text with appropriate styling class.
#[allow(dead_code)]
pub fn format_hint(hint: &InlayHintSnapshot) -> (String, &'static str) {
    let class = match hint.kind {
        InlayHintKind::Type => "inlay-hint inlay-hint-type",
        InlayHintKind::Parameter => "inlay-hint inlay-hint-param",
    };

    let mut text = hint.label.clone();

    // Add padding if requested
    if hint.padding_left {
        text.insert(0, ' ');
    }
    if hint.padding_right {
        text.push(' ');
    }

    (text, class)
}
