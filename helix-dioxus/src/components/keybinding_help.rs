//! Keybinding help bar component.
//!
//! Displays context-aware keyboard shortcut hints above the statusline,
//! with register indicators on the right.

use dioxus::prelude::*;

use crate::state::{EditorCommand, PendingKeySequence, RegisterSnapshot};
use crate::AppState;

/// Maximum characters to show in the register dialog.
const REGISTER_DIALOG_LEN: usize = 2000;

/// Returns shortcut hints (key, description) for the given context.
fn hints_for_context(mode: &str, pending: PendingKeySequence) -> Vec<(&'static str, &'static str)> {
    match pending {
        PendingKeySequence::GPrefix => vec![
            ("g", "top"),
            ("e", "end"),
            ("d", "definition"),
            ("r", "references"),
            ("y", "type def"),
            ("i", "impl"),
        ],
        PendingKeySequence::SpaceLeader => vec![
            ("/", "search"),
            ("a", "actions"),
            ("c", "comment"),
            ("d", "diag"),
            ("f", "format"),
            ("r", "rename"),
            ("s", "symbols"),
        ],
        PendingKeySequence::BracketNext => vec![("d", "next diag")],
        PendingKeySequence::BracketPrev => vec![("d", "prev diag")],
        PendingKeySequence::FindForward
        | PendingKeySequence::FindBackward
        | PendingKeySequence::TillForward
        | PendingKeySequence::TillBackward => {
            vec![]
        }
        PendingKeySequence::None => match mode {
            "INSERT" => vec![
                ("Esc", "exit"),
                ("C-w", "del word"),
                ("C-u", "del to start"),
                ("C-Space", "complete"),
                ("C-.", "actions"),
            ],
            "SELECT" => vec![
                ("Esc", "exit"),
                ("d", "delete"),
                ("y", "yank"),
                ("p", "replace"),
                ("x", "line"),
            ],
            _ => vec![
                ("i", "insert"),
                ("v", "select"),
                ("x", "line"),
                ("d", "delete"),
                ("y", "yank"),
                ("p", "paste"),
                ("/", "search"),
                (":", "command"),
                ("g..", "goto"),
                ("Spc..", "leader"),
            ],
        },
    }
}

/// Returns a prefix label for pending key sequences, if any.
fn pending_prefix(pending: PendingKeySequence) -> Option<&'static str> {
    match pending {
        PendingKeySequence::GPrefix => Some("g"),
        PendingKeySequence::SpaceLeader => Some("Space"),
        PendingKeySequence::BracketNext => Some("]"),
        PendingKeySequence::BracketPrev => Some("["),
        PendingKeySequence::FindForward => Some("f"),
        PendingKeySequence::FindBackward => Some("F"),
        PendingKeySequence::TillForward => Some("t"),
        PendingKeySequence::TillBackward => Some("T"),
        PendingKeySequence::None => None,
    }
}

/// Whether the pending sequence is a find/till character prompt.
fn is_char_prompt(pending: PendingKeySequence) -> bool {
    matches!(
        pending,
        PendingKeySequence::FindForward
            | PendingKeySequence::FindBackward
            | PendingKeySequence::TillForward
            | PendingKeySequence::TillBackward
    )
}

/// Truncate a string to `max_len` chars, appending "..." if truncated.
fn truncate(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len).collect();
        format!("{truncated}...")
    }
}

/// Human-readable label for a register character.
fn register_label(name: char) -> &'static str {
    match name {
        '+' => "clipboard",
        '*' => "selection",
        '/' => "search",
        '"' => "default",
        _ => "register",
    }
}

/// Whether a register supports clearing.
fn can_clear(name: char) -> bool {
    matches!(name, '+' | '/')
}

#[component]
pub fn KeybindingHelpBar(
    mode: String,
    pending: PendingKeySequence,
    register_snapshots: Vec<RegisterSnapshot>,
) -> Element {
    let hints = hints_for_context(&mode, pending);
    let prefix = pending_prefix(pending);
    let char_prompt = is_char_prompt(pending);

    // Track which register dialog is open (None = closed)
    let mut open_register = use_signal(|| None::<usize>);

    rsx! {
        div { class: "keybinding-help-bar",
            // Show prefix label for pending sequences
            if let Some(pfx) = prefix {
                span { class: "keybinding-help-prefix", "{pfx}" }
                span { class: "keybinding-help-separator" }
            }

            // Show character prompt for f/F/t/T
            if char_prompt {
                span { class: "keybinding-help-desc", "type a character..." }
            }

            // Show hint items with separators
            for (idx, (key, desc)) in hints.iter().enumerate() {
                if idx > 0 {
                    span { class: "keybinding-help-separator" }
                }
                span { class: "keybinding-help-item",
                    span { class: "keybinding-help-key", "{key}" }
                    span { class: "keybinding-help-desc", "{desc}" }
                }
            }

            // Spacer to push register indicators to the right
            div { style: "flex: 1;" }

            // Register indicators
            div { class: "keybinding-help-registers",
                for (idx, reg) in register_snapshots.iter().enumerate() {
                    {
                        let has_content = !reg.content.is_empty();
                        let name = reg.name;

                        rsx! {
                            div {
                                class: "keybinding-help-register",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    if open_register() == Some(idx) {
                                        open_register.set(None);
                                    } else {
                                        open_register.set(Some(idx));
                                    }
                                },

                                span {
                                    class: if has_content {
                                        "keybinding-help-register-label keybinding-help-register-active"
                                    } else {
                                        "keybinding-help-register-label"
                                    },
                                    "{name}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Register dialog (rendered outside the help bar to avoid clipping)
        if let Some(idx) = open_register() {
            if let Some(reg) = register_snapshots.get(idx) {
                RegisterDialog {
                    name: reg.name,
                    content: reg.content.clone(),
                    on_close: move |_| open_register.set(None),
                }
            }
        }
    }
}

#[component]
fn RegisterDialog(name: char, content: String, on_close: EventHandler) -> Element {
    let app_state = use_context::<AppState>();
    let label = register_label(name);
    let has_content = !content.is_empty();
    let clearable = can_clear(name) && has_content;
    let display_content = if has_content {
        truncate(&content, REGISTER_DIALOG_LEN)
    } else {
        "(empty)".to_string()
    };

    rsx! {
        // Backdrop to close on click outside
        div {
            class: "register-dialog-backdrop",
            onclick: move |_| on_close.call(()),
        }

        div {
            class: "register-dialog",
            onclick: move |evt| evt.stop_propagation(),

            // Header
            div { class: "register-dialog-header",
                div { class: "register-dialog-title",
                    span { class: "register-dialog-name", "{name}" }
                    " {label}"
                }
                div {
                    class: "register-dialog-close",
                    onclick: move |_| on_close.call(()),
                    "×"
                }
            }

            // Content
            div { class: "register-dialog-body",
                pre {
                    class: if has_content {
                        "register-dialog-content"
                    } else {
                        "register-dialog-content register-dialog-empty"
                    },
                    "{display_content}"
                }
            }

            // Footer with clear button
            if clearable {
                div { class: "register-dialog-footer",
                    button {
                        class: "register-dialog-clear-btn",
                        onclick: move |_| {
                            app_state.send_command(EditorCommand::ClearRegister(name));
                            app_state.process_commands_sync();
                            on_close.call(());
                        },
                        "Clear"
                    }
                }
            }
        }
    }
}
