//! Key trie data structure for keymap dispatch.
//!
//! `DhxKeyTrie` is a tree where leaves are `CommandSlot` values and
//! internal nodes map `KeyEvent` → child trie. `DhxKeymaps` wraps
//! per-mode tries with stateful dispatch (pending keys, sticky nodes,
//! await-char).

use std::collections::HashMap;

use helix_view::input::{KeyCode, KeyEvent};

use super::command::{AwaitCharKind, CommandSlot};
use crate::state::EditorCommand;

/// A trie node for key dispatch.
#[derive(Debug, Clone)]
pub enum DhxKeyTrie {
    /// A leaf: execute this command slot.
    Command(CommandSlot),
    /// A leaf: execute multiple command slots in sequence.
    Sequence(Vec<CommandSlot>),
    /// An internal node with named children.
    Node(DhxKeyTrieNode),
}

/// An internal trie node with a name and children.
#[derive(Debug, Clone)]
pub struct DhxKeyTrieNode {
    /// Human-readable name for this node (e.g., "goto", "space", "match").
    name: String,
    /// Map of key → child trie.
    map: HashMap<KeyEvent, DhxKeyTrie>,
    /// Insertion order for display purposes.
    order: Vec<KeyEvent>,
    /// If true, this node stays active until Esc (e.g., Z view mode).
    is_sticky: bool,
}

impl DhxKeyTrieNode {
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            map: HashMap::new(),
            order: Vec::new(),
            is_sticky: false,
        }
    }

    #[must_use]
    pub fn new_sticky(name: &str) -> Self {
        Self {
            name: name.to_string(),
            map: HashMap::new(),
            order: Vec::new(),
            is_sticky: true,
        }
    }

    pub fn insert(&mut self, key: KeyEvent, trie: DhxKeyTrie) {
        if !self.map.contains_key(&key) {
            self.order.push(key);
        }
        self.map.insert(key, trie);
    }

    #[must_use]
    pub fn get(&self, key: &KeyEvent) -> Option<&DhxKeyTrie> {
        self.map.get(key)
    }

    #[must_use]
    pub fn is_sticky(&self) -> bool {
        self.is_sticky
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Merge another node into this one. Leaf entries in `other` override
    /// entries in `self`. Sub-nodes are merged recursively.
    pub fn merge(&mut self, other: Self) {
        for (key, trie) in other.map {
            match (self.map.get_mut(&key), trie) {
                (Some(DhxKeyTrie::Node(existing)), DhxKeyTrie::Node(incoming)) => {
                    existing.merge(incoming);
                }
                (_, incoming) => {
                    self.insert(key, incoming);
                }
            }
        }
    }
}

impl DhxKeyTrie {
    #[must_use]
    pub fn cmd(cmd: EditorCommand) -> Self {
        Self::Command(CommandSlot::Cmd(cmd))
    }

    #[must_use]
    pub fn seq(cmds: Vec<EditorCommand>) -> Self {
        Self::Command(CommandSlot::Seq(cmds))
    }

    #[must_use]
    pub fn await_char(kind: AwaitCharKind) -> Self {
        Self::Command(CommandSlot::AwaitChar(kind))
    }

    #[must_use]
    pub fn node(node: DhxKeyTrieNode) -> Self {
        Self::Node(node)
    }

    /// Search the trie for a sequence of keys, returning the leaf or sub-node found.
    #[must_use]
    pub fn search(&self, keys: &[KeyEvent]) -> TrieSearchResult<'_> {
        match keys.split_first() {
            None => match self {
                Self::Command(slot) => TrieSearchResult::Found(slot),
                Self::Sequence(slots) => TrieSearchResult::FoundSeq(slots),
                Self::Node(node) => TrieSearchResult::Partial(node),
            },
            Some((first, rest)) => match self {
                Self::Node(node) => match node.get(first) {
                    Some(child) => child.search(rest),
                    None => TrieSearchResult::NotFound,
                },
                _ => TrieSearchResult::NotFound,
            },
        }
    }
}

