//! Configurable keymap system for helix-dioxus.
//!
//! This module provides a trie-based keymap that encodes key sequences as
//! paths through a tree, with `CommandSlot` values at the leaves. It supports:
//!
//! - Multi-key sequences (e.g., `g g` → goto first line)
//! - Await-char commands (e.g., `f <char>` → find forward)
//! - Sticky nodes (e.g., `Z` view mode stays active until Esc)
//! - User customization via `[keys]` in config.toml
//!
//! The keymap uses the same command names as helix-term for user familiarity.

pub mod command;
pub mod default;
pub mod trie;

use helix_view::document::Mode;
use helix_view::input::{KeyCode, KeyEvent};

use command::{resolve_await, AwaitCharKind, CommandSlot};
use trie::{DhxKeyTrie, DhxKeyTrieNode, TrieSearchResult};

use crate::state::EditorCommand;

/// Result of dispatching a key through the keymap.
#[derive(Debug)]
pub enum DhxKeymapResult {
    /// Key sequence matched — execute these commands.
    Matched(Vec<EditorCommand>),
    /// Key sequence is a prefix — more keys needed. The node is available
    /// for which-key display.
    Pending(String),
    /// Key matched an await-char command — need next character input.
    AwaitChar(AwaitCharKind),
    /// Key sequence not found in the trie.
    NotFound,
    /// Pending sequence was cancelled (Esc pressed). Returns consumed keys.
    Cancelled,
}

/// Internal: outcome of searching the trie, with owned data.
enum SearchOutcome {
    Found(CommandSlot),
    FoundSeq(Vec<CommandSlot>),
    Partial(String, bool, DhxKeyTrieNode),
    NotFound,
}

/// Internal: result of resolving a trie hit without mutable self borrow.
enum StaticTrieResult {
    Matched(Vec<EditorCommand>),
    AwaitChar(AwaitCharKind),
    Pending(String, DhxKeyTrieNode),
}

/// Stateful keymap dispatcher.
///
/// Wraps per-mode tries and tracks pending state for multi-key sequences,
/// sticky nodes, and await-char commands.
pub struct DhxKeymaps {
    /// Per-mode keymaps.
    map: std::collections::HashMap<Mode, DhxKeyTrie>,
    /// Accumulated keys for trie traversal in progress.
    state: Vec<KeyEvent>,
    /// Active sticky node (Z view mode).
    sticky: Option<DhxKeyTrieNode>,
    /// Waiting for next character (f, t, r, ", etc.).
    await_char: Option<AwaitCharKind>,
}

impl DhxKeymaps {
    /// Create keymaps from per-mode tries.
    #[must_use]
    pub fn new(map: std::collections::HashMap<Mode, DhxKeyTrie>) -> Self {
        Self {
            map,
            state: Vec::new(),
            sticky: None,
            await_char: None,
        }
    }

