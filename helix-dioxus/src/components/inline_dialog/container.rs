//! Base container for inline dialogs.
//!
//! Handles positioning and common styling for cursor-positioned popups.
//! Positioning is done in JavaScript via `positionInlineDialogs()` — the
//! dialog is rendered with `visibility: hidden` and a `data-position`
//! attribute, then a `use_effect` calls the JS function which reads the
//! cursor's `getBoundingClientRect()` and sets `top`/`left`/`visibility`.

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
    fn to_style(self) -> String {
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

/// Base container for inline dialogs.
///
/// Renders with `visibility: hidden` and calls `positionInlineDialogs()` via
/// `use_effect` so that JavaScript positions the dialog using the cursor's
/// `getBoundingClientRect()`. No pixel coordinates pass through Rust.
///
/// # Example
///
/// ```rust,ignore
/// InlineDialogContainer {
///     position: DialogPosition::Below,
///     class: "my-custom-dialog",
///     div { "Dialog content here" }
/// }
/// ```
#[component]
pub fn InlineDialogContainer(
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
    // After this component mounts/updates, call JS to position it
    use_effect(|| {
        document::eval("positionInlineDialogs()");
    });

    let constraint_style = constraints.to_style();
    let hidden_style = if constraint_style.is_empty() {
        "visibility: hidden".to_string()
    } else {
        format!("visibility: hidden; {constraint_style}")
    };

    let data_position = match position {
        DialogPosition::Above => "above",
        DialogPosition::Below => "below",
    };

    let css_class = class.unwrap_or_default();

    rsx! {
        div {
            class: "inline-dialog {css_class}",
            style: "{hidden_style}",
            "data-position": "{data_position}",
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
