//! Integration tests for key operations.
//!
//! These tests dispatch key sequences through the keymap-based dispatch
//! and verify the resulting editor state, simulating real user input.

use helix_view::document::Mode;
use helix_view::input::KeyCode;

use crate::config::DialogSearchMode;
use crate::keybindings::{handle_picker_mode, handle_search_mode};
use crate::keymap::DhxKeymapResult;
use crate::operations::{CliOps, EditingOps, MovementOps, SearchOps};
use crate::state::{EditorCommand, EditorContext, PickerMode};
use crate::test_helpers::{assert_state, assert_text, doc_view, key, special_key, test_context};

/// Dispatch a key through the keymap and return matched commands.
/// Returns empty vec for Pending/AwaitChar/NotFound/Cancelled results.
fn keymap_dispatch(ctx: &mut EditorContext, mode: Mode, key: &helix_view::input::KeyEvent) -> Vec<EditorCommand> {
    match ctx.keymaps.get(mode, *key) {
        DhxKeymapResult::Matched(cmds) => cmds,
        _ => vec![],
    }
}

/// Dispatch a sequence of editor commands on the context.
fn dispatch_commands(ctx: &mut crate::state::EditorContext, commands: &[EditorCommand]) {
    let (doc_id, view_id) = doc_view(ctx);
    for cmd in commands {
        match cmd {
            EditorCommand::MoveLeft => ctx.move_cursor(doc_id, view_id, crate::state::Direction::Left),
            EditorCommand::MoveRight => ctx.move_cursor(doc_id, view_id, crate::state::Direction::Right),
            EditorCommand::MoveUp => ctx.move_cursor(doc_id, view_id, crate::state::Direction::Up),
            EditorCommand::MoveDown => ctx.move_cursor(doc_id, view_id, crate::state::Direction::Down),
            EditorCommand::MoveWordForward => ctx.move_word_forward(doc_id, view_id),
            EditorCommand::MoveWordBackward => ctx.move_word_backward(doc_id, view_id),
            EditorCommand::GotoFirstLine => ctx.goto_first_line(doc_id, view_id),
            EditorCommand::GotoLastLine => ctx.goto_last_line(doc_id, view_id),
            EditorCommand::InsertChar(ch) => ctx.insert_char(doc_id, view_id, *ch),
            EditorCommand::DeleteCharBackward => ctx.delete_char_backward(doc_id, view_id),
            EditorCommand::EnterInsertMode => ctx.editor.mode = helix_view::document::Mode::Insert,
            EditorCommand::ExitInsertMode => ctx.editor.mode = helix_view::document::Mode::Normal,
            EditorCommand::EnterSelectMode => ctx.editor.mode = helix_view::document::Mode::Select,
            EditorCommand::ExitSelectMode => ctx.editor.mode = helix_view::document::Mode::Normal,
            EditorCommand::EnterSearchMode { backwards } => {
                ctx.search_mode = true;
                ctx.search_backwards = *backwards;
                ctx.search_input.clear();
            }
            EditorCommand::ExitSearchMode => {
                ctx.search_mode = false;
            }
            EditorCommand::SearchInput(ch) => {
                ctx.search_input.push(*ch);
            }
            EditorCommand::SearchBackspace => {
                ctx.search_input.pop();
            }
            EditorCommand::SearchExecute => {
                ctx.execute_search(doc_id, view_id);
            }
            EditorCommand::EnterCommandMode => {
                ctx.command_mode = true;
                ctx.command_input.clear();
            }
            EditorCommand::ExitCommandMode => {
                ctx.command_mode = false;
            }
            EditorCommand::CommandInput(ch) => {
                ctx.command_input.push(*ch);
            }
            _ => {
                // Skip commands not relevant to these tests
            }
        }
    }
}

// --- Normal mode movement ---

#[test]
fn normal_mode_hjkl_movement() {
    let mut ctx = test_context("#[h|]#ello\nworld\n");

    // l → move right
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('l'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "h#[e|]#llo\nworld\n");

    // j → move down
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('j'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "hello\nw#[o|]#rld\n");

    // h → move left
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('h'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "hello\n#[w|]#orld\n");

    // k → move up
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('k'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "#[h|]#ello\nworld\n");
}

#[test]
fn normal_mode_arrow_movement() {
    let mut ctx = test_context("#[h|]#ello\nworld\n");

    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &special_key(KeyCode::Right));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "h#[e|]#llo\nworld\n");

    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &special_key(KeyCode::Down));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "hello\nw#[o|]#rld\n");
}

// --- Goto last line ---

