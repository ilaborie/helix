//! Code action preview panel component.
//!
//! Displays a diff preview of what the selected code action will change.

use dioxus::prelude::*;

use crate::lsp::{CodeActionPreview, CodeActionPreviewState, DiffChangeKind};

/// Preview panel showing the diff for a code action.
#[component]
pub fn CodeActionPreviewPanel(preview: CodeActionPreviewState) -> Element {
    match preview {
        CodeActionPreviewState::Loading => {
            rsx! {
                div {
                    class: "code-action-preview code-action-preview-loading",
                    "Resolving..."
                }
            }
        }
        CodeActionPreviewState::Unavailable => {
            rsx! {
                div {
                    class: "code-action-preview code-action-preview-unavailable",
                    "No preview available"
                }
            }
        }
        CodeActionPreviewState::Available(preview) => {
            rsx! { PreviewContent { preview } }
        }
    }
}

/// Render the actual diff content.
#[component]
fn PreviewContent(preview: CodeActionPreview) -> Element {
    rsx! {
        div {
            class: "code-action-preview",

            // Stats header
            div {
                class: "code-action-preview-header",
                if preview.lines_added > 0 {
                    span {
                        class: "code-action-diff-stat-added",
                        "+{preview.lines_added}"
                    }
                }
                if preview.lines_removed > 0 {
                    span {
                        class: "code-action-diff-stat-removed",
                        "-{preview.lines_removed}"
                    }
                }
            }

            // File diffs
            for file_diff in &preview.file_diffs {
                div {
                    class: "code-action-preview-file",

                    // File header
                    div {
                        class: "code-action-preview-file-header",
                        "{file_diff.file_path}"
                    }

                    // Hunks
                    for (hi, hunk) in file_diff.hunks.iter().enumerate() {
                        if hi > 0 {
                            div {
                                class: "code-action-diff-separator",
                                "···"
                            }
                        }
                        for (li, line) in hunk.lines.iter().enumerate() {
                            div {
                                key: "{hi}-{li}",
                                class: match line.kind {
                                    DiffChangeKind::Added => "code-action-diff-line code-action-diff-added",
                                    DiffChangeKind::Removed => "code-action-diff-line code-action-diff-removed",
                                    DiffChangeKind::Context => "code-action-diff-line code-action-diff-context",
                                },

                                // Gutter with line number
                                span {
                                    class: "code-action-diff-gutter",
                                    if let Some(n) = line.old_line_number {
                                        "{n}"
                                    }
                                }
                                span {
                                    class: "code-action-diff-gutter",
                                    if let Some(n) = line.new_line_number {
                                        "{n}"
                                    }
                                }

                                // Sign
                                span {
                                    class: "code-action-diff-sign",
                                    match line.kind {
                                        DiffChangeKind::Added => "+",
                                        DiffChangeKind::Removed => "-",
                                        DiffChangeKind::Context => " ",
                                    }
                                }

                                // Content
                                span {
                                    class: "code-action-diff-text",
                                    "{line.content}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
