//! Custom Dioxus hooks for helix-dioxus components.

use dioxus::prelude::*;

use crate::state::EditorSnapshot;

/// Read the current editor snapshot from the signal context.
///
/// Components that call this automatically re-render when **any** snapshot field
/// changes — even fields the component doesn't use. For components that only need
/// a subset of fields, prefer `use_memo` with `use_snapshot_signal()` to avoid
/// unnecessary re-renders:
///
/// ```ignore
/// let signal = use_snapshot_signal();
/// let data = use_memo(move || {
///     let s = signal.read();
///     (s.field_a.clone(), s.field_b)
/// });
/// ```
#[must_use]
pub fn use_snapshot() -> EditorSnapshot {
    use_context::<Signal<EditorSnapshot>>().read().clone()
}

/// Get the snapshot signal for writing (e.g., after processing commands).
///
/// Use this in components that need to update the snapshot after sending commands.
#[must_use]
pub fn use_snapshot_signal() -> Signal<EditorSnapshot> {
    use_context::<Signal<EditorSnapshot>>()
}
