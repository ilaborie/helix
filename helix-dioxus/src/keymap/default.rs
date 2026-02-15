//! Default keymaps for helix-dioxus.
//!
//! Encodes all current hardcoded bindings from `keybindings/normal.rs`,
//! `keybindings/select.rs`, `keybindings/insert.rs`, and the prefix handlers
//! in `keybindings/mod.rs` + `app.rs` as trie structures.

use std::collections::HashMap;

use helix_view::document::Mode;
use helix_view::input::KeyCode;

use super::command::AwaitCharKind;
use super::trie::{alt, ctrl, ctrl_super, key, shift_tab, special, DhxKeyTrie, DhxKeyTrieNode};
use crate::state::{EditorCommand, ShellBehavior};

/// Build the complete default keymaps for all modes.
#[must_use]
pub fn default_keymaps() -> HashMap<Mode, DhxKeyTrie> {
    let mut map = HashMap::new();
    map.insert(Mode::Normal, DhxKeyTrie::node(normal_mode_defaults()));
    map.insert(Mode::Select, DhxKeyTrie::node(select_mode_defaults()));
    map.insert(Mode::Insert, DhxKeyTrie::node(insert_mode_defaults()));
    map
}

/// Default normal mode keybindings.
#[must_use]
pub fn normal_mode_defaults() -> DhxKeyTrieNode {
    let mut root = DhxKeyTrieNode::new("normal");

    // --- Direction keys (hjkl + arrows) ---
    root.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::MoveLeft));
    root.insert(key('l'), DhxKeyTrie::cmd(EditorCommand::MoveRight));
    root.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::MoveDown));
    root.insert(key('k'), DhxKeyTrie::cmd(EditorCommand::MoveUp));
    root.insert(special(KeyCode::Left), DhxKeyTrie::cmd(EditorCommand::MoveLeft));
    root.insert(special(KeyCode::Right), DhxKeyTrie::cmd(EditorCommand::MoveRight));
    root.insert(special(KeyCode::Down), DhxKeyTrie::cmd(EditorCommand::MoveDown));
    root.insert(special(KeyCode::Up), DhxKeyTrie::cmd(EditorCommand::MoveUp));

    // --- Word movement ---
    root.insert(key('w'), DhxKeyTrie::cmd(EditorCommand::MoveWordForward));
    root.insert(key('b'), DhxKeyTrie::cmd(EditorCommand::MoveWordBackward));
    root.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::MoveWordEnd));

    // --- WORD movement (long words) ---
    root.insert(key('W'), DhxKeyTrie::cmd(EditorCommand::MoveLongWordForward));
    root.insert(key('B'), DhxKeyTrie::cmd(EditorCommand::MoveLongWordBackward));
    root.insert(key('E'), DhxKeyTrie::cmd(EditorCommand::MoveLongWordEnd));

    // --- Line movement ---
    root.insert(key('0'), DhxKeyTrie::cmd(EditorCommand::MoveLineStart));
    root.insert(special(KeyCode::Home), DhxKeyTrie::cmd(EditorCommand::MoveLineStart));
    root.insert(key('$'), DhxKeyTrie::cmd(EditorCommand::MoveLineEnd));
    root.insert(special(KeyCode::End), DhxKeyTrie::cmd(EditorCommand::MoveLineEnd));

    // --- File/page navigation ---
    root.insert(key('G'), DhxKeyTrie::cmd(EditorCommand::GotoLastLine));
    root.insert(special(KeyCode::PageUp), DhxKeyTrie::cmd(EditorCommand::PageUp));
    root.insert(special(KeyCode::PageDown), DhxKeyTrie::cmd(EditorCommand::PageDown));

    // --- Mode changes ---
    root.insert(key('i'), DhxKeyTrie::cmd(EditorCommand::EnterInsertMode));
    root.insert(key('I'), DhxKeyTrie::cmd(EditorCommand::EnterInsertModeLineStart));
    root.insert(key('a'), DhxKeyTrie::cmd(EditorCommand::EnterInsertModeAfter));
    root.insert(key('A'), DhxKeyTrie::cmd(EditorCommand::EnterInsertModeLineEnd));
    root.insert(key('o'), DhxKeyTrie::cmd(EditorCommand::OpenLineBelow));
    root.insert(key('O'), DhxKeyTrie::cmd(EditorCommand::OpenLineAbove));
    root.insert(key('v'), DhxKeyTrie::cmd(EditorCommand::EnterSelectMode));

    // --- Editing ---
    root.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::ChangeSelection));
    root.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::DeleteSelection));
    root.insert(key('R'), DhxKeyTrie::cmd(EditorCommand::ReplaceWithYanked));
    root.insert(key('J'), DhxKeyTrie::cmd(EditorCommand::JoinLines));
    root.insert(key('&'), DhxKeyTrie::cmd(EditorCommand::AlignSelections));
    root.insert(key('_'), DhxKeyTrie::cmd(EditorCommand::TrimSelections));
    root.insert(key('='), DhxKeyTrie::cmd(EditorCommand::FormatSelections));

    // --- Regex select/split ---
    root.insert(key('s'), DhxKeyTrie::cmd(EditorCommand::EnterRegexMode { split: false }));
    root.insert(key('S'), DhxKeyTrie::cmd(EditorCommand::EnterRegexMode { split: true }));

    // --- Copy selection on next line ---
    root.insert(key('C'), DhxKeyTrie::cmd(EditorCommand::CopySelectionOnNextLine));

    // --- Rotate selections ---
    root.insert(key(')'), DhxKeyTrie::cmd(EditorCommand::RotateSelectionsForward));
    root.insert(key('('), DhxKeyTrie::cmd(EditorCommand::RotateSelectionsBackward));

    // --- History ---
    root.insert(key('u'), DhxKeyTrie::cmd(EditorCommand::Undo));
    root.insert(key('U'), DhxKeyTrie::cmd(EditorCommand::Redo));

    // --- Line selection ---
    root.insert(key('x'), DhxKeyTrie::cmd(EditorCommand::SelectLine));
    root.insert(key('X'), DhxKeyTrie::cmd(EditorCommand::ExtendToLineBounds));

    // --- Clipboard ---
    root.insert(key('p'), DhxKeyTrie::cmd(EditorCommand::Paste));
    root.insert(key('P'), DhxKeyTrie::cmd(EditorCommand::PasteBefore));
    root.insert(key('y'), DhxKeyTrie::cmd(EditorCommand::Yank));

    // --- Search ---
    root.insert(key('/'), DhxKeyTrie::cmd(EditorCommand::EnterSearchMode { backwards: false }));
    root.insert(key('?'), DhxKeyTrie::cmd(EditorCommand::EnterSearchMode { backwards: true }));
    root.insert(key('n'), DhxKeyTrie::cmd(EditorCommand::SearchNext));
    root.insert(key('N'), DhxKeyTrie::cmd(EditorCommand::SearchPrevious));
    root.insert(key('*'), DhxKeyTrie::cmd(EditorCommand::SearchWordUnderCursor));

    // --- Command mode ---
    root.insert(key(':'), DhxKeyTrie::cmd(EditorCommand::EnterCommandMode));

    // --- Selection operations ---
    root.insert(key(';'), DhxKeyTrie::cmd(EditorCommand::CollapseSelection));
    root.insert(key(','), DhxKeyTrie::cmd(EditorCommand::KeepPrimarySelection));

    // --- Indent/unindent ---
    root.insert(key('>'), DhxKeyTrie::cmd(EditorCommand::IndentLine));
    root.insert(key('<'), DhxKeyTrie::cmd(EditorCommand::UnindentLine));

    // --- Select all ---
    root.insert(key('%'), DhxKeyTrie::cmd(EditorCommand::SelectAll));

    // --- Case operations ---
    root.insert(key('~'), DhxKeyTrie::cmd(EditorCommand::ToggleCase));
    root.insert(key('`'), DhxKeyTrie::cmd(EditorCommand::ToLowercase));

    // --- LSP hover ---
    root.insert(key('K'), DhxKeyTrie::cmd(EditorCommand::TriggerHover));

    // --- Shell integration ---
    root.insert(key('|'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Replace)));
    root.insert(key('!'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Insert)));

    // --- Macro ---
    root.insert(key('Q'), DhxKeyTrie::cmd(EditorCommand::ToggleMacroRecording));
    root.insert(key('q'), DhxKeyTrie::cmd(EditorCommand::ReplayMacro));

    // --- Repeat last insert ---
    root.insert(key('.'), DhxKeyTrie::cmd(EditorCommand::RepeatLastInsert));

    // --- Escape ---
    root.insert(special(KeyCode::Esc), DhxKeyTrie::cmd(EditorCommand::CollapseSelection));

    // --- Await-char commands ---
    root.insert(key('f'), DhxKeyTrie::await_char(AwaitCharKind::FindForward));
    root.insert(key('F'), DhxKeyTrie::await_char(AwaitCharKind::FindBackward));
    root.insert(key('t'), DhxKeyTrie::await_char(AwaitCharKind::TillForward));
    root.insert(key('T'), DhxKeyTrie::await_char(AwaitCharKind::TillBackward));
    root.insert(key('r'), DhxKeyTrie::await_char(AwaitCharKind::ReplaceChar));
    root.insert(key('"'), DhxKeyTrie::await_char(AwaitCharKind::SelectRegister));

    // --- Alt+key combinations ---
    root.insert(alt('.'), DhxKeyTrie::cmd(EditorCommand::RepeatLastFind));
    root.insert(alt(';'), DhxKeyTrie::cmd(EditorCommand::FlipSelections));
    root.insert(alt('`'), DhxKeyTrie::cmd(EditorCommand::ToUppercase));
    root.insert(alt('c'), DhxKeyTrie::cmd(EditorCommand::ChangeSelectionNoYank));
    root.insert(alt('C'), DhxKeyTrie::cmd(EditorCommand::CopySelectionOnPrevLine));
    root.insert(alt('d'), DhxKeyTrie::cmd(EditorCommand::DeleteSelectionNoYank));
    root.insert(alt('s'), DhxKeyTrie::cmd(EditorCommand::SplitSelectionOnNewline));
    root.insert(alt('x'), DhxKeyTrie::cmd(EditorCommand::ShrinkToLineBounds));
    root.insert(alt('o'), DhxKeyTrie::cmd(EditorCommand::ExpandSelection));
    root.insert(alt('i'), DhxKeyTrie::cmd(EditorCommand::ShrinkSelection));
    root.insert(alt('|'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Ignore)));
    root.insert(alt('!'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Append)));

    // --- Ctrl+key combinations ---
    root.insert(ctrl('b'), DhxKeyTrie::cmd(EditorCommand::PageUp));
    root.insert(ctrl('c'), DhxKeyTrie::cmd(EditorCommand::ToggleLineComment));
    root.insert(ctrl('d'), DhxKeyTrie::cmd(EditorCommand::HalfPageDown));
    root.insert(ctrl('f'), DhxKeyTrie::cmd(EditorCommand::PageDown));
    root.insert(ctrl('h'), DhxKeyTrie::cmd(EditorCommand::PreviousBuffer));
    root.insert(ctrl('l'), DhxKeyTrie::cmd(EditorCommand::NextBuffer));
    root.insert(ctrl('a'), DhxKeyTrie::cmd(EditorCommand::Increment));
    root.insert(ctrl('i'), DhxKeyTrie::cmd(EditorCommand::JumpForward));
    root.insert(ctrl('o'), DhxKeyTrie::cmd(EditorCommand::JumpBackward));
    root.insert(ctrl('r'), DhxKeyTrie::cmd(EditorCommand::Redo));
    root.insert(ctrl('s'), DhxKeyTrie::cmd(EditorCommand::SaveSelection));
    root.insert(ctrl('u'), DhxKeyTrie::cmd(EditorCommand::HalfPageUp));
    root.insert(ctrl('x'), DhxKeyTrie::cmd(EditorCommand::Decrement));
    root.insert(ctrl(' '), DhxKeyTrie::cmd(EditorCommand::ShowCodeActions));
    root.insert(ctrl('.'), DhxKeyTrie::cmd(EditorCommand::ShowCodeActions));

    // --- g prefix (goto) ---
    root.insert(key('g'), DhxKeyTrie::node(g_prefix_defaults()));

    // --- ] prefix (bracket next) ---
    root.insert(key(']'), DhxKeyTrie::node(bracket_next_defaults()));

    // --- [ prefix (bracket prev) ---
    root.insert(key('['), DhxKeyTrie::node(bracket_prev_defaults()));

    // --- Space prefix (leader) ---
    root.insert(key(' '), DhxKeyTrie::node(space_leader_defaults()));

    // --- m prefix (match) ---
    root.insert(key('m'), DhxKeyTrie::node(match_prefix_defaults()));

    // --- z prefix (view, non-sticky) ---
    root.insert(key('z'), DhxKeyTrie::node(view_prefix_defaults()));

    // --- Z prefix (view, sticky) ---
    root.insert(key('Z'), DhxKeyTrie::node(view_prefix_sticky_defaults()));

    root
}

