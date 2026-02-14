//! Confirmation dialog component.
//!
//! A modal dialog for confirming destructive actions like quitting with unsaved changes.

use dioxus::prelude::*;

use crate::state::{ConfirmationDialogSnapshot, EditorCommand};
use crate::AppState;

/// Confirmation dialog component.
#[component]
pub fn ConfirmationDialog(
    dialog: ConfirmationDialogSnapshot,
    on_change: EventHandler<()>,
) -> Element {
    let app_state = use_context::<AppState>();

    let confirm_handler = {
        let app_state = app_state.clone();
        move |_| {
            app_state.send_command(EditorCommand::ConfirmationDialogConfirm);
            app_state.process_commands_sync();
            on_change.call(());
        }
    };

    let deny_handler = {
        let app_state = app_state.clone();
        move |_| {
            app_state.send_command(EditorCommand::ConfirmationDialogDeny);
            app_state.process_commands_sync();
            on_change.call(());
        }
    };

    let cancel_handler = {
        let app_state = app_state.clone();
        move |_| {
            app_state.send_command(EditorCommand::ConfirmationDialogCancel);
            app_state.process_commands_sync();
            on_change.call(());
        }
    };

    rsx! {
        // Overlay
        div {
            class: "confirmation-dialog-overlay",
            onmousedown: {
                let cancel = cancel_handler.clone();
                move |evt| cancel(evt)
            },

            // Dialog container
            div {
                class: "confirmation-dialog",
                onmousedown: move |evt| evt.stop_propagation(),

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
                            let cancel = cancel_handler.clone();
                            move |evt| {
                                evt.stop_propagation();
                                cancel(evt);
                            }
                        },
                        "{dialog.cancel_label}"
                        kbd { "Esc" }
                    }

                    // Deny button (optional - only shown if deny_label is set)
                    if let Some(ref deny_label) = dialog.deny_label {
                        button {
                            class: "confirmation-btn confirmation-btn-danger",
                            onmousedown: {
                                let deny = deny_handler.clone();
                                move |evt| {
                                    evt.stop_propagation();
                                    deny(evt);
                                }
                            },
                            "{deny_label}"
                            kbd { "n" }
                        }
                    }

                    // Confirm button (primary action)
                    button {
                        class: "confirmation-btn confirmation-btn-primary",
                        onmousedown: {
                            let confirm = confirm_handler.clone();
                            move |evt| {
                                evt.stop_propagation();
                                confirm(evt);
                            }
                        },
                        "{dialog.confirm_label}"
                        kbd { "y" }
                    }
                }
            }
        }
    }
}
