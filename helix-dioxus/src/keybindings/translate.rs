//! Keyboard input handling and translation.
//!
//! Translates Dioxus keyboard events to helix KeyEvent format.

use dioxus::prelude::*;
use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

/// Translate a Dioxus keyboard event to a helix KeyEvent.
pub fn translate_key_event(evt: &KeyboardEvent) -> Option<KeyEvent> {
    let code = translate_key_code(evt)?;
    let modifiers = translate_modifiers(evt);

    Some(KeyEvent { code, modifiers })
}

/// Translate Dioxus key to helix KeyCode.
fn translate_key_code(evt: &KeyboardEvent) -> Option<KeyCode> {
    use dioxus::prelude::Key;

    let key = evt.key();

    match key {
        // Letters and characters
        Key::Character(c) => {
            let ch = c.chars().next()?;
            Some(KeyCode::Char(ch))
        }

        // Function keys
        Key::F1 => Some(KeyCode::F(1)),
        Key::F2 => Some(KeyCode::F(2)),
        Key::F3 => Some(KeyCode::F(3)),
        Key::F4 => Some(KeyCode::F(4)),
        Key::F5 => Some(KeyCode::F(5)),
        Key::F6 => Some(KeyCode::F(6)),
        Key::F7 => Some(KeyCode::F(7)),
        Key::F8 => Some(KeyCode::F(8)),
        Key::F9 => Some(KeyCode::F(9)),
        Key::F10 => Some(KeyCode::F(10)),
        Key::F11 => Some(KeyCode::F(11)),
        Key::F12 => Some(KeyCode::F(12)),

        // Navigation
        Key::ArrowUp => Some(KeyCode::Up),
        Key::ArrowDown => Some(KeyCode::Down),
        Key::ArrowLeft => Some(KeyCode::Left),
        Key::ArrowRight => Some(KeyCode::Right),
        Key::Home => Some(KeyCode::Home),
        Key::End => Some(KeyCode::End),
        Key::PageUp => Some(KeyCode::PageUp),
        Key::PageDown => Some(KeyCode::PageDown),

        // Editing
        Key::Backspace => Some(KeyCode::Backspace),
        Key::Delete => Some(KeyCode::Delete),
        Key::Insert => Some(KeyCode::Insert),
        Key::Enter => Some(KeyCode::Enter),
        Key::Tab => Some(KeyCode::Tab),
        Key::Escape => Some(KeyCode::Esc),

        _ => None,
    }
}

/// Translate Dioxus modifiers to helix KeyModifiers.
fn translate_modifiers(evt: &KeyboardEvent) -> KeyModifiers {
    let mods = evt.modifiers();
    let mut result = KeyModifiers::NONE;

    if mods.shift() {
        result |= KeyModifiers::SHIFT;
    }
    if mods.ctrl() {
        result |= KeyModifiers::CONTROL;
    }
    if mods.alt() {
        result |= KeyModifiers::ALT;
    }

    result
}