/// Result of searching a trie with a key sequence.
#[derive(Debug)]
pub enum TrieSearchResult<'a> {
    /// Found a leaf command slot.
    Found(&'a CommandSlot),
    /// Found a sequence of command slots.
    FoundSeq(&'a [CommandSlot]),
    /// Reached an internal node — more keys needed.
    Partial(&'a DhxKeyTrieNode),
    /// Key sequence not found in trie.
    NotFound,
}

/// Helper to create a `KeyEvent` from a character with no modifiers.
#[must_use]
pub fn key(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: helix_view::keyboard::KeyModifiers::NONE,
    }
}

/// Helper to create a `KeyEvent` with Ctrl modifier.
#[must_use]
pub fn ctrl(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: helix_view::keyboard::KeyModifiers::CONTROL,
    }
}

/// Helper to create a `KeyEvent` with Alt modifier.
#[must_use]
pub fn alt(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: helix_view::keyboard::KeyModifiers::ALT,
    }
}

/// Helper to create a `KeyEvent` for a special key code (no modifiers).
#[must_use]
pub fn special(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: helix_view::keyboard::KeyModifiers::NONE,
    }
}

/// Helper to create a `KeyEvent` with Ctrl+Super modifiers.
#[must_use]
pub fn ctrl_super(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: helix_view::keyboard::KeyModifiers::CONTROL | helix_view::keyboard::KeyModifiers::SUPER,
    }
}

/// Helper to create a `KeyEvent` with Shift+Tab.
#[must_use]
pub fn shift_tab() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Tab,
        modifiers: helix_view::keyboard::KeyModifiers::SHIFT,
    }
}

impl<'de> serde::Deserialize<'de> for DhxKeyTrie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DhxKeyTrieVisitor)
    }
}

/// Resolve a command string to a `DhxKeyTrie`.
///
/// Handles both regular commands (`"move_char_left"`) and typable commands
/// (`":write"`, `":buffer-close"`) that match helix-term's config format.
fn resolve_command_str(v: &str) -> Result<DhxKeyTrie, String> {
    if let Some(typable) = v.strip_prefix(':') {
        Ok(DhxKeyTrie::Command(CommandSlot::Cmd(EditorCommand::TypeableCommand(
            typable.to_string(),
        ))))
    } else {
        super::command::command_from_name(v)
            .map(DhxKeyTrie::Command)
            .ok_or_else(|| format!("unknown command: {v}"))
    }
}

struct DhxKeyTrieVisitor;