/// Default select mode keybindings.
#[must_use]
pub fn select_mode_defaults() -> DhxKeyTrieNode {
    let mut root = DhxKeyTrieNode::new("select");

    // --- Direction keys (hjkl + arrows) → extend ---
    root.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::ExtendLeft));
    root.insert(key('l'), DhxKeyTrie::cmd(EditorCommand::ExtendRight));
    root.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::ExtendDown));
    root.insert(key('k'), DhxKeyTrie::cmd(EditorCommand::ExtendUp));
    root.insert(special(KeyCode::Left), DhxKeyTrie::cmd(EditorCommand::ExtendLeft));
    root.insert(special(KeyCode::Right), DhxKeyTrie::cmd(EditorCommand::ExtendRight));
    root.insert(special(KeyCode::Down), DhxKeyTrie::cmd(EditorCommand::ExtendDown));
    root.insert(special(KeyCode::Up), DhxKeyTrie::cmd(EditorCommand::ExtendUp));

    // --- Exit select mode ---
    root.insert(special(KeyCode::Esc), DhxKeyTrie::cmd(EditorCommand::ExitSelectMode));
    root.insert(key('v'), DhxKeyTrie::cmd(EditorCommand::ExitSelectMode));

    // --- Word movement → extend ---
    root.insert(key('w'), DhxKeyTrie::cmd(EditorCommand::ExtendWordForward));
    root.insert(key('b'), DhxKeyTrie::cmd(EditorCommand::ExtendWordBackward));
    root.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::ExtendWordEnd));

    // --- WORD movement → extend ---
    root.insert(key('W'), DhxKeyTrie::cmd(EditorCommand::ExtendLongWordForward));
    root.insert(key('B'), DhxKeyTrie::cmd(EditorCommand::ExtendLongWordBackward));
    root.insert(key('E'), DhxKeyTrie::cmd(EditorCommand::ExtendLongWordEnd));

    // --- Line movement → extend ---
    root.insert(key('0'), DhxKeyTrie::cmd(EditorCommand::ExtendLineStart));
    root.insert(special(KeyCode::Home), DhxKeyTrie::cmd(EditorCommand::ExtendLineStart));
    root.insert(key('$'), DhxKeyTrie::cmd(EditorCommand::ExtendLineEnd));
    root.insert(special(KeyCode::End), DhxKeyTrie::cmd(EditorCommand::ExtendLineEnd));

    // --- Page movement ---
    root.insert(special(KeyCode::PageUp), DhxKeyTrie::cmd(EditorCommand::PageUp));
    root.insert(special(KeyCode::PageDown), DhxKeyTrie::cmd(EditorCommand::PageDown));

    // --- Line selection ---
    root.insert(key('x'), DhxKeyTrie::cmd(EditorCommand::SelectLine));
    root.insert(key('X'), DhxKeyTrie::cmd(EditorCommand::ExtendLine));

    // --- Clipboard ---
    root.insert(key('y'), DhxKeyTrie::seq(vec![EditorCommand::Yank, EditorCommand::ExitSelectMode]));
    root.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::DeleteSelection));
    root.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::ChangeSelection));

    // --- Search extend ---
    root.insert(key('n'), DhxKeyTrie::cmd(EditorCommand::ExtendSearchNext));
    root.insert(key('N'), DhxKeyTrie::cmd(EditorCommand::ExtendSearchPrev));

    // --- Replace / paste ---
    root.insert(key('R'), DhxKeyTrie::cmd(EditorCommand::ReplaceWithYanked));
    root.insert(key('p'), DhxKeyTrie::cmd(EditorCommand::ReplaceWithYanked));

    // --- Indent/unindent ---
    root.insert(key('>'), DhxKeyTrie::cmd(EditorCommand::IndentLine));
    root.insert(key('<'), DhxKeyTrie::cmd(EditorCommand::UnindentLine));

    // --- Selection operations ---
    root.insert(key(';'), DhxKeyTrie::cmd(EditorCommand::CollapseSelection));
    root.insert(key(','), DhxKeyTrie::cmd(EditorCommand::KeepPrimarySelection));

    // --- Regex select/split ---
    root.insert(key('s'), DhxKeyTrie::cmd(EditorCommand::EnterRegexMode { split: false }));
    root.insert(key('S'), DhxKeyTrie::cmd(EditorCommand::EnterRegexMode { split: true }));

    // --- Copy selection on next line ---
    root.insert(key('C'), DhxKeyTrie::cmd(EditorCommand::CopySelectionOnNextLine));

    // --- Rotate selections ---
    root.insert(key(')'), DhxKeyTrie::cmd(EditorCommand::RotateSelectionsForward));
    root.insert(key('('), DhxKeyTrie::cmd(EditorCommand::RotateSelectionsBackward));

    // --- Shell integration ---
    root.insert(key('|'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Replace)));
    root.insert(key('!'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Insert)));

    // --- Macro ---
    root.insert(key('Q'), DhxKeyTrie::cmd(EditorCommand::ToggleMacroRecording));
    root.insert(key('q'), DhxKeyTrie::cmd(EditorCommand::ReplayMacro));

    // --- Await-char commands (mode-aware via resolve_await) ---
    root.insert(key('f'), DhxKeyTrie::await_char(AwaitCharKind::FindForward));
    root.insert(key('F'), DhxKeyTrie::await_char(AwaitCharKind::FindBackward));
    root.insert(key('t'), DhxKeyTrie::await_char(AwaitCharKind::TillForward));
    root.insert(key('T'), DhxKeyTrie::await_char(AwaitCharKind::TillBackward));
    root.insert(key('r'), DhxKeyTrie::await_char(AwaitCharKind::ReplaceChar));
    root.insert(key('"'), DhxKeyTrie::await_char(AwaitCharKind::SelectRegister));

    // --- Alt+key combinations ---
    root.insert(alt(';'), DhxKeyTrie::cmd(EditorCommand::FlipSelections));
    root.insert(alt('C'), DhxKeyTrie::cmd(EditorCommand::CopySelectionOnPrevLine));
    root.insert(alt('s'), DhxKeyTrie::cmd(EditorCommand::SplitSelectionOnNewline));
    root.insert(alt('o'), DhxKeyTrie::cmd(EditorCommand::ExpandSelection));
    root.insert(alt('i'), DhxKeyTrie::cmd(EditorCommand::ShrinkSelection));
    root.insert(alt('|'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Ignore)));
    root.insert(alt('!'), DhxKeyTrie::cmd(EditorCommand::EnterShellMode(ShellBehavior::Append)));

    // --- Ctrl+key combinations ---
    root.insert(ctrl('c'), DhxKeyTrie::cmd(EditorCommand::ToggleLineComment));
    root.insert(ctrl('i'), DhxKeyTrie::cmd(EditorCommand::JumpForward));
    root.insert(ctrl('o'), DhxKeyTrie::cmd(EditorCommand::JumpBackward));
    root.insert(ctrl('s'), DhxKeyTrie::cmd(EditorCommand::SaveSelection));

    // --- g prefix (select mode variant) ---
    root.insert(key('g'), DhxKeyTrie::node(select_g_prefix_defaults()));

    // --- ] prefix ---
    root.insert(key(']'), DhxKeyTrie::node(bracket_next_defaults()));

    // --- [ prefix ---
    root.insert(key('['), DhxKeyTrie::node(bracket_prev_defaults()));

    // --- Space prefix ---
    root.insert(key(' '), DhxKeyTrie::node(space_leader_defaults()));

    // --- m prefix (match) ---
    root.insert(key('m'), DhxKeyTrie::node(match_prefix_defaults()));

    // --- z prefix (view, non-sticky) ---
    root.insert(key('z'), DhxKeyTrie::node(view_prefix_defaults()));

    // --- Z prefix (view, sticky) ---
    root.insert(key('Z'), DhxKeyTrie::node(view_prefix_sticky_defaults()));

    root
}