#[test]
fn goto_last_line() {
    let mut ctx = test_context("#[h|]#ello\nworld\nfoo\n");

    // G → goto last line
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('G'));
    dispatch_commands(&mut ctx, &cmds);
    let (_view, doc) = helix_view::current_ref!(ctx.editor);
    let text = doc.text().slice(..);
    let cursor_line = text.char_to_line(doc.selection(_view.id).primary().cursor(text));
    let last_line = text.len_lines().saturating_sub(1);
    assert_eq!(cursor_line, last_line, "G should go to last line");
}

// --- Find character ---

#[test]
fn find_char_returns_await_char() {
    let mut ctx = test_context("#[h|]#ello\n");
    // f alone should return AwaitChar (two-key sequence handled by keymap)
    let result = ctx.keymaps.get(Mode::Normal, *&key('f'));
    assert!(
        matches!(result, DhxKeymapResult::AwaitChar(_)),
        "f should return AwaitChar, got: {result:?}"
    );
}

// --- Insert mode and escape ---

#[test]
fn insert_mode_and_escape() {
    let mut ctx = test_context("#[h|]#ello\n");

    // i → enter insert mode
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('i'));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, Mode::Insert);

    // Type 'x'
    let cmds = vec![EditorCommand::InsertChar('x')];
    dispatch_commands(&mut ctx, &cmds);
    assert_text(&ctx, "xhello\n");

    // Esc → exit insert mode
    let cmds = keymap_dispatch(&mut ctx, Mode::Insert, &special_key(KeyCode::Esc));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, Mode::Normal);
}

// --- Search mode ---

#[test]
fn search_forward_dispatch() {
    let mut ctx = test_context("#[h|]#ello world hello\n");

    // / → enter search mode
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('/'));
    dispatch_commands(&mut ctx, &cmds);
    assert!(ctx.search_mode);
    assert!(!ctx.search_backwards);

    // Type "world"
    for ch in "world".chars() {
        let cmds = handle_search_mode(&key(ch));
        dispatch_commands(&mut ctx, &cmds);
    }
    assert_eq!(ctx.search_input, "world");

    // Enter → execute search
    let cmds = handle_search_mode(&special_key(KeyCode::Enter));
    dispatch_commands(&mut ctx, &cmds);

    // Verify selection covers "world"
    let (_view, doc) = helix_view::current_ref!(ctx.editor);
    let text = doc.text().slice(..);
    let range = doc.selection(_view.id).primary();
    let selected: String = text.slice(range.from()..range.to()).into();
    assert_eq!(selected, "world");
}

// --- Command mode ---

#[test]
fn command_mode_entry() {
    let mut ctx = test_context("#[h|]#ello\n");

    // : → enter command mode
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key(':'));
    dispatch_commands(&mut ctx, &cmds);
    assert!(ctx.command_mode);
    assert!(ctx.command_input.is_empty());
}

// --- Select mode ---

#[test]
fn select_mode_entry_and_exit() {
    let mut ctx = test_context("#[h|]#ello\n");

    // v → enter select mode
    let cmds = keymap_dispatch(&mut ctx, Mode::Normal, &key('v'));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, Mode::Select);

    // Esc → exit select mode
    let cmds = keymap_dispatch(&mut ctx, Mode::Select, &special_key(KeyCode::Esc));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, Mode::Normal);
}

// --- Picker mode (direct vs vim-style) ---

#[test]
fn picker_direct_mode_typing_filters() {
    let cmds = handle_picker_mode(&key('a'), DialogSearchMode::Direct, false, PickerMode::default());
    crate::assert_single_command!(cmds, EditorCommand::PickerInput('a'));
}

#[test]
fn picker_vim_mode_jk_navigates() {
    let cmds = handle_picker_mode(&key('j'), DialogSearchMode::VimStyle, false, PickerMode::default());
    crate::assert_single_command!(cmds, EditorCommand::PickerDown);

    let cmds = handle_picker_mode(&key('k'), DialogSearchMode::VimStyle, false, PickerMode::default());
    crate::assert_single_command!(cmds, EditorCommand::PickerUp);
}

#[test]
fn picker_vim_mode_slash_focuses_search() {
    let cmds = handle_picker_mode(&key('/'), DialogSearchMode::VimStyle, false, PickerMode::default());
    crate::assert_single_command!(cmds, EditorCommand::PickerFocusSearch);
}

#[test]
fn picker_vim_mode_typing_filters_when_focused() {
    let cmds = handle_picker_mode(&key('a'), DialogSearchMode::VimStyle, true, PickerMode::default());
    crate::assert_single_command!(cmds, EditorCommand::PickerInput('a'));
}

#[test]
fn picker_vim_mode_esc_unfocuses_search() {
    let cmds = handle_picker_mode(
        &special_key(KeyCode::Esc),
        DialogSearchMode::VimStyle,
        true,
        PickerMode::default(),
    );
    crate::assert_single_command!(cmds, EditorCommand::PickerUnfocusSearch);
}

