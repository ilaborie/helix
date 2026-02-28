//! Git diff hover popup component.
//!
//! Shows diff content when hovering over VCS gutter markers, with
//! "Revert Change" and "Copy Original" action buttons.

use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

/// Generation counter for the git diff close grace-period timer.
///
/// Incremented on every "cancel" event (mouseenter on gutter or popup).
/// The scheduled close task compares the generation it captured at spawn
/// time against the current value — if they differ, the close was cancelled.
pub(crate) static GIT_DIFF_CLOSE_GEN: AtomicU64 = AtomicU64::new(0);

use crate::hooks::use_snapshot_signal;
use crate::icons::{lucide, Icon};
use crate::lsp::DiffChangeKind;
use crate::state::{EditorCommand, GitDiffHunkSnapshot};
use crate::AppState;

/// Git diff hover popup component.
#[component]
pub fn GitDiffPopup(hunk: GitDiffHunkSnapshot, line: usize) -> Element {
    let app_state = use_context::<AppState>();
    let mut snapshot_signal = use_snapshot_signal();

    let revert_handler = {
        let app_state = app_state.clone();
        move |_: MouseEvent| {
            app_state.send_command(EditorCommand::RevertGitHunk(line));
            app_state.process_and_notify(&mut snapshot_signal);
        }
    };

    let copy_handler = {
        let app_state = app_state.clone();
        move |_: MouseEvent| {
            app_state.send_command(EditorCommand::CopyOriginalHunk(line));
            app_state.process_and_notify(&mut snapshot_signal);
        }
    };

    let close_handler = {
        let app_state = app_state.clone();
        move |_: MouseEvent| {
            app_state.send_command(EditorCommand::CloseGitDiffHover);
            app_state.process_and_notify(&mut snapshot_signal);
        }
    };

    let has_original = hunk.lines_removed > 0;
    let stat_text = format!("+{} -{}", hunk.lines_added, hunk.lines_removed);

    // Position next to the gutter using JS after mount
    use_effect(move || {
        let _ = line;
        document::eval(&format!(
            "if (typeof positionGitDiffPopup === 'function') positionGitDiffPopup({line});"
        ));
    });

    rsx! {
        div {
            id: "git-diff-popup",
            class: "git-diff-popup",
            onmouseenter: move |_| {
                // Cancel any pending grace-period close scheduled by the gutter zone.
                GIT_DIFF_CLOSE_GEN.fetch_add(1, Ordering::Relaxed);
            },
            onmouseleave: {
                let mut close = close_handler.clone();
                move |evt: MouseEvent| close(evt)
            },

            // Header with stats and action buttons
            div {
                class: "git-diff-popup-header",

                span {
                    class: "git-diff-popup-stats",
                    span {
                        class: "git-diff-popup-stat-label",
                        style: "color: {hunk.diff_type.css_color()};",
                        match hunk.diff_type {
                            crate::state::DiffLineType::Added => "Added",
                            crate::state::DiffLineType::Modified => "Modified",
                            crate::state::DiffLineType::Deleted => "Deleted",
                        }
                    }
                    span {
                        class: "git-diff-popup-stat-count",
                        "{stat_text}"
                    }
                }

                div {
                    class: "git-diff-popup-actions",

                    if has_original {
                        button {
                            class: "git-diff-popup-btn",
                            title: "Copy Original",
                            onclick: copy_handler,
                            Icon { data: lucide::FileDiff, size: "12" }
                            "Copy"
                        }
                    }

                    button {
                        class: "git-diff-popup-btn git-diff-popup-btn-revert",
                        title: "Revert Change",
                        onclick: revert_handler,
                        Icon { data: lucide::RefreshCw, size: "12" }
                        "Revert"
                    }

                    button {
                        class: "git-diff-popup-btn-close",
                        title: "Close",
                        onclick: close_handler,
                        Icon { data: lucide::X, size: "12" }
                    }
                }
            }

            // Diff content
            div {
                class: "git-diff-popup-content",

                for (li, diff_line) in hunk.lines.iter().enumerate() {
                    div {
                        key: "gdl-{li}",
                        class: match diff_line.kind {
                            DiffChangeKind::Added => "code-action-diff-line code-action-diff-added",
                            DiffChangeKind::Removed => "code-action-diff-line code-action-diff-removed",
                            DiffChangeKind::Context => "code-action-diff-line code-action-diff-context",
                        },

                        span {
                            class: "code-action-diff-gutter",
                            if let Some(n) = diff_line.old_line_number {
                                "{n}"
                            }
                        }
                        span {
                            class: "code-action-diff-gutter",
                            if let Some(n) = diff_line.new_line_number {
                                "{n}"
                            }
                        }

                        span {
                            class: "code-action-diff-sign",
                            match diff_line.kind {
                                DiffChangeKind::Added => "+",
                                DiffChangeKind::Removed => "-",
                                DiffChangeKind::Context => " ",
                            }
                        }

                        span {
                            class: "code-action-diff-text",
                            "{diff_line.content}"
                        }
                    }
                }
            }
        }
    }
}