/// Default insert mode keybindings.
#[must_use]
pub fn insert_mode_defaults() -> DhxKeyTrieNode {
    let mut root = DhxKeyTrieNode::new("insert");

    // --- Escape ---
    root.insert(special(KeyCode::Esc), DhxKeyTrie::cmd(EditorCommand::ExitInsertMode));

    // --- Basic editing ---
    root.insert(special(KeyCode::Tab), DhxKeyTrie::cmd(EditorCommand::InsertTab));
    root.insert(shift_tab(), DhxKeyTrie::cmd(EditorCommand::UnindentLine));
    root.insert(special(KeyCode::Enter), DhxKeyTrie::cmd(EditorCommand::InsertNewline));
    root.insert(special(KeyCode::Backspace), DhxKeyTrie::cmd(EditorCommand::DeleteCharBackward));
    root.insert(special(KeyCode::Delete), DhxKeyTrie::cmd(EditorCommand::DeleteCharForward));

    // --- Arrow keys ---
    root.insert(special(KeyCode::Left), DhxKeyTrie::cmd(EditorCommand::MoveLeft));
    root.insert(special(KeyCode::Right), DhxKeyTrie::cmd(EditorCommand::MoveRight));
    root.insert(special(KeyCode::Up), DhxKeyTrie::cmd(EditorCommand::MoveUp));
    root.insert(special(KeyCode::Down), DhxKeyTrie::cmd(EditorCommand::MoveDown));
    root.insert(special(KeyCode::Home), DhxKeyTrie::cmd(EditorCommand::MoveLineStart));
    root.insert(special(KeyCode::End), DhxKeyTrie::cmd(EditorCommand::MoveLineEnd));
    root.insert(special(KeyCode::PageUp), DhxKeyTrie::cmd(EditorCommand::PageUp));
    root.insert(special(KeyCode::PageDown), DhxKeyTrie::cmd(EditorCommand::PageDown));

    // --- Alt+key combinations ---
    root.insert(alt('d'), DhxKeyTrie::cmd(EditorCommand::DeleteWordForward));
    // Alt+Backspace → delete word backward
    // NOTE: Alt+Backspace requires a special KeyEvent constructor
    {
        let alt_bs = helix_view::input::KeyEvent {
            code: KeyCode::Backspace,
            modifiers: helix_view::keyboard::KeyModifiers::ALT,
        };
        root.insert(alt_bs, DhxKeyTrie::cmd(EditorCommand::DeleteWordBackward));
    }

    // --- Ctrl+Cmd+Space (emoji picker) ---
    root.insert(ctrl_super(' '), DhxKeyTrie::cmd(EditorCommand::ShowEmojiPicker));

    // --- Ctrl+key combinations ---
    root.insert(ctrl('c'), DhxKeyTrie::cmd(EditorCommand::ToggleLineComment));
    root.insert(ctrl('d'), DhxKeyTrie::cmd(EditorCommand::DeleteCharForward));
    root.insert(ctrl('h'), DhxKeyTrie::cmd(EditorCommand::DeleteCharBackward));
    root.insert(ctrl('j'), DhxKeyTrie::cmd(EditorCommand::InsertNewline));
    root.insert(ctrl('k'), DhxKeyTrie::cmd(EditorCommand::KillToLineEnd));
    root.insert(ctrl(' '), DhxKeyTrie::cmd(EditorCommand::TriggerCompletion));
    root.insert(ctrl('.'), DhxKeyTrie::cmd(EditorCommand::ShowCodeActions));
    root.insert(ctrl('s'), DhxKeyTrie::cmd(EditorCommand::CommitUndoCheckpoint));
    root.insert(ctrl('w'), DhxKeyTrie::cmd(EditorCommand::DeleteWordBackward));
    root.insert(ctrl('x'), DhxKeyTrie::cmd(EditorCommand::TriggerCompletion));
    root.insert(ctrl('u'), DhxKeyTrie::cmd(EditorCommand::DeleteToLineStart));
    root.insert(ctrl('r'), DhxKeyTrie::await_char(AwaitCharKind::InsertRegister));

    root
}

