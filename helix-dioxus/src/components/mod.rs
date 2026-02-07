//! UI Components for helix-dioxus.
//!
//! This module contains all Dioxus UI components for the editor interface.

mod buffer_bar;
mod code_actions;
mod completion;
mod confirmation_dialog;
mod diagnostics;
mod editor_view;
mod hover;
mod inline_dialog;
mod input_dialog;
mod location_picker;
mod lsp_dialog;
mod notification;
mod picker;
mod prompt;
mod scrollbar;
mod signature_help;
mod statusline;

pub use buffer_bar::BufferBar;
pub use code_actions::CodeActionsMenu;
pub use completion::CompletionPopup;
pub use confirmation_dialog::ConfirmationDialog;
pub use diagnostics::{
    diagnostics_for_line, first_diagnostic_for_line, highest_severity_for_line, DiagnosticMarker,
    DiagnosticUnderline, ErrorLens,
};
pub use editor_view::EditorView;
pub use hover::HoverPopup;
// Note: inline_dialog types are used internally by input_dialog.rs via super::inline_dialog
pub use input_dialog::InputDialog;
pub use location_picker::LocationPicker;
pub use lsp_dialog::LspStatusDialog;
pub use notification::NotificationContainer;
pub use picker::GenericPicker;
pub use prompt::{CommandPrompt, SearchPrompt};
pub use scrollbar::Scrollbar;
pub use signature_help::SignatureHelpPopup;
pub use statusline::StatusLine;