impl<'de> serde::de::Visitor<'de> for DhxKeyTrieVisitor {
    type Value = DhxKeyTrie;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a command name (string), array of commands, or a key map (table)")
    }

    // String → single command lookup or typable command (`:command`)
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        resolve_command_str(v).map_err(|msg| serde::de::Error::custom(msg))
    }

    // Array → sequence of commands
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut slots = Vec::new();
        while let Some(name) = seq.next_element::<String>()? {
            match resolve_command_str(&name) {
                Ok(DhxKeyTrie::Command(slot)) => slots.push(slot),
                Ok(DhxKeyTrie::Sequence(seq_slots)) => slots.extend(seq_slots),
                Ok(_) => {
                    return Err(serde::de::Error::custom(format!(
                        "unexpected trie node in sequence for: {name}"
                    )));
                }
                Err(msg) => return Err(serde::de::Error::custom(msg)),
            }
        }
        if slots.len() == 1 {
            Ok(DhxKeyTrie::Command(slots.remove(0)))
        } else {
            Ok(DhxKeyTrie::Sequence(slots))
        }
    }

    // Map → trie node (key string → child DhxKeyTrie)
    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        let mut node = DhxKeyTrieNode::new("");
        while let Some(key_str) = map.next_key::<String>()? {
            let key_event: helix_view::input::KeyEvent = key_str
                .parse()
                .map_err(|e: anyhow::Error| serde::de::Error::custom(e.to_string()))?;
            let child: DhxKeyTrie = map.next_value()?;
            node.insert(key_event, child);
        }
        Ok(DhxKeyTrie::Node(node))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_simple_trie() -> DhxKeyTrie {
        let mut root = DhxKeyTrieNode::new("root");
        root.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::MoveLeft));
        root.insert(key('l'), DhxKeyTrie::cmd(EditorCommand::MoveRight));

        // Nested node: g prefix
        let mut g_node = DhxKeyTrieNode::new("goto");
        g_node.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::GotoFirstLine));
        g_node.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::GotoLastLine));
        root.insert(key('g'), DhxKeyTrie::node(g_node));

        DhxKeyTrie::node(root)
    }

    #[test]
    fn search_single_key_found() {
        let trie = build_simple_trie();
        let result = trie.search(&[key('h')]);
        assert!(matches!(result, TrieSearchResult::Found(_)));
    }

    #[test]
    fn search_single_key_not_found() {
        let trie = build_simple_trie();
        let result = trie.search(&[key('x')]);
        assert!(matches!(result, TrieSearchResult::NotFound));
    }

    #[test]
    fn search_nested_key_found() {
        let trie = build_simple_trie();
        let result = trie.search(&[key('g'), key('g')]);
        assert!(matches!(result, TrieSearchResult::Found(_)));
    }

    #[test]
    fn search_partial_at_node() {
        let trie = build_simple_trie();
        let result = trie.search(&[key('g')]);
        assert!(matches!(result, TrieSearchResult::Partial(_)));
    }

    #[test]
    fn search_nested_not_found() {
        let trie = build_simple_trie();
        let result = trie.search(&[key('g'), key('z')]);
        assert!(matches!(result, TrieSearchResult::NotFound));
    }

    #[test]
    fn search_empty_keys_on_node_returns_partial() {
        let trie = build_simple_trie();
        let result = trie.search(&[]);
        assert!(matches!(result, TrieSearchResult::Partial(_)));
    }

    #[test]
    fn search_empty_keys_on_leaf_returns_found() {
        let leaf = DhxKeyTrie::cmd(EditorCommand::MoveLeft);
        let result = leaf.search(&[]);
        assert!(matches!(result, TrieSearchResult::Found(_)));
    }

    #[test]
    fn node_merge_leaf_overrides_leaf() {
        let mut base = DhxKeyTrieNode::new("base");
        base.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::MoveLeft));

        let mut delta = DhxKeyTrieNode::new("delta");
        delta.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::MoveRight));

        base.merge(delta);

        let trie = DhxKeyTrie::node(base);
        let result = trie.search(&[key('h')]);
        match result {
            TrieSearchResult::Found(CommandSlot::Cmd(cmd)) => {
                assert!(matches!(cmd, EditorCommand::MoveRight));
            }
            _ => panic!("expected Found(MoveRight)"),
        }
    }

    #[test]
    fn node_merge_adds_new_key() {
        let mut base = DhxKeyTrieNode::new("base");
        base.insert(key('h'), DhxKeyTrie::cmd(EditorCommand::MoveLeft));

        let mut delta = DhxKeyTrieNode::new("delta");
        delta.insert(key('j'), DhxKeyTrie::cmd(EditorCommand::MoveDown));

        base.merge(delta);

        let trie = DhxKeyTrie::node(base);
        assert!(matches!(trie.search(&[key('h')]), TrieSearchResult::Found(_)));
        assert!(matches!(trie.search(&[key('j')]), TrieSearchResult::Found(_)));
    }

    #[test]
    fn node_merge_sub_node_recursive() {
        let mut base = DhxKeyTrieNode::new("base");
        let mut base_g = DhxKeyTrieNode::new("goto");
        base_g.insert(key('g'), DhxKeyTrie::cmd(EditorCommand::GotoFirstLine));
        base_g.insert(key('e'), DhxKeyTrie::cmd(EditorCommand::GotoLastLine));
        base.insert(key('g'), DhxKeyTrie::node(base_g));

        let mut delta = DhxKeyTrieNode::new("delta");
        let mut delta_g = DhxKeyTrieNode::new("goto");
        delta_g.insert(key('d'), DhxKeyTrie::cmd(EditorCommand::GotoDefinition));
        delta.insert(key('g'), DhxKeyTrie::node(delta_g));

        base.merge(delta);

        let trie = DhxKeyTrie::node(base);
        // Original bindings preserved
        assert!(matches!(trie.search(&[key('g'), key('g')]), TrieSearchResult::Found(_)));
        assert!(matches!(trie.search(&[key('g'), key('e')]), TrieSearchResult::Found(_)));
        // New binding added
        assert!(matches!(trie.search(&[key('g'), key('d')]), TrieSearchResult::Found(_)));
    }

    #[test]
    fn sticky_node_preserves_flag() {
        let node = DhxKeyTrieNode::new_sticky("view");
        assert!(node.is_sticky());

        let regular = DhxKeyTrieNode::new("goto");
        assert!(!regular.is_sticky());
    }

    // --- Deserialization tests ---

    /// Helper to deserialize a DhxKeyTrie from a TOML value embedded in a map.
    fn deser_value(toml_str: &str) -> DhxKeyTrie {
        let wrapped = format!("key = {toml_str}");
        let table: std::collections::HashMap<String, DhxKeyTrie> =
            toml::from_str(&wrapped).expect("should deserialize");
        table.into_values().next().expect("should have one value")
    }

    #[test]
    fn deserialize_single_command() {
        let trie = deser_value(r#""move_char_left""#);
        match trie {
            DhxKeyTrie::Command(CommandSlot::Cmd(cmd)) => {
                assert!(matches!(cmd, EditorCommand::MoveLeft));
            }
            _ => panic!("expected Command(MoveLeft), got {trie:?}"),
        }
    }

    #[test]
    fn deserialize_sequence() {
        let trie = deser_value(r#"["yank", "collapse_selection"]"#);
        match trie {
            DhxKeyTrie::Sequence(slots) => {
                assert_eq!(slots.len(), 2);
                assert!(matches!(slots[0], CommandSlot::Cmd(EditorCommand::Yank)));
                assert!(matches!(slots[1], CommandSlot::Cmd(EditorCommand::CollapseSelection)));
            }
            _ => panic!("expected Sequence, got {trie:?}"),
        }
    }

    #[test]
    fn deserialize_nested_node() {
        let toml_str = r#"
            h = "move_char_left"
            l = "move_char_right"
            g = { g = "goto_file_start", e = "goto_file_end" }
        "#;
        let trie: DhxKeyTrie = toml::from_str(toml_str).expect("should deserialize");
        assert!(matches!(trie.search(&[key('h')]), TrieSearchResult::Found(_)));
        assert!(matches!(trie.search(&[key('l')]), TrieSearchResult::Found(_)));
        assert!(matches!(trie.search(&[key('g'), key('g')]), TrieSearchResult::Found(_)));
        assert!(matches!(trie.search(&[key('g'), key('e')]), TrieSearchResult::Found(_)));
    }

    #[test]
    fn deserialize_invalid_command_fails() {
        let toml_str = r#"key = "nonexistent_command""#;
        let result: Result<std::collections::HashMap<String, DhxKeyTrie>, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_key_with_modifiers() {
        let toml_str = r#"
            "C-b" = "page_up"
            "A-o" = "expand_selection"
        "#;
        let trie: DhxKeyTrie = toml::from_str(toml_str).expect("should deserialize");
        assert!(matches!(trie.search(&[ctrl('b')]), TrieSearchResult::Found(_)));
        assert!(matches!(trie.search(&[alt('o')]), TrieSearchResult::Found(_)));
    }

    #[test]
    fn deserialize_typable_command() {
        let trie = deser_value(r#"":write""#);
        match trie {
            DhxKeyTrie::Command(CommandSlot::Cmd(EditorCommand::TypeableCommand(cmd))) => {
                assert_eq!(cmd, "write");
            }
            _ => panic!("expected TypeableCommand, got {trie:?}"),
        }
    }

    #[test]
    fn deserialize_typable_in_sequence() {
        let toml_str = r#"
            "C-s" = ":write"
            ret = ["open_below", "normal_mode"]
        "#;
        let trie: DhxKeyTrie = toml::from_str(toml_str).expect("should deserialize");
        // C-s → typable command
        let result = trie.search(&[ctrl('s')]);
        match result {
            TrieSearchResult::Found(slot) => {
                assert!(matches!(slot, CommandSlot::Cmd(EditorCommand::TypeableCommand(_))));
            }
            other => panic!("expected Found(TypeableCommand), got {other:?}"),
        }
    }

    #[test]
    fn deserialize_ret_key_with_sequence() {
        let toml_str = r#"
            ret = ["open_below", "normal_mode"]
        "#;
        let trie: DhxKeyTrie = toml::from_str(toml_str).expect("should deserialize");
        let result = trie.search(&[special(KeyCode::Enter)]);
        match result {
            TrieSearchResult::FoundSeq(slots) => {
                assert_eq!(slots.len(), 2);
                assert!(matches!(slots[0], CommandSlot::Cmd(EditorCommand::OpenLineBelow)));
                assert!(matches!(slots[1], CommandSlot::Cmd(EditorCommand::ExitInsertMode)));
            }
            other => panic!("expected FoundSeq, got {other:?}"),
        }
    }
}