// --- Sub-tree builders ---

fn g_prefix_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new("goto");
    node.insert(key('.'), DhxKeyTrie::cmd(EditorCommand::GotoLastModification));
    node.insert(key('a'), DhxKeyTrie::cmd(EditorCommand::GotoLastAccessedFile));
    node.insert(key('b'), DhxKeyTrie::cmd(EditorCommand::GotoWindowBottom));
    node.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::GotoWindowCenter));
    node.insert(key('D'), DhxKeyTrie::cmd(EditorCommand::GotoDeclaration));
    node.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::GotoDefinition));
    node.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::GotoLastLine));
    node.insert(key('f'), DhxKeyTrie::cmd(EditorCommand::GotoFileUnderCursor));
    node.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::GotoFirstLine));
    node.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::MoveLineStart));
    node.insert(key('i'), DhxKeyTrie::cmd(EditorCommand::GotoImplementation));
    node.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::MoveDown));
    node.insert(key('k'), DhxKeyTrie::cmd(EditorCommand::MoveUp));
    node.insert(key('l'), DhxKeyTrie::cmd(EditorCommand::MoveLineEnd));
    node.insert(key('m'), DhxKeyTrie::cmd(EditorCommand::GotoLastModifiedFile));
    node.insert(key('n'), DhxKeyTrie::cmd(EditorCommand::NextBuffer));
    node.insert(key('p'), DhxKeyTrie::cmd(EditorCommand::PreviousBuffer));
    node.insert(key('r'), DhxKeyTrie::cmd(EditorCommand::GotoReferences));
    node.insert(key('s'), DhxKeyTrie::cmd(EditorCommand::GotoFirstNonWhitespace));
    node.insert(key('t'), DhxKeyTrie::cmd(EditorCommand::GotoWindowTop));
    node.insert(key('w'), DhxKeyTrie::cmd(EditorCommand::GotoWord));
    node.insert(key('y'), DhxKeyTrie::cmd(EditorCommand::GotoTypeDefinition));
    node.insert(key('|'), DhxKeyTrie::cmd(EditorCommand::GotoColumn));
    node
}

