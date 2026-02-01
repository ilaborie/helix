//! LSP integration for helix-dioxus.
//!
//! This module provides thread-safe snapshot types for LSP data and
//! handles the async bridge between LSP operations and the sync UI.

mod conversions;
mod types;

pub use conversions::{
    convert_code_actions, convert_completion_response, convert_goto_response, convert_hover,
    convert_inlay_hints, convert_references_response, convert_signature_help,
};
pub use types::{
    CodeActionSnapshot, CompletionItemKind, CompletionItemSnapshot, DiagnosticSeverity,
    DiagnosticSnapshot, HoverSnapshot, InlayHintKind, InlayHintSnapshot, LocationSnapshot,
    LspResponse, ParameterSnapshot, SignatureHelpSnapshot, SignatureSnapshot, StoredCodeAction,
};