    /// Dispatch a key event in the given mode.
    pub fn get(&mut self, mode: Mode, key: KeyEvent) -> DhxKeymapResult {
        // If awaiting a character, resolve it
        if let Some(kind) = self.await_char.take() {
            if key.code == KeyCode::Esc {
                return DhxKeymapResult::Cancelled;
            }
            match key.code {
                KeyCode::Char(ch) => {
                    // Special case: SurroundReplaceFrom transitions to SurroundReplaceTo
                    if kind == AwaitCharKind::SurroundReplaceFrom {
                        self.await_char = Some(AwaitCharKind::SurroundReplaceTo(ch));
                        return DhxKeymapResult::AwaitChar(AwaitCharKind::SurroundReplaceTo(ch));
                    }
                    let cmds = resolve_await(kind, ch, mode);
                    return DhxKeymapResult::Matched(cmds);
                }
                _ => return DhxKeymapResult::NotFound,
            }
        }

        // Esc cancels any pending state
        if key.code == KeyCode::Esc && (!self.state.is_empty() || self.sticky.is_some()) {
            self.state.clear();
            self.sticky = None;
            return DhxKeymapResult::Cancelled;
        }

        // If we have a sticky node, search it
        if let Some(sticky) = self.sticky.take() {
            if key.code == KeyCode::Esc {
                return DhxKeymapResult::Cancelled;
            }
            match sticky.get(&key) {
                Some(trie) => {
                    let result = Self::resolve_trie_hit_static(trie);
                    // Re-install sticky for the next key
                    self.sticky = Some(sticky);
                    match result {
                        StaticTrieResult::Matched(cmds) => {
                            return DhxKeymapResult::Matched(cmds);
                        }
                        StaticTrieResult::AwaitChar(kind) => {
                            self.await_char = Some(kind);
                            return DhxKeymapResult::AwaitChar(kind);
                        }
                        StaticTrieResult::Pending(name, node) => {
                            // Nested node inside sticky — shouldn't happen normally
                            self.sticky = Some(node);
                            return DhxKeymapResult::Pending(name);
                        }
                    }
                }
                None => {
                    // Unrecognized key in sticky mode — exit sticky
                    return DhxKeymapResult::NotFound;
                }
            }
        }

        // Accumulate key for trie traversal
        self.state.push(key);

        // Get the trie for this mode and search it
        let search_result = {
            let Some(trie) = self.map.get(&mode) else {
                self.state.clear();
                return DhxKeymapResult::NotFound;
            };
            // Search and extract what we need before dropping the borrow
            match trie.search(&self.state) {
                TrieSearchResult::Found(slot) => SearchOutcome::Found(slot.clone()),
                TrieSearchResult::FoundSeq(slots) => {
                    SearchOutcome::FoundSeq(slots.to_vec())
                }
                TrieSearchResult::Partial(node) => {
                    SearchOutcome::Partial(node.name().to_string(), node.is_sticky(), node.clone())
                }
                TrieSearchResult::NotFound => SearchOutcome::NotFound,
            }
        };

        match search_result {
            SearchOutcome::Found(slot) => {
                self.state.clear();
                self.resolve_slot_owned(slot)
            }
            SearchOutcome::FoundSeq(slots) => {
                self.state.clear();
                let mut cmds = Vec::new();
                for slot in &slots {
                    match slot {
                        CommandSlot::Cmd(cmd) => cmds.push(cmd.clone()),
                        CommandSlot::Seq(seq) => cmds.extend(seq.iter().cloned()),
                        CommandSlot::AwaitChar(_) => {}
                    }
                }
                DhxKeymapResult::Matched(cmds)
            }
            SearchOutcome::Partial(name, is_sticky, node) => {
                if is_sticky {
                    self.state.clear();
                    self.sticky = Some(node);
                }
                DhxKeymapResult::Pending(name)
            }
            SearchOutcome::NotFound => {
                self.state.clear();
                DhxKeymapResult::NotFound
            }
        }
    }

    /// Whether the keymap is in a pending state (multi-key sequence in progress).
    #[must_use]
    pub fn is_pending(&self) -> bool {
        !self.state.is_empty() || self.sticky.is_some() || self.await_char.is_some()
    }

    /// Clear all pending state.
    pub fn reset(&mut self) {
        self.state.clear();
        self.sticky = None;
        self.await_char = None;
    }

    /// Whether the keymap is currently awaiting a character.
    #[must_use]
    pub fn is_awaiting_char(&self) -> bool {
        self.await_char.is_some()
    }

    /// Get the current await char kind, if any.
    #[must_use]
    pub fn await_char_kind(&self) -> Option<AwaitCharKind> {
        self.await_char
    }

    /// Whether the keymap is currently in sticky node mode (e.g., Z view).
    #[must_use]
    pub fn is_sticky(&self) -> bool {
        self.sticky.is_some()
    }

    /// Resolve a trie hit without borrowing self (for sticky node context).
    fn resolve_trie_hit_static(trie: &DhxKeyTrie) -> StaticTrieResult {
        match trie {
            DhxKeyTrie::Command(slot) => match slot {
                CommandSlot::Cmd(cmd) => StaticTrieResult::Matched(vec![cmd.clone()]),
                CommandSlot::Seq(cmds) => StaticTrieResult::Matched(cmds.clone()),
                CommandSlot::AwaitChar(kind) => StaticTrieResult::AwaitChar(*kind),
            },
            DhxKeyTrie::Sequence(slots) => {
                let mut cmds = Vec::new();
                for slot in slots {
                    match slot {
                        CommandSlot::Cmd(cmd) => cmds.push(cmd.clone()),
                        CommandSlot::Seq(seq) => cmds.extend(seq.iter().cloned()),
                        CommandSlot::AwaitChar(_) => {}
                    }
                }
                StaticTrieResult::Matched(cmds)
            }
            DhxKeyTrie::Node(node) => {
                StaticTrieResult::Pending(node.name().to_string(), node.clone())
            }
        }
    }

