//! Integration tests for key operations.
//!
//! These tests dispatch key sequences through keybinding handlers
//! and verify the resulting editor state, simulating real user input.

use helix_view::input::KeyCode;

use crate::config::DialogSearchMode;
use crate::keybindings::{
    handle_normal_mode, handle_picker_mode, handle_search_mode, handle_select_mode,
};
use crate::operations::{CliOps, EditingOps, MovementOps, SearchOps};
use crate::state::{EditorCommand, PickerMode};
use crate::test_helpers::{assert_state, assert_text, doc_view, key, special_key, test_context};

/// Dispatch a sequence of editor commands on the context.
fn dispatch_commands(ctx: &mut crate::state::EditorContext, commands: &[EditorCommand]) {
    let (doc_id, view_id) = doc_view(ctx);
    for cmd in commands {
        match cmd {
            EditorCommand::MoveLeft => {
                ctx.move_cursor(doc_id, view_id, crate::state::Direction::Left)
            }
            EditorCommand::MoveRight => {
                ctx.move_cursor(doc_id, view_id, crate::state::Direction::Right)
            }
            EditorCommand::MoveUp => ctx.move_cursor(doc_id, view_id, crate::state::Direction::Up),
            EditorCommand::MoveDown => {
                ctx.move_cursor(doc_id, view_id, crate::state::Direction::Down)
            }
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
    let cmds = handle_normal_mode(&key('l'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "h#[e|]#llo\nworld\n");

    // j → move down
    let cmds = handle_normal_mode(&key('j'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "hello\nw#[o|]#rld\n");

    // h → move left
    let cmds = handle_normal_mode(&key('h'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "hello\n#[w|]#orld\n");

    // k → move up
    let cmds = handle_normal_mode(&key('k'));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "#[h|]#ello\nworld\n");
}

#[test]
fn normal_mode_arrow_movement() {
    let mut ctx = test_context("#[h|]#ello\nworld\n");

    let cmds = handle_normal_mode(&special_key(KeyCode::Right));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "h#[e|]#llo\nworld\n");

    let cmds = handle_normal_mode(&special_key(KeyCode::Down));
    dispatch_commands(&mut ctx, &cmds);
    assert_state(&ctx, "hello\nw#[o|]#rld\n");
}

// --- Goto last line ---

#[test]
fn goto_last_line() {
    let mut ctx = test_context("#[h|]#ello\nworld\nfoo\n");

    // G → goto last line
    let cmds = handle_normal_mode(&key('G'));
    dispatch_commands(&mut ctx, &cmds);
    let (_view, doc) = helix_view::current_ref!(ctx.editor);
    let text = doc.text().slice(..);
    let cursor_line = text.char_to_line(doc.selection(_view.id).primary().cursor(text));
    let last_line = text.len_lines().saturating_sub(1);
    assert_eq!(cursor_line, last_line, "G should go to last line");
}

// --- Find character ---

#[test]
fn find_char_returns_pending() {
    // f alone should produce no commands (two-key sequence handled by app.rs)
    let cmds = handle_normal_mode(&key('f'));
    assert!(cmds.is_empty(), "f should produce no commands (pending)");
}

// --- Insert mode and escape ---

#[test]
fn insert_mode_and_escape() {
    let mut ctx = test_context("#[h|]#ello\n");

    // i → enter insert mode
    let cmds = handle_normal_mode(&key('i'));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, helix_view::document::Mode::Insert);

    // Type 'x'
    let cmds = vec![EditorCommand::InsertChar('x')];
    dispatch_commands(&mut ctx, &cmds);
    assert_text(&ctx, "xhello\n");

    // Esc → exit insert mode
    let cmds = crate::keybindings::handle_insert_mode(&special_key(KeyCode::Esc));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, helix_view::document::Mode::Normal);
}

// --- Search mode ---

#[test]
fn search_forward_dispatch() {
    let mut ctx = test_context("#[h|]#ello world hello\n");

    // / → enter search mode
    let cmds = handle_normal_mode(&key('/'));
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
    let cmds = handle_normal_mode(&key(':'));
    dispatch_commands(&mut ctx, &cmds);
    assert!(ctx.command_mode);
    assert!(ctx.command_input.is_empty());
}

// --- Select mode ---

#[test]
fn select_mode_entry_and_exit() {
    let mut ctx = test_context("#[h|]#ello\n");

    // v → enter select mode
    let cmds = handle_normal_mode(&key('v'));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, helix_view::document::Mode::Select);

    // Esc → exit select mode
    let cmds = handle_select_mode(&special_key(KeyCode::Esc));
    dispatch_commands(&mut ctx, &cmds);
    assert_eq!(ctx.editor.mode, helix_view::document::Mode::Normal);
}

// --- Picker mode (direct vs vim-style) ---

#[test]
fn picker_direct_mode_typing_filters() {
    let cmds = handle_picker_mode(
        &key('a'),
        DialogSearchMode::Direct,
        false,
        PickerMode::default(),
    );
    crate::assert_single_command!(cmds, EditorCommand::PickerInput('a'));
}

#[test]
fn picker_vim_mode_jk_navigates() {
    let cmds = handle_picker_mode(
        &key('j'),
        DialogSearchMode::VimStyle,
        false,
        PickerMode::default(),
    );
    crate::assert_single_command!(cmds, EditorCommand::PickerDown);

    let cmds = handle_picker_mode(
        &key('k'),
        DialogSearchMode::VimStyle,
        false,
        PickerMode::default(),
    );
    crate::assert_single_command!(cmds, EditorCommand::PickerUp);
}

#[test]
fn picker_vim_mode_slash_focuses_search() {
    let cmds = handle_picker_mode(
        &key('/'),
        DialogSearchMode::VimStyle,
        false,
        PickerMode::default(),
    );
    crate::assert_single_command!(cmds, EditorCommand::PickerFocusSearch);
}

#[test]
fn picker_vim_mode_typing_filters_when_focused() {
    let cmds = handle_picker_mode(
        &key('a'),
        DialogSearchMode::VimStyle,
        true,
        PickerMode::default(),
    );
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
