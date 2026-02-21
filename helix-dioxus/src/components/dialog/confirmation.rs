//! Confirmation dialog component.
//!
//! A modal dialog for confirming destructive actions like quitting with unsaved changes.

use dioxus::prelude::*;

use crate::components::{KbdKey, ModalOverlay};
use crate::hooks::use_snapshot_signal;
use crate::state::{ConfirmationDialogSnapshot, EditorCommand};
use crate::AppState;

/// Confirmation dialog component.
#[component]
pub fn ConfirmationDialog(dialog: ConfirmationDialogSnapshot) -> Element {
    let app_state = use_context::<AppState>();
    let mut snapshot_signal = use_snapshot_signal();

    let confirm_handler = {
        let app_state = app_state.clone();
        move |_| {
            app_state.send_command(EditorCommand::ConfirmationDialogConfirm);
            app_state.process_and_notify(&mut snapshot_signal);
        }
    };

    let deny_handler = {
        let app_state = app_state.clone();
        move |_| {
            app_state.send_command(EditorCommand::ConfirmationDialogDeny);
            app_state.process_and_notify(&mut snapshot_signal);
        }
    };

    let cancel_handler = {
        let app_state = app_state.clone();
        move |_| {
            app_state.send_command(EditorCommand::ConfirmationDialogCancel);
            app_state.process_and_notify(&mut snapshot_signal);
        }
    };

    rsx! {
        ModalOverlay {
            class: "confirmation-dialog",
            z_index: "--z-confirmation",
            on_backdrop_click: {
                let mut cancel = cancel_handler.clone();
                move |evt| cancel(evt)
            },

            // Title
            div {
                class: "confirmation-dialog-title",
                "{dialog.title}"
            }

            // Message
            div {
                class: "confirmation-dialog-message",
                "{dialog.message}"
            }

            // Buttons
            div {
                class: "confirmation-dialog-buttons",

                // Cancel button (always present)
                button {
                    class: "confirmation-btn confirmation-btn-secondary",
                    onmousedown: {
                        let mut cancel = cancel_handler.clone();
                        move |evt| {
                            evt.stop_propagation();
                            cancel(evt);
                        }
                    },
                    KbdKey { label: "Esc" }
                    "{dialog.cancel_label}"
                }

                // Deny button (optional - only shown if deny_label is set)
                if let Some(ref deny_label) = dialog.deny_label {
                    button {
                        class: "confirmation-btn confirmation-btn-danger",
                        onmousedown: {
                            let mut deny = deny_handler.clone();
                            move |evt| {
                                evt.stop_propagation();
                                deny(evt);
                            }
                        },
                        KbdKey { label: "n" }
                        "{deny_label}"
                    }
                }

                // Confirm button (primary action)
                button {
                    class: "confirmation-btn confirmation-btn-primary",
                    onmousedown: {
                        let mut confirm = confirm_handler.clone();
                        move |evt| {
                            evt.stop_propagation();
                            confirm(evt);
                        }
                    },
                    KbdKey { label: "y" }
                    "{dialog.confirm_label}"
                }
            }
        }
    }
}
