//! Custom Dioxus hooks for helix-dioxus components.

use dioxus::prelude::*;

use crate::state::EditorSnapshot;
use crate::AppState;

/// Subscribe to editor state changes and return the current snapshot.
///
/// Reads the `version` signal (triggering re-renders on change),
/// then fetches the latest `EditorSnapshot` from `AppState`.
#[must_use]
pub fn use_editor_snapshot(version: ReadSignal<usize>) -> (AppState, EditorSnapshot) {
    let _ = version();
    let app_state = use_context::<AppState>();
    let snapshot = app_state.get_snapshot();
    (app_state, snapshot)
}
