//! Test helpers for editor operation tests.
//!
//! Provides utilities to create an `EditorContext` from annotated text
//! (using helix-core's `#[|]#` selection syntax) and assert the resulting state.

use std::sync::mpsc;

use helix_core::test::{plain, print};
use helix_core::Transaction;
use helix_view::input::{KeyCode, KeyEvent, KeyModifiers};
use helix_view::{current_ref, DocumentId, ViewId};

use crate::state::EditorContext;

/// Global Tokio runtime shared across all tests.
///
/// `create_handlers()` spawns a `word_index::Handler` via `tokio::spawn`,
/// so a Tokio runtime must be active. We store it in a `OnceLock` so it
/// lives for the entire test process.
static TEST_RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

/// One-time initialization of the `helix_event` registry, plus enter the
/// shared Tokio runtime on the calling thread.
///
/// `events::register()` panics if called twice (the `helix_event` registry
/// rejects duplicate event registrations). `std::sync::Once` ensures
/// it runs exactly once even when tests execute in parallel.
///
/// The Tokio runtime enter guard is returned so callers can keep it alive
/// for the duration of the test.
pub(crate) fn init() -> tokio::runtime::EnterGuard<'static> {
    use std::sync::Once;
    static EVENTS_INIT: Once = Once::new();
    EVENTS_INIT.call_once(|| {
        helix_loader::initialize_config_file(None);
        helix_loader::initialize_log_file(None);
        crate::events::register();
    });

    let runtime = TEST_RUNTIME
        .get_or_init(|| tokio::runtime::Runtime::new().expect("tokio runtime should start"));
    runtime.enter()
}

/// Create an `EditorContext` with the document text and selection described
/// by the annotated string.
///
/// Uses helix-core's annotated format:
/// - `#[|h]#` — primary selection with head before anchor
/// - `#[h|]#` — primary selection with head after anchor
///
/// # Example
///
/// ```ignore
/// let mut ctx = test_context("#[|h]#ello\nworld\n");
/// ```
pub fn test_context(annotated: &str) -> EditorContext {
    let _guard = init();

    let (text, selection) = print(annotated);
    let (tx, rx) = mpsc::channel();

    let config = crate::config::DhxConfig::default();
    let mut ctx =
        EditorContext::new(&config, None, rx, tx).expect("EditorContext creation should succeed");

    // Replace the scratch buffer content with our test text
    {
        let (view, _doc) = current_ref!(ctx.editor);
        let doc_id = view.doc;
        let view_id = view.id;
        let doc = ctx.editor.document_mut(doc_id).expect("doc exists");

        // Build a transaction that replaces the entire document text
        let old_text = doc.text().clone();
        let old_len = old_text.len_chars();
        let transaction =
            Transaction::change(&old_text, std::iter::once((0, old_len, Some(text.into()))));
        doc.apply(&transaction, view_id);
        doc.set_selection(view_id, selection);
    }

    ctx
}

/// Get the current `(DocumentId, ViewId)` from the context.
pub fn doc_view(ctx: &EditorContext) -> (DocumentId, ViewId) {
    let (view, _doc) = current_ref!(ctx.editor);
    (view.doc, view.id)
}

/// Assert that the editor state matches the expected annotated string.
///
/// Converts the current document text and selection to annotated format
/// and compares against `expected`.
pub fn assert_state(ctx: &EditorContext, expected: &str) {
    let (view, doc) = current_ref!(ctx.editor);
    let text = doc.text().clone();
    let selection = doc.selection(view.id);
    let actual = plain(text, selection);
    assert_eq!(
        actual, expected,
        "\n--- actual ---\n{actual}\n--- expected ---\n{expected}\n"
    );
}

/// Assert that the document text matches the expected string (ignoring selection).
pub fn assert_text(ctx: &EditorContext, expected: &str) {
    let (_view, doc) = current_ref!(ctx.editor);
    let text: String = doc.text().slice(..).into();
    assert_eq!(
        text, expected,
        "\n--- actual ---\n{text}\n--- expected ---\n{expected}\n"
    );
}

/// Create a `KeyEvent` with no modifiers for the given character.
pub fn key(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyModifiers::NONE,
    }
}

/// Create a `KeyEvent` with the Ctrl modifier for the given character.
pub fn ctrl_key(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyModifiers::CONTROL,
    }
}

/// Create a `KeyEvent` with the Alt modifier for the given character.
pub fn alt_key(ch: char) -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyModifiers::ALT,
    }
}

/// Create a `KeyEvent` for a special (non-character) key.
pub fn special_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
    }
}

/// Assert that a command list contains exactly one command matching the pattern.
///
/// Usage: `assert_single_command!(cmds, EditorCommand::JumpBackward);`
#[macro_export]
macro_rules! assert_single_command {
    ($cmds:expr, $pattern:pat) => {{
        assert_eq!(
            $cmds.len(),
            1,
            "expected 1 command, got {}: {:?}",
            $cmds.len(),
            $cmds
        );
        assert!(
            matches!($cmds[0], $pattern),
            "expected {}, got {:?}",
            stringify!($pattern),
            $cmds[0]
        );
    }};
}
