//! Main application component.
//!
//! This is the root Dioxus component that composes the editor UI.

use dioxus::prelude::*;

use crate::components::{
    BufferBar, CommandPrompt, EditorView, GenericPicker, SearchPrompt, StatusLine,
};
use crate::input::translate_key_event;
use crate::keybindings::{
    handle_command_mode, handle_insert_mode, handle_normal_mode, handle_picker_mode,
    handle_search_mode, handle_select_mode,
};
use crate::AppState;

/// Main application component.
#[component]
pub fn App() -> Element {
    // Get app state from context
    let app_state = use_context::<AppState>();

    // Track version for re-renders when editor state changes
    let mut version = use_signal(|| 0_usize);

    // Read the signal to subscribe to changes and trigger re-render of this component
    let _ = version();

    // Auto-focus the app container on mount
    use_effect(|| {
        document::eval("focusAppContainer();");
    });

    // Clone app_state for the closure
    let app_state_for_handler = app_state.clone();

    // Handle keyboard input at the app level
    let onkeydown = move |evt: KeyboardEvent| {
        log::info!("Key pressed: {:?}", evt.key());

        // Get current mode
        let snapshot = app_state_for_handler.get_snapshot();

        // Translate to helix key event
        if let Some(key_event) = translate_key_event(&evt) {
            log::info!(
                "Mode: {}, command_mode: {}, search_mode: {}, picker_visible: {}",
                snapshot.mode,
                snapshot.command_mode,
                snapshot.search_mode,
                snapshot.picker_visible
            );

            // Handle input based on UI state first, then editor mode
            let commands = if snapshot.picker_visible {
                handle_picker_mode(&key_event)
            } else if snapshot.command_mode {
                handle_command_mode(&key_event)
            } else if snapshot.search_mode {
                handle_search_mode(&key_event)
            } else {
                match snapshot.mode.as_str() {
                    "NORMAL" => handle_normal_mode(&key_event),
                    "INSERT" => handle_insert_mode(&key_event),
                    "SELECT" => handle_select_mode(&key_event),
                    _ => vec![],
                }
            };

            // Send commands to editor
            log::info!("Commands: {:?}", commands);
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

    log::info!(
        "Rendering App: command_mode={}, search_mode={}, picker_visible={}",
        snapshot.command_mode,
        snapshot.search_mode,
        snapshot.picker_visible
    );

    rsx! {
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

            // Status line at the bottom
            StatusLine { version: version }

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
        }
    }
}
