//! Macro recording and replay operations.

use helix_view::input::KeyEvent;

use crate::state::EditorContext;

/// Extension trait for macro recording/replay on `EditorContext`.
pub trait MacroOps {
    /// Toggle macro recording. If not recording, starts recording to the
    /// selected register (or `@` by default). If recording, stops and writes
    /// the recorded keys to the register.
    fn toggle_macro_recording(&mut self);

    /// Record a key event if currently recording a macro and not replaying.
    /// Should be called before dispatching the key.
    fn maybe_record_key(&mut self, key: &KeyEvent);

    /// Replay a macro from the selected register (or `@` by default).
    /// Parses the register content, then replays each key.
    fn replay_macro(&mut self);
}

impl MacroOps for EditorContext {
    fn toggle_macro_recording(&mut self) {
        if let Some((reg, keys)) = self.editor.macro_recording.take() {
            // Stop recording: serialize keys and write to register
            let key_string: String = keys.iter().map(KeyEvent::key_sequence_format).collect();

            let _ = self.editor.registers.write(reg, vec![key_string]);

            log::info!(
                "Macro recording stopped, saved {} keys to register '{reg}'",
                keys.len()
            );
        } else {
            // Start recording: use selected register or default '@'
            let reg = self.editor.selected_register.take().unwrap_or('@');
            self.editor.macro_recording = Some((reg, Vec::new()));
            log::info!("Macro recording started to register '{reg}'");
        }
    }

    fn maybe_record_key(&mut self, key: &KeyEvent) {
        // Don't record keys while replaying a macro (prevents recursion)
        if !self.editor.macro_replaying.is_empty() {
            return;
        }

        if let Some((_, ref mut keys)) = self.editor.macro_recording {
            keys.push(*key);
        }
    }

    fn replay_macro(&mut self) {
        let reg = self.editor.selected_register.take().unwrap_or('@');

        // Prevent recursive replay
        if self.editor.macro_replaying.contains(&reg) {
            log::warn!("Macro recursion prevented for register '{reg}'");
            return;
        }

        // Read the register content
        let keys_str = {
            let content = self
                .editor
                .registers
                .read(reg, &self.editor)
                .and_then(|mut vals| vals.next().map(|val| val.to_string()));
            match content {
                Some(text) if !text.is_empty() => text,
                _ => {
                    log::info!("Register '{reg}' is empty, nothing to replay");
                    return;
                }
            }
        };

        // Parse the macro string into key events
        let keys = match helix_view::input::parse_macro(&keys_str) {
            Ok(keys) => keys,
            Err(err) => {
                log::error!("Failed to parse macro from register '{reg}': {err}");
                return;
            }
        };

        log::info!("Replaying {} keys from register '{reg}'", keys.len());

        // Mark as replaying to prevent recording and recursion
        self.editor.macro_replaying.push(reg);

        // Replay each key through dispatch
        for key in &keys {
            self.replay_key(key);
        }

        // Done replaying
        self.editor.macro_replaying.pop();
    }
}

