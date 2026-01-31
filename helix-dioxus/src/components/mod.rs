//! UI Components for helix-dioxus.
//!
//! This module contains all Dioxus UI components for the editor interface.

mod buffer_bar;
mod code_actions;
mod completion;
mod diagnostics;
mod editor_view;
mod hover;
mod inlay_hints;
mod location_picker;
mod picker;
mod prompt;
mod signature_help;
mod statusline;

pub use buffer_bar::BufferBar;
pub use code_actions::CodeActionsMenu;
pub use completion::CompletionPopup;
pub use diagnostics::{
    first_diagnostic_for_line, highest_severity_for_line, DiagnosticMarker, ErrorLens,
};
pub use editor_view::EditorView;
pub use hover::HoverPopup;
// Note: inlay_hints utilities (format_hint, hints_for_line) are available
// but not re-exported until LSP client integration is complete.
pub use location_picker::LocationPicker;
pub use picker::GenericPicker;
pub use prompt::{CommandPrompt, SearchPrompt};
pub use signature_help::SignatureHelpPopup;
pub use statusline::StatusLine;