fn select_g_prefix_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new("goto");
    // Extend variants (preserve anchor)
    node.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::ExtendToFirstLine));
    node.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::ExtendToLastLine));
    node.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::ExtendLineStart));
    node.insert(key('l'), DhxKeyTrie::cmd(EditorCommand::ExtendLineEnd));
    node.insert(key('s'), DhxKeyTrie::cmd(EditorCommand::ExtendGotoFirstNonWhitespace));
    node.insert(key('|'), DhxKeyTrie::cmd(EditorCommand::ExtendGotoColumn));

    // LSP gotos (jump, don't extend)
    node.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::GotoDefinition));
    node.insert(key('D'), DhxKeyTrie::cmd(EditorCommand::GotoDeclaration));
    node.insert(key('y'), DhxKeyTrie::cmd(EditorCommand::GotoTypeDefinition));
    node.insert(key('i'), DhxKeyTrie::cmd(EditorCommand::GotoImplementation));
    node.insert(key('r'), DhxKeyTrie::cmd(EditorCommand::GotoReferences));
    node.insert(key('f'), DhxKeyTrie::cmd(EditorCommand::GotoFileUnderCursor));

    // gw - word jump (extends selection in select mode)
    node.insert(key('w'), DhxKeyTrie::cmd(EditorCommand::ExtendToWord));

    // Window/buffer navigation (same as normal mode)
    node.insert(key('t'), DhxKeyTrie::cmd(EditorCommand::GotoWindowTop));
    node.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::GotoWindowCenter));
    node.insert(key('b'), DhxKeyTrie::cmd(EditorCommand::GotoWindowBottom));
    node.insert(key('n'), DhxKeyTrie::cmd(EditorCommand::NextBuffer));
    node.insert(key('p'), DhxKeyTrie::cmd(EditorCommand::PreviousBuffer));
    node.insert(key('a'), DhxKeyTrie::cmd(EditorCommand::GotoLastAccessedFile));
    node.insert(key('m'), DhxKeyTrie::cmd(EditorCommand::GotoLastModifiedFile));
    node.insert(key('.'), DhxKeyTrie::cmd(EditorCommand::GotoLastModification));
    node
}

fn bracket_next_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new("next");
    node.insert(key('a'), DhxKeyTrie::cmd(EditorCommand::NextParameter));
    node.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::NextComment));
    node.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::NextDiagnostic));
    node.insert(key('f'), DhxKeyTrie::cmd(EditorCommand::NextFunction));
    node.insert(key('p'), DhxKeyTrie::cmd(EditorCommand::NextParagraph));
    node.insert(key('t'), DhxKeyTrie::cmd(EditorCommand::NextClass));
    node.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::NextChange));
    node.insert(key('D'), DhxKeyTrie::cmd(EditorCommand::GotoLastDiagnostic));
    node.insert(key('G'), DhxKeyTrie::cmd(EditorCommand::GotoLastChange));
    node.insert(key(' '), DhxKeyTrie::cmd(EditorCommand::AddNewlineBelow));
    node
}

fn bracket_prev_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new("prev");
    node.insert(key('a'), DhxKeyTrie::cmd(EditorCommand::PrevParameter));
    node.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::PrevComment));
    node.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::PrevDiagnostic));
    node.insert(key('f'), DhxKeyTrie::cmd(EditorCommand::PrevFunction));
    node.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::PrevChange));
    node.insert(key('p'), DhxKeyTrie::cmd(EditorCommand::PrevParagraph));
    node.insert(key('t'), DhxKeyTrie::cmd(EditorCommand::PrevClass));
    node.insert(key('D'), DhxKeyTrie::cmd(EditorCommand::GotoFirstDiagnostic));
    node.insert(key('G'), DhxKeyTrie::cmd(EditorCommand::GotoFirstChange));
    node.insert(key(' '), DhxKeyTrie::cmd(EditorCommand::AddNewlineAbove));
    node
}

fn space_leader_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new("space");
    node.insert(key('/'), DhxKeyTrie::cmd(EditorCommand::ShowGlobalSearch));
    node.insert(key('?'), DhxKeyTrie::cmd(EditorCommand::ShowCommandPanel));
    node.insert(key('a'), DhxKeyTrie::cmd(EditorCommand::ShowCodeActions));
    node.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::ToggleLineComment));
    node.insert(key('C'), DhxKeyTrie::cmd(EditorCommand::ToggleBlockComment));
    node.insert(key('b'), DhxKeyTrie::cmd(EditorCommand::ShowBufferPicker));
    node.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::ShowDocumentDiagnostics));
    node.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::ShowFileExplorer));
    node.insert(key('E'), DhxKeyTrie::cmd(EditorCommand::ShowFileExplorerInBufferDir));
    node.insert(key('D'), DhxKeyTrie::cmd(EditorCommand::ShowWorkspaceDiagnostics));
    node.insert(key('f'), DhxKeyTrie::cmd(EditorCommand::ShowFilePicker));
    node.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::ShowChangedFilesPicker));
    node.insert(key('F'), DhxKeyTrie::cmd(EditorCommand::ShowFilePickerInBufferDir));
    node.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::SelectReferencesToSymbol));
    node.insert(key('i'), DhxKeyTrie::cmd(EditorCommand::ToggleInlayHints));
    node.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::ShowJumpListPicker));
    node.insert(key('k'), DhxKeyTrie::cmd(EditorCommand::TriggerHover));
    node.insert(key('p'), DhxKeyTrie::seq(vec![
        EditorCommand::SetSelectedRegister('+'),
        EditorCommand::Paste,
    ]));
    node.insert(key('P'), DhxKeyTrie::seq(vec![
        EditorCommand::SetSelectedRegister('+'),
        EditorCommand::PasteBefore,
    ]));
    node.insert(key('r'), DhxKeyTrie::cmd(EditorCommand::RenameSymbol));
    node.insert(key('R'), DhxKeyTrie::seq(vec![
        EditorCommand::SetSelectedRegister('+'),
        EditorCommand::ReplaceWithYanked,
    ]));
    node.insert(key('s'), DhxKeyTrie::cmd(EditorCommand::ShowDocumentSymbols));
    node.insert(key('S'), DhxKeyTrie::cmd(EditorCommand::ShowWorkspaceSymbols));
    node.insert(key('y'), DhxKeyTrie::seq(vec![
        EditorCommand::SetSelectedRegister('+'),
        EditorCommand::Yank,
    ]));
    node.insert(key('Y'), DhxKeyTrie::seq(vec![
        EditorCommand::SetSelectedRegister('+'),
        EditorCommand::YankMainSelectionToClipboard,
    ]));
    node.insert(key('\''), DhxKeyTrie::cmd(EditorCommand::ShowLastPicker));
    node
}

fn match_prefix_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new("match");
    node.insert(key('m'), DhxKeyTrie::cmd(EditorCommand::MatchBracket));
    node.insert(key('i'), DhxKeyTrie::await_char(AwaitCharKind::SelectInsidePair));
    node.insert(key('a'), DhxKeyTrie::await_char(AwaitCharKind::SelectAroundPair));
    node.insert(key('s'), DhxKeyTrie::await_char(AwaitCharKind::SurroundAdd));
    node.insert(key('d'), DhxKeyTrie::await_char(AwaitCharKind::SurroundDelete));
    node.insert(key('r'), DhxKeyTrie::await_char(AwaitCharKind::SurroundReplaceFrom));
    node
}