    /// Resolve an owned command slot.
    fn resolve_slot_owned(&mut self, slot: CommandSlot) -> DhxKeymapResult {
        match slot {
            CommandSlot::Cmd(cmd) => DhxKeymapResult::Matched(vec![cmd]),
            CommandSlot::Seq(cmds) => DhxKeymapResult::Matched(cmds),
            CommandSlot::AwaitChar(kind) => {
                self.await_char = Some(kind);
                DhxKeymapResult::AwaitChar(kind)
            }
        }
    }
}

/// Merge user key overrides into a base trie.
///
/// The `delta` trie is merged into `base`:
/// - Leaf entries in `delta` override entries in `base`
/// - Sub-nodes are merged recursively
pub fn merge_keys(base: &mut DhxKeyTrie, delta: DhxKeyTrie) {
    match (base, delta) {
        (DhxKeyTrie::Node(base_node), DhxKeyTrie::Node(delta_node)) => {
            base_node.merge(delta_node);
        }
        (base, delta) => {
            *base = delta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trie::key;

    fn build_test_keymaps() -> DhxKeymaps {
        let mut root = DhxKeyTrieNode::new("normal");
        root.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::MoveLeft));
        root.insert(key('l'), DhxKeyTrie::cmd(EditorCommand::MoveRight));
        root.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::MoveDown));
        root.insert(key('k'), DhxKeyTrie::cmd(EditorCommand::MoveUp));

        // g prefix
        let mut g_node = DhxKeyTrieNode::new("goto");
        g_node.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::GotoFirstLine));
        g_node.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::GotoLastLine));
        root.insert(key('g'), DhxKeyTrie::node(g_node));

        // f = await char (find forward)
        root.insert(
            key('f'),
            DhxKeyTrie::await_char(AwaitCharKind::FindForward),
        );

        // Z = sticky view mode
        let mut z_sticky = DhxKeyTrieNode::new_sticky("view");
        z_sticky.insert(key('z'), DhxKeyTrie::cmd(EditorCommand::AlignViewCenter));
        z_sticky.insert(key('t'), DhxKeyTrie::cmd(EditorCommand::AlignViewTop));
        root.insert(key('Z'), DhxKeyTrie::node(z_sticky));

        let mut map = std::collections::HashMap::new();
        map.insert(Mode::Normal, DhxKeyTrie::node(root));

        DhxKeymaps::new(map)
    }

    #[test]
    fn dispatch_single_key() {
        let mut km = build_test_keymaps();
        let result = km.get(Mode::Normal, key('h'));
        match result {
            DhxKeymapResult::Matched(cmds) => {
                assert_eq!(cmds.len(), 1);
                assert!(matches!(cmds[0], EditorCommand::MoveLeft));
            }
            _ => panic!("expected Matched"),
        }
    }

    #[test]
    fn dispatch_two_key_sequence() {
        let mut km = build_test_keymaps();

        // First key: pending
        let result = km.get(Mode::Normal, key('g'));
        assert!(matches!(result, DhxKeymapResult::Pending(_)));

        // Second key: matched
        let result = km.get(Mode::Normal, key('g'));
        match result {
            DhxKeymapResult::Matched(cmds) => {
                assert_eq!(cmds.len(), 1);
                assert!(matches!(cmds[0], EditorCommand::GotoFirstLine));
            }
            _ => panic!("expected Matched"),
        }
    }

    #[test]
    fn dispatch_not_found() {
        let mut km = build_test_keymaps();
        let result = km.get(Mode::Normal, key('x'));
        assert!(matches!(result, DhxKeymapResult::NotFound));
    }

    #[test]
    fn dispatch_await_char() {
        let mut km = build_test_keymaps();

        // Press f → await char
        let result = km.get(Mode::Normal, key('f'));
        assert!(matches!(result, DhxKeymapResult::AwaitChar(_)));

        // Press 'x' → resolved
        let result = km.get(Mode::Normal, key('x'));
        match result {
            DhxKeymapResult::Matched(cmds) => {
                assert_eq!(cmds.len(), 1);
                assert!(matches!(cmds[0], EditorCommand::FindCharForward('x')));
            }
            _ => panic!("expected Matched"),
        }
    }

    #[test]
    fn dispatch_esc_cancels_pending() {
        let mut km = build_test_keymaps();

        // Start a sequence
        let result = km.get(Mode::Normal, key('g'));
        assert!(matches!(result, DhxKeymapResult::Pending(_)));

        // Esc cancels
        let result = km.get(
            Mode::Normal,
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: helix_view::keyboard::KeyModifiers::NONE,
            },
        );
        assert!(matches!(result, DhxKeymapResult::Cancelled));
        assert!(!km.is_pending());
    }

    #[test]
    fn dispatch_esc_cancels_await_char() {
        let mut km = build_test_keymaps();

        // Press f → await char
        let result = km.get(Mode::Normal, key('f'));
        assert!(matches!(result, DhxKeymapResult::AwaitChar(_)));

        // Esc cancels
        let result = km.get(
            Mode::Normal,
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: helix_view::keyboard::KeyModifiers::NONE,
            },
        );
        assert!(matches!(result, DhxKeymapResult::Cancelled));
        assert!(!km.is_pending());
    }

    #[test]
    fn dispatch_sticky_node_stays_active() {
        let mut km = build_test_keymaps();

        // Press Z → enter sticky view mode
        let result = km.get(Mode::Normal, key('Z'));
        assert!(matches!(result, DhxKeymapResult::Pending(_)));
        assert!(km.is_pending());

        // Press z → align center (still sticky)
        let result = km.get(Mode::Normal, key('z'));
        match result {
            DhxKeymapResult::Matched(cmds) => {
                assert_eq!(cmds.len(), 1);
                assert!(matches!(cmds[0], EditorCommand::AlignViewCenter));
            }
            _ => panic!("expected Matched"),
        }
        // Still in sticky mode
        assert!(km.is_pending());

        // Press t → align top (still sticky)
        let result = km.get(Mode::Normal, key('t'));
        match result {
            DhxKeymapResult::Matched(cmds) => {
                assert_eq!(cmds.len(), 1);
                assert!(matches!(cmds[0], EditorCommand::AlignViewTop));
            }
            _ => panic!("expected Matched"),
        }
        assert!(km.is_pending());

        // Esc exits sticky
        let result = km.get(
            Mode::Normal,
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: helix_view::keyboard::KeyModifiers::NONE,
            },
        );
        assert!(matches!(result, DhxKeymapResult::Cancelled));
        assert!(!km.is_pending());
    }

    #[test]
    fn dispatch_sticky_unknown_key_exits() {
        let mut km = build_test_keymaps();

        // Enter sticky mode
        km.get(Mode::Normal, key('Z'));
        assert!(km.is_pending());

        // Unknown key exits sticky
        let result = km.get(Mode::Normal, key('x'));
        assert!(matches!(result, DhxKeymapResult::NotFound));
        assert!(!km.is_pending());
    }

    #[test]
    fn dispatch_surround_replace_two_char_await() {
        // Build a keymap with m prefix → mr (surround replace)
        let mut root = DhxKeyTrieNode::new("normal");
        let mut m_node = DhxKeyTrieNode::new("match");
        m_node.insert(
            key('r'),
            DhxKeyTrie::await_char(AwaitCharKind::SurroundReplaceFrom),
        );
        root.insert(key('m'), DhxKeyTrie::node(m_node));

        let mut map = std::collections::HashMap::new();
        map.insert(Mode::Normal, DhxKeyTrie::node(root));
        let mut km = DhxKeymaps::new(map);

        // m → pending
        let result = km.get(Mode::Normal, key('m'));
        assert!(matches!(result, DhxKeymapResult::Pending(_)));

        // r → await char (SurroundReplaceFrom)
        let result = km.get(Mode::Normal, key('r'));
        assert!(matches!(
            result,
            DhxKeymapResult::AwaitChar(AwaitCharKind::SurroundReplaceFrom)
        ));

        // '(' → await char transitions to SurroundReplaceTo('(')
        let result = km.get(Mode::Normal, key('('));
        assert!(matches!(
            result,
            DhxKeymapResult::AwaitChar(AwaitCharKind::SurroundReplaceTo('('))
        ));

        // '[' → resolved: SurroundReplace('(', '[')
        let result = km.get(Mode::Normal, key('['));
        match result {
            DhxKeymapResult::Matched(cmds) => {
                assert_eq!(cmds.len(), 1);
                assert!(matches!(cmds[0], EditorCommand::SurroundReplace('(', '[')));
            }
            _ => panic!("expected Matched, got {result:?}"),
        }
    }

    #[test]
    fn merge_keys_leaf_overrides() {
        let mut base = DhxKeyTrie::cmd(EditorCommand::MoveLeft);
        let delta = DhxKeyTrie::cmd(EditorCommand::MoveRight);
        merge_keys(&mut base, delta);

        match base {
            DhxKeyTrie::Command(CommandSlot::Cmd(cmd)) => {
                assert!(matches!(cmd, EditorCommand::MoveRight));
            }
            _ => panic!("expected Command(MoveRight)"),
        }
    }

    #[test]
    fn merge_keys_node_recursive() {
        let mut base_g = DhxKeyTrieNode::new("goto");
        base_g.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::GotoFirstLine));
        let mut base_root = DhxKeyTrieNode::new("root");
        base_root.insert(key('g'), DhxKeyTrie::node(base_g));
        let mut base = DhxKeyTrie::node(base_root);

        let mut delta_g = DhxKeyTrieNode::new("goto");
        delta_g.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::GotoDefinition));
        let mut delta_root = DhxKeyTrieNode::new("root");
        delta_root.insert(key('g'), DhxKeyTrie::node(delta_g));
        let delta = DhxKeyTrie::node(delta_root);

        merge_keys(&mut base, delta);

        // Both bindings should exist
        assert!(matches!(
            base.search(&[key('g'), key('g')]),
            TrieSearchResult::Found(_)
        ));
        assert!(matches!(
            base.search(&[key('g'), key('d')]),
            TrieSearchResult::Found(_)
        ));
    }

    #[test]
    fn is_pending_tracks_state() {
        let mut km = build_test_keymaps();
        assert!(!km.is_pending());

        km.get(Mode::Normal, key('g'));
        assert!(km.is_pending());

        km.get(Mode::Normal, key('g'));
        assert!(!km.is_pending());
    }

    #[test]
    fn reset_clears_all_state() {
        let mut km = build_test_keymaps();
        km.get(Mode::Normal, key('g'));
        assert!(km.is_pending());

        km.reset();
        assert!(!km.is_pending());
    }

    #[test]
    fn merge_user_config_ret_binding() {
        use trie::special;

        // Start with defaults
        let mut map = default::default_keymaps();

        // Simulate user config: [keys.normal] ret = ["open_below", "normal_mode"]
        let toml_str = r#"ret = ["open_below", "normal_mode"]"#;
        let user_trie: DhxKeyTrie = toml::from_str(toml_str).expect("should deserialize");

        // Merge into normal mode defaults
        merge_keys(map.get_mut(&Mode::Normal).expect("normal"), user_trie);

        // Dispatch Enter key through keymaps
        let mut km = DhxKeymaps::new(map);
        let result = km.get(Mode::Normal, special(KeyCode::Enter));
        match result {
            DhxKeymapResult::Matched(cmds) => {
                assert_eq!(cmds.len(), 2, "expected 2 commands, got {cmds:?}");
                assert!(matches!(cmds[0], EditorCommand::OpenLineBelow));
                assert!(matches!(cmds[1], EditorCommand::ExitInsertMode));
            }
            other => panic!("expected Matched with 2 commands, got {other:?}"),
        }
    }
}
