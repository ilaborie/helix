//! Code actions menu component.
//!
//! Displays available code actions (quick fixes, refactors) at cursor position,
//! with an optional preview panel showing the diff for the selected action.

use crate::icons::{lucide, Icon};
use dioxus::prelude::*;

use super::code_action_preview::CodeActionPreviewPanel;
use crate::components::inline_dialog::{DialogConstraints, InlineDialogContainer, InlineListItem};
use crate::lsp::{CodeActionPreviewState, CodeActionSnapshot};

/// Get the icon and color for a code action kind.
fn action_kind_style(kind: Option<&str>, is_preferred: bool) -> (&'static str, Element) {
    if is_preferred {
        return (
            "var(--warning)",
            rsx! { Icon { data: lucide::Star, size: "12", fill: "currentColor" } },
        );
    }

    match kind {
        Some(k) if k.starts_with("quickfix") => (
            "var(--success)",
            rsx! { Icon { data: lucide::Wrench, size: "12", fill: "currentColor" } },
        ),
        Some(k) if k.starts_with("refactor.extract") => (
            "var(--accent)",
            rsx! { Icon { data: lucide::PackagePlus, size: "12", fill: "currentColor" } },
        ),
        Some(k) if k.starts_with("refactor") => (
            "var(--purple)",
            rsx! { Icon { data: lucide::FileCode, size: "12", fill: "currentColor" } },
        ),
        Some(k) if k.starts_with("source") => (
            "var(--hint)",
            rsx! { Icon { data: lucide::FileCode, size: "12", fill: "currentColor" } },
        ),
        _ => (
            "var(--text)",
            rsx! { Icon { data: lucide::Lightbulb, size: "12", fill: "currentColor" } },
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
    let is_quickfix = action.kind.as_deref().is_some_and(|k| k.starts_with("quickfix"));

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

/// Search bar with filter input and count.
#[component]
fn SearchBar(filter: String, filtered_count: usize, total_count: usize) -> Element {
    rsx! {
        div {
            class: "code-actions-search",
            span {
                class: "icon-wrapper",
                style: "color: var(--text-dim);",
                Icon { data: lucide::Search, size: "14", fill: "currentColor" }
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
    }
}

/// Whether the preview is displayable (not all unavailable).
fn has_displayable_preview(preview: Option<&CodeActionPreviewState>) -> bool {
    matches!(
        preview,
        Some(CodeActionPreviewState::Loading | CodeActionPreviewState::Available(_))
    )
}

/// Code actions menu that displays available fixes and refactors.
#[component]
pub fn CodeActionsMenu(
    actions: Vec<CodeActionSnapshot>,
    selected: usize,
    cursor_line: usize,
    cursor_col: usize,
    filter: String,
    #[props(default)] preview: Option<CodeActionPreviewState>,
) -> Element {
    // Filter actions by title (case-insensitive substring match)
    let filter_lower = filter.to_lowercase();
    let filtered_actions: Vec<_> = actions
        .iter()
        .filter(|action| filter.is_empty() || action.title.to_lowercase().contains(&filter_lower))
        .cloned()
        .collect();

    let total_count = actions.len();
    let filtered_count = filtered_actions.len();
    let show_preview = has_displayable_preview(preview.as_ref());

    let constraints = if show_preview {
        DialogConstraints {
            min_width: Some(600),
            max_width: Some(800),
            max_height: Some(400),
        }
    } else {
        DialogConstraints {
            min_width: Some(220),
            max_width: Some(450),
            max_height: Some(300),
        }
    };

    rsx! {
        InlineDialogContainer {
            cursor_line,
            cursor_col,
            class: "code-actions-menu",
            constraints,

            if show_preview {
                // Two-column layout
                div {
                    class: "code-actions-layout",

                    // Left column: action list
                    div {
                        class: "code-actions-list-column",

                        SearchBar {
                            filter: filter.clone(),
                            filtered_count,
                            total_count,
                        }

                        div {
                            class: "inline-dialog-items",
                            if filtered_actions.is_empty() {
                                div {
                                    class: "inline-dialog-empty",
                                    if filter.is_empty() {
                                        "No code actions available"
                                    } else {
                                        "No matching code actions"
                                    }
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

                    // Right column: preview
                    div {
                        class: "code-actions-preview-column",
                        if let Some(ref preview_state) = preview {
                            CodeActionPreviewPanel { preview: preview_state.clone() }
                        }
                    }
                }
            } else {
                // Single-column layout (original)
                div {
                    class: "inline-dialog-list",

                    SearchBar {
                        filter: filter.clone(),
                        filtered_count,
                        total_count,
                    }

                    if filtered_actions.is_empty() {
                        div {
                            class: "inline-dialog-empty",
                            if filter.is_empty() {
                                "No code actions available"
                            } else {
                                "No matching code actions"
                            }
                        }
                    } else {
                        div {
                            class: "inline-dialog-items",
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
            }
        }
    }
}