fn view_prefix_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new("view");

    // Alignment
    node.insert(key('z'), DhxKeyTrie::cmd(EditorCommand::AlignViewCenter));
    node.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::AlignViewCenter));
    node.insert(key('m'), DhxKeyTrie::cmd(EditorCommand::AlignViewCenter));
    node.insert(key('t'), DhxKeyTrie::cmd(EditorCommand::AlignViewTop));
    node.insert(key('b'), DhxKeyTrie::cmd(EditorCommand::AlignViewBottom));

    // Scroll by line
    node.insert(key('k'), DhxKeyTrie::cmd(EditorCommand::ScrollUp(1)));
    node.insert(special(KeyCode::Up), DhxKeyTrie::cmd(EditorCommand::ScrollUp(1)));
    node.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::ScrollDown(1)));
    node.insert(special(KeyCode::Down), DhxKeyTrie::cmd(EditorCommand::ScrollDown(1)));

    // Page scroll
    node.insert(special(KeyCode::PageUp), DhxKeyTrie::cmd(EditorCommand::PageUp));
    node.insert(special(KeyCode::PageDown), DhxKeyTrie::cmd(EditorCommand::PageDown));
    node.insert(special(KeyCode::Backspace), DhxKeyTrie::cmd(EditorCommand::HalfPageUp));
    node.insert(key(' '), DhxKeyTrie::cmd(EditorCommand::HalfPageDown));

    // Ctrl combinations inside view mode
    node.insert(ctrl('b'), DhxKeyTrie::cmd(EditorCommand::PageUp));
    node.insert(ctrl('f'), DhxKeyTrie::cmd(EditorCommand::PageDown));
    node.insert(ctrl('u'), DhxKeyTrie::cmd(EditorCommand::HalfPageUp));
    node.insert(ctrl('d'), DhxKeyTrie::cmd(EditorCommand::HalfPageDown));

    // Search
    node.insert(key('/'), DhxKeyTrie::cmd(EditorCommand::EnterSearchMode { backwards: false }));
    node.insert(key('?'), DhxKeyTrie::cmd(EditorCommand::EnterSearchMode { backwards: true }));
    node.insert(key('n'), DhxKeyTrie::cmd(EditorCommand::SearchNext));
    node.insert(key('N'), DhxKeyTrie::cmd(EditorCommand::SearchPrevious));

    node
}

