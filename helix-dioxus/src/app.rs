//! Main application component.
//!
//! This is the root Dioxus component that composes the editor UI.

use dioxus::prelude::*;

use crate::buffer_bar::BufferBar;
use crate::editor_view::EditorView;
use crate::input::translate_key_event;
use crate::picker::GenericPicker;
use crate::prompt::{CommandPrompt, SearchPrompt};
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

    // Read the signal to subscribe to changes and trigger re-render of this component
    let _ = version();

    // Auto-focus the app container on mount
    use_effect(|| {
        document::eval(
            r#"
            requestAnimationFrame(() => {
                const container = document.querySelector('.app-container');
                if (container) {
                    container.focus();
                }
            });
        "#,
        );
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
            style: "display: flex; flex-direction: column; height: 100vh; outline: none; position: relative;",

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

/// Handle keyboard input in Normal mode.
fn handle_normal_mode(key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    use helix_view::input::{KeyCode, KeyModifiers};

    // Handle Ctrl+key combinations first
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('r') => vec![EditorCommand::Redo],
            KeyCode::Char('h') => vec![EditorCommand::PreviousBuffer],
            KeyCode::Char('l') => vec![EditorCommand::NextBuffer],
            _ => vec![],
        };
    }

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

        // History
        KeyCode::Char('u') => vec![EditorCommand::Undo],
        KeyCode::Char('U') => vec![EditorCommand::Redo], // Shift+U also redoes (helix convention)

        // Visual selection mode
        KeyCode::Char('v') => vec![EditorCommand::EnterSelectMode],

        // Line selection (helix x/X)
        KeyCode::Char('x') => vec![EditorCommand::SelectLine],

        // Delete selection (works in normal mode due to selection-first model)
        KeyCode::Char('d') => vec![EditorCommand::DeleteSelection],

        // Clipboard
        KeyCode::Char('p') => vec![EditorCommand::Paste],
        KeyCode::Char('P') => vec![EditorCommand::PasteBefore],
        KeyCode::Char('y') => vec![EditorCommand::Yank],

        // Search
        KeyCode::Char('/') => vec![EditorCommand::EnterSearchMode { backwards: false }],
        KeyCode::Char('?') => vec![EditorCommand::EnterSearchMode { backwards: true }],
        KeyCode::Char('n') => vec![EditorCommand::SearchNext],
        KeyCode::Char('N') => vec![EditorCommand::SearchPrevious],

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
        // Exit select mode
        KeyCode::Esc => vec![EditorCommand::ExitSelectMode],

        // Character movement - extends selection
        KeyCode::Char('h') | KeyCode::Left => vec![EditorCommand::ExtendLeft],
        KeyCode::Char('l') | KeyCode::Right => vec![EditorCommand::ExtendRight],
        KeyCode::Char('j') | KeyCode::Down => vec![EditorCommand::ExtendDown],
        KeyCode::Char('k') | KeyCode::Up => vec![EditorCommand::ExtendUp],

        // Word movement - extends selection
        KeyCode::Char('w') => vec![EditorCommand::ExtendWordForward],
        KeyCode::Char('b') => vec![EditorCommand::ExtendWordBackward],

        // Line movement - extends selection
        KeyCode::Char('0') | KeyCode::Home => vec![EditorCommand::ExtendLineStart],
        KeyCode::Char('$') | KeyCode::End => vec![EditorCommand::ExtendLineEnd],

        // Line selection
        KeyCode::Char('x') => vec![EditorCommand::SelectLine],
        KeyCode::Char('X') => vec![EditorCommand::ExtendLine],

        // Clipboard operations
        KeyCode::Char('y') => vec![EditorCommand::Yank, EditorCommand::ExitSelectMode],
        KeyCode::Char('d') => vec![EditorCommand::DeleteSelection],

        // Paste replaces selection
        KeyCode::Char('p') => vec![EditorCommand::DeleteSelection, EditorCommand::Paste],

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
        KeyCode::Char(ch) => vec![EditorCommand::PickerInput(ch)],
        _ => vec![],
    }
}

/// Handle keyboard input in Search mode.
fn handle_search_mode(key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    use helix_view::input::KeyCode;

    match key.code {
        KeyCode::Esc => vec![EditorCommand::ExitSearchMode],
        KeyCode::Enter => vec![EditorCommand::SearchExecute],
        KeyCode::Backspace => vec![EditorCommand::SearchBackspace],
        KeyCode::Char(ch) => vec![EditorCommand::SearchInput(ch)],
        _ => vec![],
    }
}
