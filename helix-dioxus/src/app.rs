//! Main application component.
//!
//! This is the root Dioxus component that composes the editor UI.

use dioxus::prelude::*;

use crate::editor_view::EditorView;
use crate::input::translate_key_event;
use crate::picker::FilePicker;
use crate::prompt::CommandPrompt;
use crate::state::EditorCommand;
use crate::statusline::StatusLine;
use crate::AppState;

/// Main application component.
#[component]
pub fn App() -> Element {
    // Get app state from context
    let app_state = use_context::<AppState>();

    // Track version for re-renders when editor state changes
    let mut version = use_signal(|| 0_usize);

    // Clone app_state for the closure
    let app_state_for_handler = app_state.clone();

    // Handle keyboard input at the app level
    let onkeydown = move |evt: KeyboardEvent| {
        log::debug!("Key pressed: {:?}", evt.key());

        // Get current mode
        let snapshot = app_state_for_handler.get_snapshot();

        // Translate to helix key event
        if let Some(key_event) = translate_key_event(&evt) {
            // Handle input based on UI state first, then editor mode
            let commands = if snapshot.picker_visible {
                handle_picker_mode(&key_event)
            } else if snapshot.command_mode {
                handle_command_mode(&key_event)
            } else {
                match snapshot.mode.as_str() {
                    "NORMAL" => handle_normal_mode(&key_event),
                    "INSERT" => handle_insert_mode(&key_event),
                    "SELECT" => handle_select_mode(&key_event),
                    _ => vec![],
                }
            };

            // Send commands to editor
            for cmd in commands {
                app_state_for_handler.send_command(cmd);
            }

            // Trigger re-render (snapshot will be refreshed by event handler)
            version += 1;

            // Prevent default browser behavior for handled keys
            evt.prevent_default();
        }
    };

    // Get snapshot for conditional rendering
    let snapshot = app_state.get_snapshot();

    rsx! {
        div {
            class: "app-container",
            tabindex: 0,
            onkeydown: onkeydown,
            style: "display: flex; flex-direction: column; height: 100vh; outline: none; position: relative;",

            // Editor view takes up most of the space
            div {
                style: "flex: 1; overflow: hidden;",
                EditorView { version: version() }
            }

            // Command prompt (shown when in command mode)
            if snapshot.command_mode {
                CommandPrompt { input: snapshot.command_input.clone() }
            }

            // Status line at the bottom
            StatusLine { version: version() }

            // File picker overlay (shown when picker is visible)
            if snapshot.picker_visible {
                FilePicker {
                    items: snapshot.picker_filtered.clone(),
                    selected: snapshot.picker_selected,
                    filter: snapshot.picker_filter.clone(),
                    total: snapshot.picker_total,
                }
            }
        }
    }
}

/// Handle keyboard input in Normal mode.
fn handle_normal_mode(key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    use helix_view::input::KeyCode;

    match key.code {
        // Movement
        KeyCode::Char('h') | KeyCode::Left => vec![EditorCommand::MoveLeft],
        KeyCode::Char('l') | KeyCode::Right => vec![EditorCommand::MoveRight],
        KeyCode::Char('j') | KeyCode::Down => vec![EditorCommand::MoveDown],
        KeyCode::Char('k') | KeyCode::Up => vec![EditorCommand::MoveUp],

        // Word movement
        KeyCode::Char('w') => vec![EditorCommand::MoveWordForward],
        KeyCode::Char('b') => vec![EditorCommand::MoveWordBackward],

        // Line movement
        KeyCode::Char('0') | KeyCode::Home => vec![EditorCommand::MoveLineStart],
        KeyCode::Char('$') | KeyCode::End => vec![EditorCommand::MoveLineEnd],

        // File navigation
        KeyCode::Char('G') => vec![EditorCommand::GotoLastLine],
        // TODO: 'gg' requires tracking previous key

        // Mode changes
        KeyCode::Char('i') => vec![EditorCommand::EnterInsertMode],
        KeyCode::Char('a') => vec![EditorCommand::EnterInsertModeAfter],
        KeyCode::Char('A') => vec![EditorCommand::EnterInsertModeLineEnd],
        KeyCode::Char('o') => vec![EditorCommand::OpenLineBelow],
        KeyCode::Char('O') => vec![EditorCommand::OpenLineAbove],

        // Command mode
        KeyCode::Char(':') => vec![EditorCommand::EnterCommandMode],

        _ => vec![],
    }
}

/// Handle keyboard input in Insert mode.
fn handle_insert_mode(key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    use helix_view::input::{KeyCode, KeyModifiers};

    // Check for Escape first
    if key.code == KeyCode::Esc {
        return vec![EditorCommand::ExitInsertMode];
    }

    // Handle control key combinations
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return vec![];
    }

    match key.code {
        KeyCode::Char(c) => vec![EditorCommand::InsertChar(c)],
        KeyCode::Enter => vec![EditorCommand::InsertNewline],
        KeyCode::Backspace => vec![EditorCommand::DeleteCharBackward],
        KeyCode::Delete => vec![EditorCommand::DeleteCharForward],
        KeyCode::Left => vec![EditorCommand::MoveLeft],
        KeyCode::Right => vec![EditorCommand::MoveRight],
        KeyCode::Up => vec![EditorCommand::MoveUp],
        KeyCode::Down => vec![EditorCommand::MoveDown],
        _ => vec![],
    }
}

/// Handle keyboard input in Select mode.
fn handle_select_mode(key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    use helix_view::input::KeyCode;

    match key.code {
        KeyCode::Esc => vec![EditorCommand::ExitInsertMode], // Also exits select mode
        KeyCode::Char('h') | KeyCode::Left => vec![EditorCommand::ExtendLeft],
        KeyCode::Char('l') | KeyCode::Right => vec![EditorCommand::ExtendRight],
        KeyCode::Char('j') | KeyCode::Down => vec![EditorCommand::ExtendDown],
        KeyCode::Char('k') | KeyCode::Up => vec![EditorCommand::ExtendUp],
        _ => vec![],
    }
}

/// Handle keyboard input in Command mode.
fn handle_command_mode(key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    use helix_view::input::KeyCode;

    match key.code {
        KeyCode::Esc => vec![EditorCommand::ExitCommandMode],
        KeyCode::Enter => vec![EditorCommand::CommandExecute],
        KeyCode::Backspace => vec![EditorCommand::CommandBackspace],
        KeyCode::Char(c) => vec![EditorCommand::CommandInput(c)],
        _ => vec![],
    }
}

/// Handle keyboard input in File picker mode.
fn handle_picker_mode(key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    use helix_view::input::{KeyCode, KeyModifiers};

    // Ctrl+n/p for navigation (like many fuzzy finders)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('n') => vec![EditorCommand::PickerDown],
            KeyCode::Char('p') => vec![EditorCommand::PickerUp],
            _ => vec![],
        };
    }

    match key.code {
        KeyCode::Esc => vec![EditorCommand::PickerCancel],
        KeyCode::Enter => vec![EditorCommand::PickerConfirm],
        KeyCode::Down => vec![EditorCommand::PickerDown],
        KeyCode::Up => vec![EditorCommand::PickerUp],
        KeyCode::Backspace => vec![EditorCommand::PickerBackspace],
        KeyCode::Char(c) => vec![EditorCommand::PickerInput(c)],
        _ => vec![],
    }
}
