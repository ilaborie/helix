//! UI Components for helix-dioxus.
//!
//! This module contains all Dioxus UI components for the editor interface.
//!
//! ## Submodules
//!
//! - [`lsp`] — LSP-related popups (code actions, completion, hover, signature help, location picker)
//! - [`dialog`] — Dialogs and prompts (confirmation, input, LSP status, notifications, command/search)
//! - [`picker`] — Generic picker overlay
//! - [`inline_dialog`] — Reusable inline dialog primitives

// Chrome components (stay at root level)
mod buffer_bar;
mod diagnostics;
mod editor_view;
mod keybinding_help;
mod scrollbar;
mod statusline;

// Submodules
mod dialog;
mod inline_dialog;
mod lsp;
mod picker;

pub use buffer_bar::BufferBar;
pub use diagnostics::{
    diagnostics_for_line, first_diagnostic_for_line, highest_severity_for_line, DiagnosticMarker,
    DiagnosticUnderline, ErrorLens,
};
pub use editor_view::EditorView;
pub use keybinding_help::KeybindingHelpBar;
pub use scrollbar::Scrollbar;
pub use statusline::StatusLine;

pub use dialog::{
    CommandCompletionPopup, CommandPrompt, ConfirmationDialog, InputDialog, LspStatusDialog,
    NotificationContainer, RegexPrompt, SearchPrompt, ShellPrompt,
};
pub use lsp::{CodeActionsMenu, CompletionPopup, HoverPopup, LocationPicker, SignatureHelpPopup};
pub use picker::GenericPicker;