impl EditorContext {
    /// Replay a single key event by dispatching it through the appropriate mode handler.
    fn replay_key(&mut self, key: &KeyEvent) {
        use crate::keybindings::{
            handle_command_mode, handle_insert_mode, handle_normal_mode, handle_regex_mode,
            handle_search_mode, handle_select_mode, handle_shell_mode,
        };
        use crate::state::EditorCommand;
        use helix_view::document::Mode;
        use helix_view::input::KeyCode;

        // Determine commands based on current mode and UI state
        let commands = if self.command_mode {
            handle_command_mode(key)
        } else if self.search_mode {
            handle_search_mode(key)
        } else if self.regex_mode {
            handle_regex_mode(key)
        } else if self.shell_mode {
            handle_shell_mode(key)
        } else {
            match self.editor.mode() {
                Mode::Insert => {
                    // Handle C-r in insert mode
                    if key
                        .modifiers
                        .contains(helix_view::input::KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('r')
                    {
                        // Skip C-r prefix — in replay we can't do multi-key sequences
                        // for register insertion
                        vec![]
                    } else {
                        handle_insert_mode(key)
                    }
                }
                Mode::Select => handle_select_mode(key),
                Mode::Normal => handle_normal_mode(key),
            }
        };

        for cmd in commands {
            // Skip macro commands during replay to prevent infinite recursion
            if matches!(
                cmd,
                EditorCommand::ToggleMacroRecording | EditorCommand::ReplayMacro
            ) {
                continue;
            }
            self.handle_command(cmd);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{assert_state, doc_view, key, test_context};
    use helix_view::input::{KeyCode, KeyModifiers};

    #[test]
    fn toggle_starts_recording_default_register() {
        let mut ctx = test_context("#[|h]#ello\n");
        assert!(ctx.editor.macro_recording.is_none());

        ctx.toggle_macro_recording();
        assert!(ctx.editor.macro_recording.is_some());
        let (reg, keys) = ctx.editor.macro_recording.as_ref().expect("recording");
        assert_eq!(*reg, '@');
        assert!(keys.is_empty());
    }

    #[test]
    fn toggle_starts_recording_selected_register() {
        let mut ctx = test_context("#[|h]#ello\n");
        ctx.editor.selected_register = Some('a');

        ctx.toggle_macro_recording();
        let (reg, _) = ctx.editor.macro_recording.as_ref().expect("recording");
        assert_eq!(*reg, 'a');
        assert!(ctx.editor.selected_register.is_none());
    }

    #[test]
    fn toggle_stops_recording_and_saves() {
        let mut ctx = test_context("#[|h]#ello\n");
        ctx.toggle_macro_recording(); // start

        // Record some keys
        let key_l = KeyEvent {
            code: KeyCode::Char('l'),
            modifiers: KeyModifiers::NONE,
        };
        ctx.maybe_record_key(&key_l);
        ctx.maybe_record_key(&key_l);

        ctx.toggle_macro_recording(); // stop

        assert!(ctx.editor.macro_recording.is_none());

        // Verify register content
        let content: String = ctx
            .editor
            .registers
            .read('@', &ctx.editor)
            .expect("register should exist")
            .map(|c| c.to_string())
            .collect();
        assert_eq!(content, "ll");
    }

    #[test]
    fn maybe_record_key_records_when_recording() {
        let mut ctx = test_context("#[|h]#ello\n");
        ctx.toggle_macro_recording();

        let key_j = KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
        };
        ctx.maybe_record_key(&key_j);

        let (_, keys) = ctx.editor.macro_recording.as_ref().expect("recording");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].code, KeyCode::Char('j'));
    }

    #[test]
    fn maybe_record_key_skips_when_not_recording() {
        let mut ctx = test_context("#[|h]#ello\n");
        let key_j = key('j');
        ctx.maybe_record_key(&key_j);
        // No panic, just a no-op
        assert!(ctx.editor.macro_recording.is_none());
    }

    #[test]
    fn maybe_record_key_skips_during_replay() {
        let mut ctx = test_context("#[|h]#ello\n");
        ctx.editor.macro_replaying.push('@');
        ctx.editor.macro_recording = Some(('b', Vec::new()));

        let key_j = key('j');
        ctx.maybe_record_key(&key_j);

        let (_, keys) = ctx.editor.macro_recording.as_ref().expect("recording");
        assert!(keys.is_empty(), "should not record during replay");
    }

    #[test]
    fn replay_empty_register_is_noop() {
        let mut ctx = test_context("#[|h]#ello\n");
        ctx.replay_macro();
        // Should not panic, just do nothing — state unchanged
        assert_state(&ctx, "#[|h]#ello\n");
    }

    #[test]
    fn replay_movement_macro() {
        let mut ctx = test_context("#[|h]#ello\nworld\n");
        let (doc_id, view_id) = doc_view(&ctx);

        // Manually write a macro to register '@': move right twice
        ctx.editor
            .registers
            .write('@', vec!["ll".to_string()])
            .expect("write should succeed");

        ctx.replay_macro();

        // Cursor should have moved right twice
        let doc = ctx.editor.document(doc_id).expect("doc");
        let sel = doc.selection(view_id);
        let pos = sel.primary().cursor(doc.text().slice(..));
        assert_eq!(pos, 2, "cursor should be at position 2 after 'll'");
    }

    #[test]
    fn replay_prevents_recursion() {
        let mut ctx = test_context("#[|h]#ello\n");

        // Write a macro that tries to replay itself
        ctx.editor
            .registers
            .write('@', vec!["q".to_string()])
            .expect("write");

        // Should not infinitely recurse — the q inside replay is skipped
        ctx.replay_macro();
    }
}