#[test]
fn picker_vim_mode_esc_cancels_when_unfocused() {
    let cmds = handle_picker_mode(
        &special_key(KeyCode::Esc),
        DialogSearchMode::VimStyle,
        false,
        PickerMode::default(),
    );
    crate::assert_single_command!(cmds, EditorCommand::PickerCancel);
}

// --- Config reload tests ---

#[test]
fn config_reload_shows_notification() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");
    assert!(ctx.notifications.is_empty());

    ctx.reload_config();

    assert_eq!(ctx.notifications.len(), 1);
    assert_eq!(ctx.notifications[0].message, "Config reloaded");
}

#[test]
fn config_reload_via_command_mode() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");
    ctx.command_input = "config-reload".to_string();
    ctx.execute_command();

    assert!(!ctx.notifications.is_empty());
    assert_eq!(ctx.notifications[0].message, "Config reloaded");
    // Command mode should be cleared
    assert!(!ctx.command_mode);
    assert!(ctx.command_input.is_empty());
}

// --- :set command tests ---

#[test]
fn set_option_boolean() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");
    // cursorline defaults to false
    assert!(!ctx.editor.config().cursorline);

    ctx.set_option("cursorline", "true");

    assert!(ctx.editor.config().cursorline);
    assert_eq!(
        ctx.notifications.last().expect("notification").message,
        "Set cursorline = true"
    );
}

#[test]
fn set_option_number() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");
    assert_eq!(ctx.editor.config().scrolloff, 5);

    ctx.set_option("scrolloff", "10");

    assert_eq!(ctx.editor.config().scrolloff, 10);
}

#[test]
fn set_option_unknown_key() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    ctx.set_option("nonexistent-key", "true");

    let msg = &ctx.notifications.last().expect("notification").message;
    assert!(msg.contains("Unknown config key"), "got: {msg}");
}

#[test]
fn set_option_via_command_mode() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    ctx.command_input = "set cursorline true".to_string();
    ctx.execute_command();

    assert!(ctx.editor.config().cursorline);
}

#[test]
fn toggle_option_boolean() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");
    assert!(!ctx.editor.config().cursorline);

    ctx.toggle_option("cursorline");
    assert!(ctx.editor.config().cursorline);

    ctx.toggle_option("cursorline");
    assert!(!ctx.editor.config().cursorline);
}

#[test]
fn toggle_option_via_command_mode() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    ctx.command_input = "toggle cursorline".to_string();
    ctx.execute_command();

    assert!(ctx.editor.config().cursorline);
}

// --- Dot-repeat (RepeatLastInsert) ---

#[test]
fn dot_repeat_insert_chars() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    // Enter insert, type "ab", exit
    ctx.handle_command(EditorCommand::EnterInsertMode);
    ctx.handle_command(EditorCommand::InsertChar('a'));
    ctx.handle_command(EditorCommand::InsertChar('b'));
    ctx.handle_command(EditorCommand::ExitInsertMode);

    // Move right past 'h'
    ctx.handle_command(EditorCommand::MoveRight);

    // Dot repeat should insert "ab" again
    ctx.handle_command(EditorCommand::RepeatLastInsert);

    assert_text(&ctx, "abhabello\n");
}

#[test]
fn dot_repeat_preserves_recording_after_replay() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    // Enter insert, type "x", exit
    ctx.handle_command(EditorCommand::EnterInsertMode);
    ctx.handle_command(EditorCommand::InsertChar('x'));
    ctx.handle_command(EditorCommand::ExitInsertMode);

    // Replay
    ctx.handle_command(EditorCommand::RepeatLastInsert);

    // Recording should still be 1 command (InsertChar('x'))
    assert_eq!(ctx.last_insert_keys.len(), 1);
}

#[test]
fn dot_repeat_open_line_below() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    // OpenLineBelow enters insert mode + opens new line
    ctx.handle_command(EditorCommand::OpenLineBelow);
    ctx.handle_command(EditorCommand::InsertChar('x'));
    ctx.handle_command(EditorCommand::ExitInsertMode);

    // Go back to first line
    ctx.handle_command(EditorCommand::GotoFirstLine);

    // Dot repeat should open below + insert 'x'
    ctx.handle_command(EditorCommand::RepeatLastInsert);

    assert_text(&ctx, "hello\nx\nx\n");
}

#[test]
fn dot_repeat_with_backspace() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    // Enter insert, type "abc", backspace (delete 'c'), exit
    ctx.handle_command(EditorCommand::EnterInsertMode);
    ctx.handle_command(EditorCommand::InsertChar('a'));
    ctx.handle_command(EditorCommand::InsertChar('b'));
    ctx.handle_command(EditorCommand::InsertChar('c'));
    ctx.handle_command(EditorCommand::DeleteCharBackward);
    ctx.handle_command(EditorCommand::ExitInsertMode);

    // Verify first insert result
    assert_text(&ctx, "abhello\n");

    // Move right past 'h'
    ctx.handle_command(EditorCommand::MoveRight);

    // Dot repeat should replay: insert 'a', 'b', 'c', backspace → "ab"
    ctx.handle_command(EditorCommand::RepeatLastInsert);

    assert_text(&ctx, "abhabello\n");
}

