//! Main application component.
//!
//! This is the root Dioxus component that composes the editor UI.

use dioxus::prelude::*;
use helix_view::input::{KeyCode, KeyModifiers};

use crate::components::{
    BufferBar, CodeActionsMenu, CommandPrompt, CompletionPopup, ConfirmationDialog, EditorView,
    GenericPicker, HoverPopup, InputDialog, KeybindingHelpBar, LocationPicker, LspStatusDialog,
    NotificationContainer, SearchPrompt, SignatureHelpPopup, StatusLine,
};
use crate::keybindings::{
    handle_bracket_next, handle_bracket_prev, handle_code_actions_mode, handle_command_mode,
    handle_completion_mode, handle_confirmation_mode, handle_g_prefix, handle_input_dialog_mode,
    handle_insert_mode, handle_location_picker_mode, handle_lsp_dialog_mode, handle_normal_mode,
    handle_picker_mode, handle_search_mode, handle_select_mode, handle_space_leader,
    handle_view_prefix, translate_key_event,
};
use crate::state::{EditorCommand, PendingKeySequence};
use crate::AppState;

/// Main application component.
#[component]
pub fn App() -> Element {
    // Get app state from context
    let app_state = use_context::<AppState>();

    // Track version for re-renders when editor state changes
    let mut version = use_signal(|| 0_usize);

    // Track pending key sequence for multi-key commands (g, ], [, Space)
    let mut pending_key = use_signal(PendingKeySequence::default);

    // Read the signal to subscribe to changes and trigger re-render of this component
    let _ = version();

    // Auto-focus the app container on mount
    use_effect(|| {
        document::eval("focusAppContainer();");
    });

    // Track the last seen snapshot version to avoid unnecessary re-renders
    let mut last_snapshot_version = use_signal(|| 0_u64);

    // Background coroutine to poll for LSP events (diagnostics, etc.)
    // This ensures UI updates when async events arrive without keyboard input.
    let app_state_for_poll = app_state.clone();
    use_future(move || {
        let app_state = app_state_for_poll.clone();
        async move {
            log::info!("LSP polling coroutine started");
            loop {
                // Wait for a short interval before polling
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                // Process any pending commands (including LSP events)
                app_state.process_commands_sync();

                // Only trigger re-render if snapshot actually changed
                let snapshot = app_state.get_snapshot();
                let current_version = snapshot.snapshot_version;
                if current_version != last_snapshot_version() {
                    log::info!(
                        "Snapshot changed: v{} -> v{}, diagnostics: {}, errors: {}, warnings: {}",
                        last_snapshot_version(),
                        current_version,
                        snapshot.diagnostics.len(),
                        snapshot.error_count,
                        snapshot.warning_count
                    );
                    last_snapshot_version.set(current_version);
                    version += 1;
                }
            }
        }
    });

    // Clone app_state for the closure
    let app_state_for_handler = app_state.clone();

    // Handle keyboard input at the app level
    let onkeydown = move |evt: KeyboardEvent| {
        log::trace!("Key pressed: {:?}", evt.key());

        // Get current mode
        let snapshot = app_state_for_handler.get_snapshot();

        // Translate to helix key event
        if let Some(key_event) = translate_key_event(&evt) {
            log::trace!(
                "Mode: {}, command_mode: {}, search_mode: {}, picker_visible: {}, pending_key: {:?}",
                snapshot.mode,
                snapshot.command_mode,
                snapshot.search_mode,
                snapshot.picker_visible,
                pending_key()
            );

            // Handle input based on UI state first, then editor mode
            // Confirmation dialog takes highest precedence, then input dialog
            let commands = if snapshot.confirmation_dialog_visible {
                handle_confirmation_mode(&key_event)
            } else if snapshot.input_dialog_visible {
                handle_input_dialog_mode(&key_event)
            } else if snapshot.lsp_dialog_visible {
                handle_lsp_dialog_mode(&key_event)
            } else if snapshot.location_picker_visible {
                handle_location_picker_mode(&key_event)
            } else if snapshot.code_actions_visible {
                handle_code_actions_mode(&key_event)
            } else if snapshot.completion_visible {
                handle_completion_mode(&key_event)
            } else if snapshot.picker_visible {
                handle_picker_mode(&key_event)
            } else if snapshot.command_mode {
                handle_command_mode(&key_event)
            } else if snapshot.search_mode {
                handle_search_mode(&key_event)
            } else if snapshot.mode == "NORMAL" || snapshot.mode == "SELECT" {
                // Handle multi-key sequences in normal/select mode
                let current_pending = pending_key();
                match current_pending {
                    PendingKeySequence::GPrefix => {
                        pending_key.set(PendingKeySequence::None);
                        handle_g_prefix(&key_event)
                    }
                    PendingKeySequence::BracketNext => {
                        pending_key.set(PendingKeySequence::None);
                        handle_bracket_next(&key_event)
                    }
                    PendingKeySequence::BracketPrev => {
                        pending_key.set(PendingKeySequence::None);
                        handle_bracket_prev(&key_event)
                    }
                    PendingKeySequence::SpaceLeader => {
                        pending_key.set(PendingKeySequence::None);
                        handle_space_leader(&key_event)
                    }
                    PendingKeySequence::FindForward => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            if snapshot.mode == "SELECT" {
                                vec![EditorCommand::ExtendFindCharForward(ch)]
                            } else {
                                vec![EditorCommand::FindCharForward(ch)]
                            }
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::FindBackward => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            if snapshot.mode == "SELECT" {
                                vec![EditorCommand::ExtendFindCharBackward(ch)]
                            } else {
                                vec![EditorCommand::FindCharBackward(ch)]
                            }
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::TillForward => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            if snapshot.mode == "SELECT" {
                                vec![EditorCommand::ExtendTillCharForward(ch)]
                            } else {
                                vec![EditorCommand::TillCharForward(ch)]
                            }
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::TillBackward => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            if snapshot.mode == "SELECT" {
                                vec![EditorCommand::ExtendTillCharBackward(ch)]
                            } else {
                                vec![EditorCommand::TillCharBackward(ch)]
                            }
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::RegisterPrefix => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            vec![EditorCommand::SetSelectedRegister(ch)]
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::ReplacePrefix => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            vec![EditorCommand::ReplaceChar(ch)]
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::MatchPrefix => match key_event.code {
                        KeyCode::Char('m') => {
                            pending_key.set(PendingKeySequence::None);
                            vec![EditorCommand::MatchBracket]
                        }
                        KeyCode::Char('i') => {
                            pending_key.set(PendingKeySequence::MatchInside);
                            vec![]
                        }
                        KeyCode::Char('a') => {
                            pending_key.set(PendingKeySequence::MatchAround);
                            vec![]
                        }
                        KeyCode::Char('s') => {
                            pending_key.set(PendingKeySequence::MatchSurround);
                            vec![]
                        }
                        KeyCode::Char('d') => {
                            pending_key.set(PendingKeySequence::MatchDeleteSurround);
                            vec![]
                        }
                        KeyCode::Char('r') => {
                            pending_key.set(PendingKeySequence::MatchReplaceSurroundFrom);
                            vec![]
                        }
                        _ => {
                            pending_key.set(PendingKeySequence::None);
                            vec![]
                        }
                    },
                    PendingKeySequence::MatchInside => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            vec![EditorCommand::SelectInsidePair(ch)]
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::MatchAround => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            vec![EditorCommand::SelectAroundPair(ch)]
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::MatchSurround => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            vec![EditorCommand::SurroundAdd(ch)]
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::MatchDeleteSurround => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(ch) = key_event.code {
                            vec![EditorCommand::SurroundDelete(ch)]
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::MatchReplaceSurroundFrom => {
                        if let KeyCode::Char(old) = key_event.code {
                            pending_key.set(PendingKeySequence::MatchReplaceSurroundTo(old));
                        } else {
                            pending_key.set(PendingKeySequence::None);
                        }
                        vec![]
                    }
                    PendingKeySequence::MatchReplaceSurroundTo(old) => {
                        pending_key.set(PendingKeySequence::None);
                        if let KeyCode::Char(new) = key_event.code {
                            vec![EditorCommand::SurroundReplace(old, new)]
                        } else {
                            vec![]
                        }
                    }
                    PendingKeySequence::ViewPrefix => {
                        pending_key.set(PendingKeySequence::None);
                        handle_view_prefix(&key_event)
                    }
                    PendingKeySequence::ViewPrefixSticky => {
                        if key_event.code == KeyCode::Esc {
                            pending_key.set(PendingKeySequence::None);
                            vec![]
                        } else {
                            let cmds = handle_view_prefix(&key_event);
                            // Exit sticky mode on unrecognized key
                            if cmds.is_empty() {
                                pending_key.set(PendingKeySequence::None);
                            }
                            cmds
                        }
                    }
                    PendingKeySequence::None => {
                        // Check for Ctrl modifier first - Ctrl+key combos go to normal mode handler
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                            if snapshot.mode == "SELECT" {
                                handle_select_mode(&key_event)
                            } else {
                                handle_normal_mode(&key_event)
                            }
                        } else {
                            // Check if this starts a sequence (only without modifiers)
                            match key_event.code {
                                KeyCode::Char('g') => {
                                    pending_key.set(PendingKeySequence::GPrefix);
                                    vec![]
                                }
                                KeyCode::Char(']') => {
                                    pending_key.set(PendingKeySequence::BracketNext);
                                    vec![]
                                }
                                KeyCode::Char('[') => {
                                    pending_key.set(PendingKeySequence::BracketPrev);
                                    vec![]
                                }
                                KeyCode::Char(' ') => {
                                    pending_key.set(PendingKeySequence::SpaceLeader);
                                    vec![]
                                }
                                KeyCode::Char('f') => {
                                    pending_key.set(PendingKeySequence::FindForward);
                                    vec![]
                                }
                                KeyCode::Char('F') => {
                                    pending_key.set(PendingKeySequence::FindBackward);
                                    vec![]
                                }
                                KeyCode::Char('t') => {
                                    pending_key.set(PendingKeySequence::TillForward);
                                    vec![]
                                }
                                KeyCode::Char('T') => {
                                    pending_key.set(PendingKeySequence::TillBackward);
                                    vec![]
                                }
                                KeyCode::Char('r') => {
                                    pending_key.set(PendingKeySequence::ReplacePrefix);
                                    vec![]
                                }
                                KeyCode::Char('m') => {
                                    pending_key.set(PendingKeySequence::MatchPrefix);
                                    vec![]
                                }
                                KeyCode::Char('"') => {
                                    pending_key.set(PendingKeySequence::RegisterPrefix);
                                    vec![]
                                }
                                KeyCode::Char('z') => {
                                    pending_key.set(PendingKeySequence::ViewPrefix);
                                    vec![]
                                }
                                KeyCode::Char('Z') => {
                                    pending_key.set(PendingKeySequence::ViewPrefixSticky);
                                    vec![]
                                }
                                _ => {
                                    if snapshot.mode == "SELECT" {
                                        handle_select_mode(&key_event)
                                    } else {
                                        handle_normal_mode(&key_event)
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                match snapshot.mode.as_str() {
                    "INSERT" => handle_insert_mode(&key_event),
                    _ => vec![],
                }
            };

            // Send commands to editor
            log::trace!("Commands: {:?}", commands);
            for cmd in commands {
                app_state_for_handler.send_command(cmd);
            }

            // Process commands synchronously and update snapshot before triggering re-render
            app_state_for_handler.process_commands_sync();

            // Trigger re-render with updated snapshot
            version += 1;

            // Prevent default browser behavior for handled keys
            evt.prevent_default();
        }
    };

    // Get snapshot for conditional rendering
    let snapshot = app_state.get_snapshot();

    rsx! {
        // Load external stylesheet
        document::Stylesheet { href: asset!("/assets/styles.css") }

        // Dynamic window title based on current buffer
        document::Title { "helix-dioxus - {snapshot.file_name}" }

        div {
            class: "app-container",
            tabindex: 0,
            onkeydown: onkeydown,
            // position: relative needed for picker overlay positioning
            style: "position: relative;",

            // Buffer bar at the top
            BufferBar {
                version: version,
                on_change: move |_| {
                    version += 1;
                },
            }

            // Editor view takes up most of the space
            div {
                style: "flex: 1; overflow: hidden;",
                EditorView { version: version }
            }

            // Command prompt (shown when in command mode)
            if snapshot.command_mode {
                CommandPrompt { input: snapshot.command_input.clone() }
            }

            // Search prompt (shown when in search mode)
            if snapshot.search_mode {
                SearchPrompt {
                    input: snapshot.search_input.clone(),
                    backwards: snapshot.search_backwards,
                }
            }

            // Keybinding help bar (above statusline)
            KeybindingHelpBar {
                mode: snapshot.mode.clone(),
                pending: *pending_key.read(),
                register_snapshots: snapshot.registers.clone(),
            }

            // Status line at the bottom
            StatusLine {
                version: version,
                on_change: move |_| {
                    version += 1;
                },
            }

            // Generic picker overlay (shown when picker is visible)
            if snapshot.picker_visible {
                GenericPicker {
                    items: snapshot.picker_items.clone(),
                    selected: snapshot.picker_selected,
                    filter: snapshot.picker_filter.clone(),
                    total: snapshot.picker_total,
                    mode: snapshot.picker_mode,
                    current_path: snapshot.picker_current_path.clone(),
                }
            }

            // LSP - Completion popup
            if snapshot.completion_visible {
                CompletionPopup {
                    items: snapshot.completion_items.clone(),
                    selected: snapshot.completion_selected,
                    cursor_line: snapshot.cursor_line,
                    cursor_col: snapshot.cursor_col,
                }
            }

            // LSP - Hover popup
            if snapshot.hover_visible {
                if let Some(ref hover) = snapshot.hover_content {
                    HoverPopup {
                        hover: hover.clone(),
                        cursor_line: snapshot.cursor_line,
                        cursor_col: snapshot.cursor_col,
                    }
                }
            }

            // LSP - Signature help popup
            if snapshot.signature_help_visible {
                if let Some(ref sig_help) = snapshot.signature_help {
                    SignatureHelpPopup {
                        signature_help: sig_help.clone(),
                        cursor_line: snapshot.cursor_line,
                        cursor_col: snapshot.cursor_col,
                    }
                }
            }

            // LSP - Code actions menu
            if snapshot.code_actions_visible {
                CodeActionsMenu {
                    actions: snapshot.code_actions.clone(),
                    selected: snapshot.code_action_selected,
                    cursor_line: snapshot.cursor_line,
                    cursor_col: snapshot.cursor_col,
                    filter: snapshot.code_action_filter.clone(),
                }
            }

            // LSP - Location picker
            if snapshot.location_picker_visible {
                LocationPicker {
                    title: snapshot.location_picker_title.clone(),
                    locations: snapshot.locations.clone(),
                    selected: snapshot.location_selected,
                }
            }

            // LSP - Status dialog
            if snapshot.lsp_dialog_visible {
                LspStatusDialog {
                    servers: snapshot.lsp_servers.clone(),
                    selected: snapshot.lsp_server_selected,
                    on_change: move |_| {
                        version += 1;
                    },
                }
            }

            // Input dialog (for rename, goto line, etc.)
            if snapshot.input_dialog_visible {
                InputDialog {
                    dialog: snapshot.input_dialog.clone(),
                    cursor_line: snapshot.cursor_line,
                    cursor_col: snapshot.cursor_col,
                }
            }

            // Confirmation dialog (for quit with unsaved changes, etc.)
            if snapshot.confirmation_dialog_visible {
                ConfirmationDialog {
                    dialog: snapshot.confirmation_dialog.clone(),
                    on_change: move |_| {
                        version += 1;
                    },
                }
            }

            // Notification toasts (bottom-right corner)
            NotificationContainer {
                notifications: snapshot.notifications.clone(),
            }
        }
    }
}
