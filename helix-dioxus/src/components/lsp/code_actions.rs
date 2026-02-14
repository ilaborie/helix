//! Code actions menu component.
//!
//! Displays available code actions (quick fixes, refactors) at cursor position.

use dioxus::prelude::*;
use lucide_dioxus::{FileCode, Lightbulb, PackagePlus, Search, Star, Wrench};

use crate::components::inline_dialog::{DialogConstraints, InlineListDialog, InlineListItem};
use crate::lsp::CodeActionSnapshot;

/// Get the icon and color for a code action kind.
fn action_kind_style(kind: Option<&str>, is_preferred: bool) -> (&'static str, Element) {
    if is_preferred {
        return (
            "var(--warning)",
            rsx! { Star { size: 12, color: "currentColor" } },
        );
    }

    match kind {
        Some(k) if k.starts_with("quickfix") => (
            "var(--success)",
            rsx! { Wrench { size: 12, color: "currentColor" } },
        ),
        Some(k) if k.starts_with("refactor.extract") => (
            "var(--accent)",
            rsx! { PackagePlus { size: 12, color: "currentColor" } },
        ),
        Some(k) if k.starts_with("refactor") => (
            "var(--purple)",
            rsx! { FileCode { size: 12, color: "currentColor" } },
        ),
        Some(k) if k.starts_with("source") => (
            "var(--hint)",
            rsx! { FileCode { size: 12, color: "currentColor" } },
        ),
        _ => (
            "var(--text)",
            rsx! { Lightbulb { size: 12, color: "currentColor" } },
        ),
    }
}

/// Get a short display name for a code action kind.
fn kind_short_name(kind: &str) -> &'static str {
    match kind {
        k if k.starts_with("quickfix") => "fix",
        k if k.starts_with("refactor.extract") => "extract",
        k if k.starts_with("refactor.inline") => "inline",
        k if k.starts_with("refactor.rewrite") => "rewrite",
        k if k.starts_with("refactor") => "refactor",
        k if k.starts_with("source.organizeImports") => "imports",
        k if k.starts_with("source") => "source",
        _ => "",
    }
}

/// A single code action item in the menu.
#[component]
fn CodeActionItem(action: CodeActionSnapshot, is_selected: bool) -> Element {
    let is_disabled = action.disabled.is_some();

    // Get icon and color based on action kind
    let (kind_color, icon) = action_kind_style(action.kind.as_deref(), action.is_preferred);

    // Check if this is a quickfix action
    let is_quickfix = action
        .kind
        .as_deref()
        .is_some_and(|k| k.starts_with("quickfix"));

    let mut item_class = String::new();
    if is_disabled {
        item_class.push_str("code-action-disabled");
    }
    if is_quickfix {
        if !item_class.is_empty() {
            item_class.push(' ');
        }
        item_class.push_str("code-action-quickfix");
    }

    rsx! {
        InlineListItem {
            is_selected,
            class: if item_class.is_empty() { None } else { Some(item_class) },

            // Action type icon (colored)
            span {
                class: "code-action-icon icon-wrapper",
                style: "color: {kind_color};",
                {icon}
            }

            // Action title (normal text color)
            span {
                class: "code-action-title",
                "{action.title}"
            }

            // Kind indicator (colored to match icon)
            if let Some(ref kind) = action.kind {
                span {
                    class: "code-action-kind",
                    style: "color: {kind_color}; border-color: {kind_color};",
                    "{kind_short_name(kind)}"
                }
            }
        }
    }
}

/// Code actions menu that displays available fixes and refactors.
#[component]
pub fn CodeActionsMenu(
    actions: Vec<CodeActionSnapshot>,
    selected: usize,
    cursor_line: usize,
    cursor_col: usize,
    filter: String,
) -> Element {
    // Filter actions by title (case-insensitive substring match)
    let filter_lower = filter.to_lowercase();
    let filtered_actions: Vec<_> = actions
        .iter()
        .filter(|a| filter.is_empty() || a.title.to_lowercase().contains(&filter_lower))
        .cloned()
        .collect();

    let total_count = actions.len();
    let filtered_count = filtered_actions.len();

    // Determine empty message based on filter state
    let empty_message = if filter.is_empty() {
        "No code actions available"
    } else {
        "No matching code actions"
    };

    let constraints = DialogConstraints {
        min_width: Some(220),
        max_width: Some(450),
        max_height: Some(300),
    };

    rsx! {
        InlineListDialog {
            cursor_line,
            cursor_col,
            selected,
            empty_message,
            class: "code-actions-menu",
            constraints,
            has_items: !filtered_actions.is_empty(),

            // Search input at the top
            div {
                class: "code-actions-search",
                span {
                    class: "icon-wrapper",
                    style: "color: var(--text-dim);",
                    Search { size: 14, color: "currentColor" }
                }
                span {
                    class: "code-actions-search-input",
                    if filter.is_empty() {
                        span { class: "code-actions-search-placeholder", "Type to filter..." }
                    } else {
                        "{filter}"
                    }
                }
                span {
                    class: "code-actions-count",
                    "{filtered_count}/{total_count}"
                }
            }

            for (idx, action) in filtered_actions.iter().enumerate() {
                CodeActionItem {
                    key: "{idx}",
                    action: action.clone(),
                    is_selected: idx == selected,
                }
            }
        }
    }
}
