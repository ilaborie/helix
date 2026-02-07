//! Base container for inline dialogs.
//!
//! Handles positioning and common styling for cursor-positioned popups.

use dioxus::prelude::*;

/// Position of the dialog relative to the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogPosition {
    /// Dialog appears above the cursor line.
    Above,
    /// Dialog appears below the cursor line.
    #[default]
    Below,
}

/// Configuration for dialog dimensions and constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogConstraints {
    /// Minimum width in pixels.
    pub min_width: Option<u32>,
    /// Maximum width in pixels.
    pub max_width: Option<u32>,
    /// Maximum height in pixels.
    pub max_height: Option<u32>,
}

impl Default for DialogConstraints {
    fn default() -> Self {
        Self {
            min_width: Some(200),
            max_width: Some(500),
            max_height: Some(300),
        }
    }
}

impl DialogConstraints {
    /// Generate CSS style string for the constraints.
    fn to_style(&self) -> String {
        let mut parts = Vec::new();
        if let Some(min_w) = self.min_width {
            parts.push(format!("min-width: {min_w}px"));
        }
        if let Some(max_w) = self.max_width {
            parts.push(format!("max-width: {max_w}px"));
        }
        if let Some(max_h) = self.max_height {
            parts.push(format!("max-height: {max_h}px"));
        }
        parts.join("; ")
    }
}

// Constants for position calculation
const LINE_HEIGHT: usize = 21;
const BUFFER_BAR_HEIGHT: usize = 40;
const CHAR_WIDTH: usize = 8;
const GUTTER_WIDTH: usize = 60;
const MIN_TOP: usize = 40;
const MAX_TOP: usize = 400;
const MAX_LEFT: usize = 600;

/// Calculate pixel position for the dialog.
fn calculate_position(
    cursor_line: usize,
    cursor_col: usize,
    position: DialogPosition,
) -> (usize, usize) {
    let top = match position {
        DialogPosition::Above => {
            let base = cursor_line.saturating_sub(1) * LINE_HEIGHT + BUFFER_BAR_HEIGHT;
            base.max(MIN_TOP)
        }
        DialogPosition::Below => {
            let base = (cursor_line + 1) * LINE_HEIGHT + BUFFER_BAR_HEIGHT;
            base.min(MAX_TOP)
        }
    };

    let left = (cursor_col * CHAR_WIDTH + GUTTER_WIDTH).min(MAX_LEFT);

    (top, left)
}

/// Base container for inline dialogs.
///
/// Provides consistent positioning and styling for cursor-positioned popups.
/// Use this as a building block for specific dialog types.
///
/// # Example
///
/// ```rust,ignore
/// InlineDialogContainer {
///     cursor_line: 10,
///     cursor_col: 5,
///     position: DialogPosition::Below,
///     class: "my-custom-dialog",
///     div { "Dialog content here" }
/// }
/// ```
#[component]
pub fn InlineDialogContainer(
    /// Line number where the cursor is (0-indexed).
    cursor_line: usize,
    /// Column number where the cursor is (0-indexed).
    cursor_col: usize,
    /// Position relative to cursor (above or below).
    #[props(default)]
    position: DialogPosition,
    /// Optional CSS class to add to the container.
    #[props(default)]
    class: Option<String>,
    /// Size constraints for the dialog.
    #[props(default)]
    constraints: DialogConstraints,
    /// Child elements to render inside the dialog.
    children: Element,
) -> Element {
    let (top, left) = calculate_position(cursor_line, cursor_col, position);

    let position_style = format!("top: {top}px; left: {left}px;");
    let constraint_style = constraints.to_style();

    let combined_style = if constraint_style.is_empty() {
        position_style
    } else {
        format!("{position_style} {constraint_style}")
    };

    let css_class = class.unwrap_or_default();

    rsx! {
        div {
            class: "inline-dialog {css_class}",
            style: "{combined_style}",
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_position_below() {
        let (top, left) = calculate_position(5, 10, DialogPosition::Below);
        // (5 + 1) * 21 + 40 = 166
        assert_eq!(top, 166);
        // 10 * 8 + 60 = 140
        assert_eq!(left, 140);
    }

    #[test]
    fn test_calculate_position_above() {
        let (top, left) = calculate_position(5, 10, DialogPosition::Above);
        // (5 - 1) * 21 + 40 = 124
        assert_eq!(top, 124);
        assert_eq!(left, 140);
    }

    #[test]
    fn test_calculate_position_capped() {
        // Test max caps
        let (top, left) = calculate_position(30, 100, DialogPosition::Below);
        assert_eq!(top, MAX_TOP);
        assert_eq!(left, MAX_LEFT);
    }

    #[test]
    fn test_constraints_to_style() {
        let constraints = DialogConstraints {
            min_width: Some(200),
            max_width: Some(500),
            max_height: Some(300),
        };
        let style = constraints.to_style();
        assert!(style.contains("min-width: 200px"));
        assert!(style.contains("max-width: 500px"));
        assert!(style.contains("max-height: 300px"));
    }

    #[test]
    fn test_constraints_no_limits() {
        let constraints = DialogConstraints {
            min_width: None,
            max_width: None,
            max_height: None,
        };
        assert!(constraints.to_style().is_empty());
    }
}
