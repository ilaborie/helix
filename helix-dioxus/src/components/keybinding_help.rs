//! Keybinding help bar component.
//!
//! Displays context-aware keyboard shortcut hints above the statusline.

use dioxus::prelude::*;

use crate::state::PendingKeySequence;

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

#[component]
pub fn KeybindingHelpBar(mode: String, pending: PendingKeySequence) -> Element {
    let hints = hints_for_context(&mode, pending);
    let prefix = pending_prefix(pending);
    let char_prompt = is_char_prompt(pending);

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
        }
    }
}