#[test]
fn exit_insert_mode_dismisses_signature_help() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    // Simulate signature help being visible (as if LSP responded)
    ctx.signature_help_visible = true;
    ctx.signature_help = Some(crate::lsp::SignatureHelpSnapshot::default());

    // Exit insert mode should dismiss signature help
    ctx.handle_command(EditorCommand::ExitInsertMode);

    assert!(!ctx.signature_help_visible);
    assert!(ctx.signature_help.is_none());
}

#[test]
fn dot_repeat_does_not_record_during_replay() {
    let _guard = crate::test_helpers::init();
    let mut ctx = test_context("#[|h]#ello\n");

    // Enter insert, type "y", exit
    ctx.handle_command(EditorCommand::EnterInsertMode);
    ctx.handle_command(EditorCommand::InsertChar('y'));
    ctx.handle_command(EditorCommand::ExitInsertMode);

    // First dot repeat
    ctx.handle_command(EditorCommand::RepeatLastInsert);

    // Second dot repeat should still insert just "y", not "yy"
    ctx.handle_command(EditorCommand::RepeatLastInsert);

    assert_text(&ctx, "yyyhello\n");
    assert_eq!(ctx.last_insert_keys.len(), 1);
}

/// Simulate Alt+Down move-line-down: extend_to_line_bounds, delete_selection, paste_after.
/// This is the full dispatch path through handle_command (same as the real app).
#[test]
fn move_line_down_via_handle_command() {
    let mut ctx = test_context("fn main() {\n    #[p|]#rintln!(\"Hello\");\n    let x = 1;\n}\n");

    // Dispatch the 3-command sequence (as the user config would)
    ctx.handle_command(EditorCommand::ExtendToLineBounds);
    ctx.handle_command(EditorCommand::DeleteSelection);
    ctx.handle_command(EditorCommand::Paste);

    // println should move below "let x = 1;"
    assert_text(&ctx, "fn main() {\n    let x = 1;\n    println!(\"Hello\");\n}\n");
}

#[test]
fn move_line_down_test_error_rs_content() {
    // Exact content of examples/test_error.rs — cursor on println line
    let mut ctx = test_context(
        "// Test file with intentional error for testing LSP diagnostics.\n\
         // DO NOT DELETE - see CLAUDE.md for details.\n\
         \n\
         fn main() {\n\
         \n\
         \u{0020}   #[p|]#rintln!(\"Hello\");\n\
         \u{0020}   let x: String = 1;\n\
         }\n",
    );

    let before_len = {
        let (_, doc) = helix_view::current_ref!(ctx.editor);
        doc.text().len_lines()
    };

    ctx.handle_command(EditorCommand::ExtendToLineBounds);
    ctx.handle_command(EditorCommand::DeleteSelection);
    ctx.handle_command(EditorCommand::Paste);

    let after_len = {
        let (_, doc) = helix_view::current_ref!(ctx.editor);
        doc.text().len_lines()
    };

    // Line count must not change (no extra empty line)
    assert_eq!(
        before_len, after_len,
        "line count changed from {before_len} to {after_len}"
    );

    assert_text(
        &ctx,
        "// Test file with intentional error for testing LSP diagnostics.\n\
         // DO NOT DELETE - see CLAUDE.md for details.\n\
         \n\
         fn main() {\n\
         \n\
         \u{0020}   let x: String = 1;\n\
         \u{0020}   println!(\"Hello\");\n\
         }\n",
    );
}

/// Join lines should place cursor at the join space (matching helix-term select_space=true).
#[test]
fn join_lines_cursor_at_join_space() {
    let mut ctx = test_context("#[h|]#ello\n    world\n");

    ctx.handle_command(EditorCommand::JoinLines);

    // After join, cursor should be at the space between "hello" and "world"
    assert_state(&ctx, "hello#[ |]#world\n");
}

/// Join lines with multi-line selection.
#[test]
fn join_lines_multi_line_selection() {
    let mut ctx = test_context("#[line1\n    line2\n    lin|]#e3\n");

    ctx.handle_command(EditorCommand::JoinLines);

    // Both joins produce spaces; cursor should be at the first join space
    assert_text(&ctx, "line1 line2 line3\n");
}

/// Join lines on single line joins with next line.
#[test]
fn join_lines_single_line() {
    let mut ctx = test_context("foo\n  #[b|]#ar\nbaz\n");

    ctx.handle_command(EditorCommand::JoinLines);

    assert_state(&ctx, "foo\n  bar#[ |]#baz\n");
}
