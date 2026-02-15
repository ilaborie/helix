//! List-based inline dialog component.
//!
//! Provides a scrollable list dialog with selection support,
//! suitable for completion menus, code actions, etc.

use dioxus::prelude::*;

use super::container::{DialogConstraints, DialogPosition, InlineDialogContainer};

/// A single item in an inline list dialog.
///
/// Wraps content with selection styling and keyboard navigation support.
#[component]
pub fn InlineListItem(
    /// Whether this item is currently selected.
    is_selected: bool,
    /// Optional additional CSS class.
    #[props(default)]
    class: Option<String>,
    /// Child elements to render inside the item.
    children: Element,
) -> Element {
    let base_class = "inline-dialog-item";
    let selected_class = if is_selected { "inline-dialog-item-selected" } else { "" };
    let custom_class = class.unwrap_or_default();

    rsx! {
        div {
            class: "{base_class} {selected_class} {custom_class}",
            {children}
        }
    }
}

/// List-based inline dialog with selection support.
///
/// Renders a positioned popup containing a scrollable list of items.
/// Handles empty state and provides consistent styling.
///
/// # Example
///
/// ```rust,ignore
/// InlineListDialog {
///     cursor_line: 10,
///     cursor_col: 5,
///     selected: current_selection,
///     empty_message: "No completions available",
///
///     for (idx, item) in items.iter().enumerate() {
///         InlineListItem {
///             key: "{idx}",
///             is_selected: idx == current_selection,
///             span { "{item.label}" }
///         }
///     }
/// }
/// ```
#[component]
pub fn InlineListDialog(
    /// Line number where the cursor is (0-indexed).
    cursor_line: usize,
    /// Column number where the cursor is (0-indexed).
    cursor_col: usize,
    /// Index of the currently selected item.
    selected: usize,
    /// Message to display when the list is empty.
    #[props(default = "No items".to_string())]
    empty_message: String,
    /// Position relative to cursor (above or below).
    #[props(default)]
    position: DialogPosition,
    /// Optional CSS class for the container.
    #[props(default)]
    class: Option<String>,
    /// Size constraints for the dialog.
    #[props(default)]
    constraints: DialogConstraints,
    /// Whether the list has items (used for empty state).
    #[props(default = true)]
    has_items: bool,
    /// Child elements (should be `InlineListItem` components).
    children: Element,
) -> Element {
    // Mark selected as used (it's provided for context, actual selection
    // styling is handled by InlineListItem)
    let _ = selected;

    let combined_class = match class {
        Some(c) => format!("inline-dialog-list {c}"),
        None => "inline-dialog-list".to_string(),
    };

    rsx! {
        InlineDialogContainer {
            cursor_line,
            cursor_col,
            position,
            class: combined_class,
            constraints,

            if has_items {
                div {
                    class: "inline-dialog-items",
                    {children}
                }
            } else {
                div {
                    class: "inline-dialog-empty",
                    "{empty_message}"
                }
            }
        }
    }
}
