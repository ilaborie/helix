//! Input dialog component for prompting user input.
//!
//! Used for operations like rename symbol, goto line, etc.

use dioxus::prelude::*;

use crate::components::KbdKey;
use crate::state::InputDialogSnapshot;

use crate::components::inline_dialog::{DialogConstraints, DialogPosition, InlineDialogContainer};

/// An inline input dialog positioned at the cursor.
#[component]
pub fn InputDialog(dialog: InputDialogSnapshot, cursor_line: usize, cursor_col: usize) -> Element {
    let placeholder = dialog.placeholder.as_deref().unwrap_or("");
    let show_placeholder = dialog.value.is_empty() && !placeholder.is_empty();

    rsx! {
        InlineDialogContainer {
            cursor_line,
            cursor_col,
            position: DialogPosition::Below,
            class: "input-dialog",
            constraints: DialogConstraints {
                min_width: Some(300),
                max_width: Some(400),
                max_height: None,
            },

            div {
                class: "input-dialog-title",
                "{dialog.title}"
            }

            div {
                class: "input-dialog-prompt",
                "{dialog.prompt}"
            }

            div {
                class: "input-dialog-input-container",

                if show_placeholder {
                    // When showing placeholder, cursor comes first (at position 0)
                    span {
                        class: "input-dialog-cursor",
                    }
                    span {
                        class: "input-dialog-placeholder",
                        "{placeholder}"
                    }
                } else {
                    // When showing value, cursor comes after the text
                    span {
                        class: "input-dialog-value",
                        "{dialog.value}"
                    }
                    span {
                        class: "input-dialog-cursor",
                    }
                }
            }

            div {
                class: "input-dialog-help",

                span {
                    KbdKey { label: "Enter" }
                    " Confirm"
                }

                span {
                    KbdKey { label: "Esc" }
                    " Cancel"
                }
            }
        }
    }
}