fn view_prefix_sticky_defaults() -> DhxKeyTrieNode {
    let mut node = DhxKeyTrieNode::new_sticky("view");

    // Same bindings as non-sticky view
    node.insert(key('z'), DhxKeyTrie::cmd(EditorCommand::AlignViewCenter));
    node.insert(key('c'), DhxKeyTrie::cmd(EditorCommand::AlignViewCenter));
    node.insert(key('m'), DhxKeyTrie::cmd(EditorCommand::AlignViewCenter));
    node.insert(key('t'), DhxKeyTrie::cmd(EditorCommand::AlignViewTop));
    node.insert(key('b'), DhxKeyTrie::cmd(EditorCommand::AlignViewBottom));

    node.insert(key('k'), DhxKeyTrie::cmd(EditorCommand::ScrollUp(1)));
    node.insert(special(KeyCode::Up), DhxKeyTrie::cmd(EditorCommand::ScrollUp(1)));
    node.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::ScrollDown(1)));
    node.insert(special(KeyCode::Down), DhxKeyTrie::cmd(EditorCommand::ScrollDown(1)));

    node.insert(special(KeyCode::PageUp), DhxKeyTrie::cmd(EditorCommand::PageUp));
    node.insert(special(KeyCode::PageDown), DhxKeyTrie::cmd(EditorCommand::PageDown));
    node.insert(special(KeyCode::Backspace), DhxKeyTrie::cmd(EditorCommand::HalfPageUp));
    node.insert(key(' '), DhxKeyTrie::cmd(EditorCommand::HalfPageDown));

    node.insert(ctrl('b'), DhxKeyTrie::cmd(EditorCommand::PageUp));
    node.insert(ctrl('f'), DhxKeyTrie::cmd(EditorCommand::PageDown));
    node.insert(ctrl('u'), DhxKeyTrie::cmd(EditorCommand::HalfPageUp));
    node.insert(ctrl('d'), DhxKeyTrie::cmd(EditorCommand::HalfPageDown));

    node.insert(key('/'), DhxKeyTrie::cmd(EditorCommand::EnterSearchMode { backwards: false }));
    node.insert(key('?'), DhxKeyTrie::cmd(EditorCommand::EnterSearchMode { backwards: true }));
    node.insert(key('n'), DhxKeyTrie::cmd(EditorCommand::SearchNext));
    node.insert(key('N'), DhxKeyTrie::cmd(EditorCommand::SearchPrevious));

    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::command::CommandSlot;
    use crate::keymap::trie::TrieSearchResult;

    /// Helper: search a trie and expect a single command.
    fn expect_cmd(trie: &DhxKeyTrie, keys: &[helix_view::input::KeyEvent]) -> EditorCommand {
        match trie.search(keys) {
            TrieSearchResult::Found(CommandSlot::Cmd(cmd)) => cmd.clone(),
            TrieSearchResult::Found(CommandSlot::Seq(cmds)) => {
                panic!("expected single cmd, got seq: {cmds:?}")
            }
            TrieSearchResult::Found(CommandSlot::AwaitChar(kind)) => {
                panic!("expected cmd, got await char: {kind:?}")
            }
            TrieSearchResult::FoundSeq(slots) => {
                panic!("expected single cmd, got sequence: {slots:?}")
            }
            TrieSearchResult::Partial(node) => {
                panic!("expected cmd, got partial node: {}", node.name())
            }
            TrieSearchResult::NotFound => {
                panic!("key sequence not found in trie")
            }
        }
    }

    /// Helper: search a trie and expect an await-char.
    fn expect_await(trie: &DhxKeyTrie, keys: &[helix_view::input::KeyEvent]) -> AwaitCharKind {
        match trie.search(keys) {
            TrieSearchResult::Found(CommandSlot::AwaitChar(kind)) => *kind,
            other => panic!("expected AwaitChar, got {other:?}"),
        }
    }

    /// Helper: search a trie and expect a command sequence.
    fn expect_seq(trie: &DhxKeyTrie, keys: &[helix_view::input::KeyEvent]) -> Vec<EditorCommand> {
        match trie.search(keys) {
            TrieSearchResult::Found(CommandSlot::Seq(cmds)) => cmds.clone(),
            TrieSearchResult::Found(CommandSlot::Cmd(cmd)) => vec![cmd.clone()],
            other => panic!("expected seq, got {other:?}"),
        }
    }

    /// Helper: search a trie and expect a partial (sub-node).
    fn expect_partial(trie: &DhxKeyTrie, keys: &[helix_view::input::KeyEvent]) {
        match trie.search(keys) {
            TrieSearchResult::Partial(_) => {}
            other => panic!("expected Partial, got {other:?}"),
        }
    }

    // ======== Normal Mode ========

    #[test]
    fn normal_hjkl_movement() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[key('h')]), EditorCommand::MoveLeft));
        assert!(matches!(expect_cmd(&trie, &[key('j')]), EditorCommand::MoveDown));
        assert!(matches!(expect_cmd(&trie, &[key('k')]), EditorCommand::MoveUp));
        assert!(matches!(expect_cmd(&trie, &[key('l')]), EditorCommand::MoveRight));
    }

    #[test]
    fn normal_arrow_movement() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Left)]), EditorCommand::MoveLeft));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Right)]), EditorCommand::MoveRight));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Up)]), EditorCommand::MoveUp));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Down)]), EditorCommand::MoveDown));
    }

    #[test]
    fn normal_word_movement() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[key('w')]), EditorCommand::MoveWordForward));
        assert!(matches!(expect_cmd(&trie, &[key('b')]), EditorCommand::MoveWordBackward));
        assert!(matches!(expect_cmd(&trie, &[key('e')]), EditorCommand::MoveWordEnd));
        assert!(matches!(expect_cmd(&trie, &[key('W')]), EditorCommand::MoveLongWordForward));
        assert!(matches!(expect_cmd(&trie, &[key('B')]), EditorCommand::MoveLongWordBackward));
        assert!(matches!(expect_cmd(&trie, &[key('E')]), EditorCommand::MoveLongWordEnd));
    }

    #[test]
    fn normal_g_prefix() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        expect_partial(&trie, &[key('g')]);
        assert!(matches!(expect_cmd(&trie, &[key('g'), key('g')]), EditorCommand::GotoFirstLine));
        assert!(matches!(expect_cmd(&trie, &[key('g'), key('e')]), EditorCommand::GotoLastLine));
        assert!(matches!(expect_cmd(&trie, &[key('g'), key('d')]), EditorCommand::GotoDefinition));
        assert!(matches!(expect_cmd(&trie, &[key('g'), key('r')]), EditorCommand::GotoReferences));
        assert!(matches!(expect_cmd(&trie, &[key('g'), key('h')]), EditorCommand::MoveLineStart));
        assert!(matches!(expect_cmd(&trie, &[key('g'), key('l')]), EditorCommand::MoveLineEnd));
        assert!(matches!(expect_cmd(&trie, &[key('g'), key('w')]), EditorCommand::GotoWord));
    }

    #[test]
    fn normal_space_leader() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        expect_partial(&trie, &[key(' ')]);
        assert!(matches!(expect_cmd(&trie, &[key(' '), key('f')]), EditorCommand::ShowFilePicker));
        assert!(matches!(expect_cmd(&trie, &[key(' '), key('b')]), EditorCommand::ShowBufferPicker));
        assert!(matches!(expect_cmd(&trie, &[key(' '), key('r')]), EditorCommand::RenameSymbol));
        assert!(matches!(expect_cmd(&trie, &[key(' '), key('k')]), EditorCommand::TriggerHover));
    }

    #[test]
    fn normal_space_clipboard_sequences() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        let cmds = expect_seq(&trie, &[key(' '), key('y')]);
        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], EditorCommand::SetSelectedRegister('+')));
        assert!(matches!(cmds[1], EditorCommand::Yank));
    }

    #[test]
    fn normal_bracket_next() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        expect_partial(&trie, &[key(']')]);
        assert!(matches!(expect_cmd(&trie, &[key(']'), key('d')]), EditorCommand::NextDiagnostic));
        assert!(matches!(expect_cmd(&trie, &[key(']'), key('f')]), EditorCommand::NextFunction));
        assert!(matches!(expect_cmd(&trie, &[key(']'), key('g')]), EditorCommand::NextChange));
    }

    #[test]
    fn normal_bracket_prev() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        expect_partial(&trie, &[key('[')]);
        assert!(matches!(expect_cmd(&trie, &[key('['), key('d')]), EditorCommand::PrevDiagnostic));
        assert!(matches!(expect_cmd(&trie, &[key('['), key('f')]), EditorCommand::PrevFunction));
        assert!(matches!(expect_cmd(&trie, &[key('['), key('g')]), EditorCommand::PrevChange));
    }

    #[test]
    fn normal_match_prefix() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        expect_partial(&trie, &[key('m')]);
        assert!(matches!(expect_cmd(&trie, &[key('m'), key('m')]), EditorCommand::MatchBracket));
        assert!(matches!(
            expect_await(&trie, &[key('m'), key('i')]),
            AwaitCharKind::SelectInsidePair
        ));
        assert!(matches!(
            expect_await(&trie, &[key('m'), key('a')]),
            AwaitCharKind::SelectAroundPair
        ));
        assert!(matches!(
            expect_await(&trie, &[key('m'), key('s')]),
            AwaitCharKind::SurroundAdd
        ));
        assert!(matches!(
            expect_await(&trie, &[key('m'), key('d')]),
            AwaitCharKind::SurroundDelete
        ));
        assert!(matches!(
            expect_await(&trie, &[key('m'), key('r')]),
            AwaitCharKind::SurroundReplaceFrom
        ));
    }

    #[test]
    fn normal_view_prefix() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        expect_partial(&trie, &[key('z')]);
        assert!(matches!(expect_cmd(&trie, &[key('z'), key('z')]), EditorCommand::AlignViewCenter));
        assert!(matches!(expect_cmd(&trie, &[key('z'), key('t')]), EditorCommand::AlignViewTop));
        assert!(matches!(expect_cmd(&trie, &[key('z'), key('b')]), EditorCommand::AlignViewBottom));
        assert!(matches!(expect_cmd(&trie, &[key('z'), key('k')]), EditorCommand::ScrollUp(1)));
        assert!(matches!(expect_cmd(&trie, &[key('z'), key('j')]), EditorCommand::ScrollDown(1)));
    }

    #[test]
    fn normal_view_prefix_sticky() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        match trie.search(&[key('Z')]) {
            TrieSearchResult::Partial(node) => assert!(node.is_sticky()),
            other => panic!("expected sticky Partial, got {other:?}"),
        }
    }

    #[test]
    fn normal_await_char_commands() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        assert!(matches!(expect_await(&trie, &[key('f')]), AwaitCharKind::FindForward));
        assert!(matches!(expect_await(&trie, &[key('F')]), AwaitCharKind::FindBackward));
        assert!(matches!(expect_await(&trie, &[key('t')]), AwaitCharKind::TillForward));
        assert!(matches!(expect_await(&trie, &[key('T')]), AwaitCharKind::TillBackward));
        assert!(matches!(expect_await(&trie, &[key('r')]), AwaitCharKind::ReplaceChar));
        assert!(matches!(expect_await(&trie, &[key('"')]), AwaitCharKind::SelectRegister));
    }

    #[test]
    fn normal_alt_keys() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[alt('.')]), EditorCommand::RepeatLastFind));
        assert!(matches!(expect_cmd(&trie, &[alt(';')]), EditorCommand::FlipSelections));
        assert!(matches!(expect_cmd(&trie, &[alt('o')]), EditorCommand::ExpandSelection));
        assert!(matches!(expect_cmd(&trie, &[alt('i')]), EditorCommand::ShrinkSelection));
        assert!(matches!(expect_cmd(&trie, &[alt('d')]), EditorCommand::DeleteSelectionNoYank));
    }

    #[test]
    fn normal_ctrl_keys() {
        let trie = DhxKeyTrie::node(normal_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[ctrl('b')]), EditorCommand::PageUp));
        assert!(matches!(expect_cmd(&trie, &[ctrl('f')]), EditorCommand::PageDown));
        assert!(matches!(expect_cmd(&trie, &[ctrl('o')]), EditorCommand::JumpBackward));
        assert!(matches!(expect_cmd(&trie, &[ctrl('i')]), EditorCommand::JumpForward));
        assert!(matches!(expect_cmd(&trie, &[ctrl('r')]), EditorCommand::Redo));
        assert!(matches!(expect_cmd(&trie, &[ctrl('s')]), EditorCommand::SaveSelection));
    }

    // ======== Select Mode ========

    #[test]
    fn select_hjkl_extends() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[key('h')]), EditorCommand::ExtendLeft));
        assert!(matches!(expect_cmd(&trie, &[key('j')]), EditorCommand::ExtendDown));
        assert!(matches!(expect_cmd(&trie, &[key('k')]), EditorCommand::ExtendUp));
        assert!(matches!(expect_cmd(&trie, &[key('l')]), EditorCommand::ExtendRight));
    }

    #[test]
    fn select_word_extends() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[key('w')]), EditorCommand::ExtendWordForward));
        assert!(matches!(expect_cmd(&trie, &[key('b')]), EditorCommand::ExtendWordBackward));
        assert!(matches!(expect_cmd(&trie, &[key('e')]), EditorCommand::ExtendWordEnd));
    }

    #[test]
    fn select_esc_exits() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        assert!(matches!(
            expect_cmd(&trie, &[special(KeyCode::Esc)]),
            EditorCommand::ExitSelectMode
        ));
    }

    #[test]
    fn select_y_yanks_and_exits() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        let cmds = expect_seq(&trie, &[key('y')]);
        assert_eq!(cmds.len(), 2);
        assert!(matches!(cmds[0], EditorCommand::Yank));
        assert!(matches!(cmds[1], EditorCommand::ExitSelectMode));
    }

    #[test]
    fn select_g_prefix_extends() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        assert!(matches!(
            expect_cmd(&trie, &[key('g'), key('g')]),
            EditorCommand::ExtendToFirstLine
        ));
        assert!(matches!(
            expect_cmd(&trie, &[key('g'), key('e')]),
            EditorCommand::ExtendToLastLine
        ));
        assert!(matches!(
            expect_cmd(&trie, &[key('g'), key('h')]),
            EditorCommand::ExtendLineStart
        ));
        assert!(matches!(
            expect_cmd(&trie, &[key('g'), key('l')]),
            EditorCommand::ExtendLineEnd
        ));
    }

    #[test]
    fn select_g_prefix_lsp_gotos() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        assert!(matches!(
            expect_cmd(&trie, &[key('g'), key('d')]),
            EditorCommand::GotoDefinition
        ));
        assert!(matches!(
            expect_cmd(&trie, &[key('g'), key('r')]),
            EditorCommand::GotoReferences
        ));
    }

    #[test]
    fn select_search_extends() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[key('n')]), EditorCommand::ExtendSearchNext));
        assert!(matches!(expect_cmd(&trie, &[key('N')]), EditorCommand::ExtendSearchPrev));
    }

    #[test]
    fn select_replace_and_paste() {
        let trie = DhxKeyTrie::node(select_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[key('R')]), EditorCommand::ReplaceWithYanked));
        assert!(matches!(expect_cmd(&trie, &[key('p')]), EditorCommand::ReplaceWithYanked));
    }

    // ======== Insert Mode ========

    #[test]
    fn insert_esc_exits() {
        let trie = DhxKeyTrie::node(insert_mode_defaults());
        assert!(matches!(
            expect_cmd(&trie, &[special(KeyCode::Esc)]),
            EditorCommand::ExitInsertMode
        ));
    }

    #[test]
    fn insert_special_keys() {
        let trie = DhxKeyTrie::node(insert_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Tab)]), EditorCommand::InsertTab));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Enter)]), EditorCommand::InsertNewline));
        assert!(matches!(
            expect_cmd(&trie, &[special(KeyCode::Backspace)]),
            EditorCommand::DeleteCharBackward
        ));
        assert!(matches!(
            expect_cmd(&trie, &[special(KeyCode::Delete)]),
            EditorCommand::DeleteCharForward
        ));
    }

    #[test]
    fn insert_shift_tab_unindents() {
        let trie = DhxKeyTrie::node(insert_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[shift_tab()]), EditorCommand::UnindentLine));
    }

    #[test]
    fn insert_ctrl_keys() {
        let trie = DhxKeyTrie::node(insert_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[ctrl('k')]), EditorCommand::KillToLineEnd));
        assert!(matches!(expect_cmd(&trie, &[ctrl('w')]), EditorCommand::DeleteWordBackward));
        assert!(matches!(expect_cmd(&trie, &[ctrl('u')]), EditorCommand::DeleteToLineStart));
        assert!(matches!(expect_cmd(&trie, &[ctrl(' ')]), EditorCommand::TriggerCompletion));
        assert!(matches!(expect_cmd(&trie, &[ctrl('s')]), EditorCommand::CommitUndoCheckpoint));
    }

    #[test]
    fn insert_ctrl_r_insert_register() {
        let trie = DhxKeyTrie::node(insert_mode_defaults());
        assert!(matches!(
            expect_await(&trie, &[ctrl('r')]),
            AwaitCharKind::InsertRegister
        ));
    }

    #[test]
    fn insert_ctrl_cmd_space_emoji() {
        let trie = DhxKeyTrie::node(insert_mode_defaults());
        assert!(matches!(
            expect_cmd(&trie, &[ctrl_super(' ')]),
            EditorCommand::ShowEmojiPicker
        ));
    }

    #[test]
    fn insert_arrow_keys() {
        let trie = DhxKeyTrie::node(insert_mode_defaults());
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Left)]), EditorCommand::MoveLeft));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Right)]), EditorCommand::MoveRight));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Up)]), EditorCommand::MoveUp));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Down)]), EditorCommand::MoveDown));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::Home)]), EditorCommand::MoveLineStart));
        assert!(matches!(expect_cmd(&trie, &[special(KeyCode::End)]), EditorCommand::MoveLineEnd));
    }

    // ======== default_keymaps() ========

    #[test]
    fn default_keymaps_has_all_modes() {
        let maps = default_keymaps();
        assert!(maps.contains_key(&Mode::Normal));
        assert!(maps.contains_key(&Mode::Select));
        assert!(maps.contains_key(&Mode::Insert));
    }

    #[test]
    fn default_keymaps_normal_h_moves_left() {
        let maps = default_keymaps();
        let trie = maps.get(&Mode::Normal).expect("normal mode exists");
        assert!(matches!(expect_cmd(trie, &[key('h')]), EditorCommand::MoveLeft));
    }

    #[test]
    fn default_keymaps_select_h_extends_left() {
        let maps = default_keymaps();
        let trie = maps.get(&Mode::Select).expect("select mode exists");
        assert!(matches!(expect_cmd(trie, &[key('h')]), EditorCommand::ExtendLeft));
    }

    #[test]
    fn default_keymaps_insert_esc_exits() {
        let maps = default_keymaps();
        let trie = maps.get(&Mode::Insert).expect("insert mode exists");
        assert!(matches!(
            expect_cmd(trie, &[special(KeyCode::Esc)]),
            EditorCommand::ExitInsertMode
        ));
    }
}
