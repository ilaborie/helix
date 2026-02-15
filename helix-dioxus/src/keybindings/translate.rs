//! Keyboard input handling and translation.
//!
//! Translates Dioxus keyboard events to helix `KeyEvent` format.

use dioxus::html::keyboard_types::Code;
use dioxus::prelude::*;
use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};

/// Translate a Dioxus keyboard event to a helix `KeyEvent`.
///
/// Normalizes SHIFT for character keys: Dioxus reports `Shift` in modifiers
/// for all shifted characters (`:`, `D`, `!`, etc.), but helix's keymap
/// stores them as plain `Char(':')` / `Char('D')` with no SHIFT modifier
/// (the shift is already encoded in the character). We strip SHIFT for
/// `Char` codes so the trie lookup matches correctly.
#[must_use]
pub fn translate_key_event(evt: &KeyboardEvent) -> Option<KeyEvent> {
    let code = translate_key_code(evt)?;
    let mut modifiers = translate_modifiers(evt);

    // Strip SHIFT for character keys — the character itself already encodes
    // the shift (e.g., ':' vs ';', 'D' vs 'd'). Keep SHIFT only for
    // non-character keys like Shift+Tab, Shift+Left, etc.
    if matches!(code, KeyCode::Char(_)) {
        modifiers.remove(KeyModifiers::SHIFT);
    }

    Some(KeyEvent { code, modifiers })
}

/// Translate Dioxus key to helix `KeyCode`.
fn translate_key_code(evt: &KeyboardEvent) -> Option<KeyCode> {
    use dioxus::prelude::Key;

    let key = evt.key();

    // Debug: log all key events to help diagnose issues
    log::trace!("translate_key_code: key={:?}, code={}", key, evt.code());

    // On macOS, Alt (Option) composes special characters (e.g., Alt+o → ø).
    // Use the physical key code to get the intended character.
    if evt.modifiers().alt() {
        if let Some(code) = key_code_from_physical(evt.code(), evt.modifiers().shift()) {
            return Some(code);
        }
    }

    match key {
        // Space key (handle explicitly for Ctrl+Space)
        Key::Character(ref c) if c == " " => Some(KeyCode::Char(' ')),

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

/// Map physical key code to [`KeyCode`] for Alt+key normalization on macOS.
///
/// When the Alt (Option) key is pressed on macOS, the OS composes special characters
/// (e.g., Alt+o → ø, Alt+i → ˆ). This function maps the physical key code back to
/// the intended character so keybindings like Alt+o work correctly.
fn key_code_from_physical(code: Code, shift: bool) -> Option<KeyCode> {
    let ch = match code {
        Code::KeyA => 'a',
        Code::KeyB => 'b',
        Code::KeyC => 'c',
        Code::KeyD => 'd',
        Code::KeyE => 'e',
        Code::KeyF => 'f',
        Code::KeyG => 'g',
        Code::KeyH => 'h',
        Code::KeyI => 'i',
        Code::KeyJ => 'j',
        Code::KeyK => 'k',
        Code::KeyL => 'l',
        Code::KeyM => 'm',
        Code::KeyN => 'n',
        Code::KeyO => 'o',
        Code::KeyP => 'p',
        Code::KeyQ => 'q',
        Code::KeyR => 'r',
        Code::KeyS => 's',
        Code::KeyT => 't',
        Code::KeyU => 'u',
        Code::KeyV => 'v',
        Code::KeyW => 'w',
        Code::KeyX => 'x',
        Code::KeyY => 'y',
        Code::KeyZ => 'z',
        Code::Digit0 => '0',
        Code::Digit1 => '1',
        Code::Digit2 => '2',
        Code::Digit3 => '3',
        Code::Digit4 => '4',
        Code::Digit5 => '5',
        Code::Digit6 => '6',
        Code::Digit7 => '7',
        Code::Digit8 => '8',
        Code::Digit9 => '9',
        Code::Period => '.',
        Code::Comma => ',',
        Code::Semicolon => ';',
        Code::Quote => '\'',
        Code::Backquote => '`',
        Code::Slash => '/',
        Code::Backslash => '\\',
        Code::Minus => '-',
        Code::Equal => '=',
        Code::BracketLeft => '[',
        Code::BracketRight => ']',
        Code::Space => ' ',
        _ => return None,
    };
    let ch = if shift { ch.to_ascii_uppercase() } else { ch };
    Some(KeyCode::Char(ch))
}

/// Translate Dioxus modifiers to helix `KeyModifiers`.
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
    if mods.meta() {
        result |= KeyModifiers::SUPER;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_key_letters() {
        assert_eq!(key_code_from_physical(Code::KeyO, false), Some(KeyCode::Char('o')));
        assert_eq!(key_code_from_physical(Code::KeyI, false), Some(KeyCode::Char('i')));
        assert_eq!(key_code_from_physical(Code::KeyA, false), Some(KeyCode::Char('a')));
        assert_eq!(key_code_from_physical(Code::KeyZ, false), Some(KeyCode::Char('z')));
    }

    #[test]
    fn physical_key_shift_uppercases() {
        assert_eq!(key_code_from_physical(Code::KeyC, true), Some(KeyCode::Char('C')));
        assert_eq!(key_code_from_physical(Code::KeyS, true), Some(KeyCode::Char('S')));
    }

    #[test]
    fn physical_key_digits() {
        assert_eq!(key_code_from_physical(Code::Digit0, false), Some(KeyCode::Char('0')));
        assert_eq!(key_code_from_physical(Code::Digit9, false), Some(KeyCode::Char('9')));
    }

    #[test]
    fn physical_key_punctuation() {
        assert_eq!(key_code_from_physical(Code::Period, false), Some(KeyCode::Char('.')));
        assert_eq!(key_code_from_physical(Code::Semicolon, false), Some(KeyCode::Char(';')));
        assert_eq!(key_code_from_physical(Code::Backquote, false), Some(KeyCode::Char('`')));
        assert_eq!(key_code_from_physical(Code::Minus, false), Some(KeyCode::Char('-')));
    }

    #[test]
    fn physical_key_unknown_returns_none() {
        assert_eq!(key_code_from_physical(Code::AltLeft, false), None);
        assert_eq!(key_code_from_physical(Code::Enter, false), None);
        assert_eq!(key_code_from_physical(Code::Escape, false), None);
    }
}
