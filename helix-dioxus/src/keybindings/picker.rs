//! Picker mode keybinding handler.

use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

use crate::config::DialogSearchMode;
use crate::state::{EditorCommand, PickerMode};

/// Handle keyboard input in File picker mode.
///
/// Behavior depends on the `dialog_search_mode` config:
/// - `Direct` (default): typing filters directly, arrows navigate.
/// - `VimStyle`: j/k navigate, `/` focuses search, typing only filters when focused.
pub fn handle_picker_mode(
    key: &KeyEvent,
    search_mode: DialogSearchMode,
    search_focused: bool,
    picker_mode: PickerMode,
) -> Vec<EditorCommand> {
    // Ctrl+n/p for navigation (both modes)
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('n') => vec![EditorCommand::PickerDown],
            KeyCode::Char('p') => vec![EditorCommand::PickerUp],
            _ => vec![],
        };
    }

    match search_mode {
        DialogSearchMode::Direct => handle_picker_direct(key, picker_mode),
        DialogSearchMode::VimStyle => handle_picker_vim(key, search_focused, picker_mode),
    }
}

/// Direct mode: typing filters, arrows navigate.
fn handle_picker_direct(key: &KeyEvent, picker_mode: PickerMode) -> Vec<EditorCommand> {
    let is_explorer = picker_mode == PickerMode::FileExplorer;
    match key.code {
        KeyCode::Esc => vec![EditorCommand::PickerCancel],
        KeyCode::Enter => vec![EditorCommand::PickerConfirm],
        KeyCode::Down => vec![EditorCommand::PickerDown],
        KeyCode::Up => vec![EditorCommand::PickerUp],
        KeyCode::Left if is_explorer => vec![EditorCommand::ExplorerCollapseOrParent],
        KeyCode::Right if is_explorer => vec![EditorCommand::ExplorerExpand],
        KeyCode::Home => vec![EditorCommand::PickerFirst],
        KeyCode::End => vec![EditorCommand::PickerLast],
        KeyCode::PageUp => vec![EditorCommand::PickerPageUp],
        KeyCode::PageDown => vec![EditorCommand::PickerPageDown],
        KeyCode::Backspace => vec![EditorCommand::PickerBackspace],
        KeyCode::Char(ch) => vec![EditorCommand::PickerInput(ch)],
        _ => vec![],
    }
}

/// Vim-style mode: j/k navigate, `/` focuses search.
fn handle_picker_vim(
    key: &KeyEvent,
    search_focused: bool,
    picker_mode: PickerMode,
) -> Vec<EditorCommand> {
    let is_explorer = picker_mode == PickerMode::FileExplorer;
    if search_focused {
        // Search input is focused: typing filters, Esc unfocuses, Enter confirms search
        match key.code {
            KeyCode::Esc | KeyCode::Enter => vec![EditorCommand::PickerUnfocusSearch],
            KeyCode::Backspace => vec![EditorCommand::PickerBackspace],
            KeyCode::Char(ch) => vec![EditorCommand::PickerInput(ch)],
            // Allow arrow navigation even when search is focused
            KeyCode::Down => vec![EditorCommand::PickerDown],
            KeyCode::Up => vec![EditorCommand::PickerUp],
            _ => vec![],
        }
    } else {
        // Navigation mode: j/k and arrows navigate, / focuses search
        match key.code {
            KeyCode::Esc => vec![EditorCommand::PickerCancel],
            KeyCode::Enter => vec![EditorCommand::PickerConfirm],
            KeyCode::Down | KeyCode::Char('j') => vec![EditorCommand::PickerDown],
            KeyCode::Up | KeyCode::Char('k') => vec![EditorCommand::PickerUp],
            KeyCode::Left | KeyCode::Char('h') if is_explorer => {
                vec![EditorCommand::ExplorerCollapseOrParent]
            }
            KeyCode::Right | KeyCode::Char('l') if is_explorer => {
                vec![EditorCommand::ExplorerExpand]
            }
            KeyCode::Char('/') => vec![EditorCommand::PickerFocusSearch],
            KeyCode::Char('g') => vec![EditorCommand::PickerFirst],
            KeyCode::Char('G') => vec![EditorCommand::PickerLast],
            KeyCode::Home => vec![EditorCommand::PickerFirst],
            KeyCode::End => vec![EditorCommand::PickerLast],
            KeyCode::PageUp => vec![EditorCommand::PickerPageUp],
            KeyCode::PageDown => vec![EditorCommand::PickerPageDown],
            _ => vec![],
        }
    }
}
