//! Event registration for helix-dioxus.
//!
//! Events must be registered with helix_event before hooks can be registered for them.
//! This module registers the helix-view events that helix-dioxus uses.

use helix_event::register_event;
use helix_view::events::{
    ConfigDidChange, DiagnosticsDidChange, DocumentDidChange, DocumentDidClose, DocumentDidOpen,
    DocumentFocusLost, LanguageServerExited, LanguageServerInitialized, SelectionDidChange,
};

/// Register all events used by helix-dioxus.
///
/// This must be called before `helix_view::handlers::register_hooks()` or any
/// other hook registration, otherwise the application will panic with
/// "Tried to register handler for unknown event".
pub fn register() {
    register_event::<DocumentDidOpen>();
    register_event::<DocumentDidChange>();
    register_event::<DocumentDidClose>();
    register_event::<DocumentFocusLost>();
    register_event::<SelectionDidChange>();
    register_event::<DiagnosticsDidChange>();
    register_event::<LanguageServerInitialized>();
    register_event::<LanguageServerExited>();
    register_event::<ConfigDidChange>();
}
