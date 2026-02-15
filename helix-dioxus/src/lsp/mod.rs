//! LSP integration for helix-dioxus.
//!
//! This module provides thread-safe snapshot types for LSP data and
//! handles the async bridge between LSP operations and the sync UI.

mod conversions;
pub mod diff;
mod types;

pub use conversions::{
    convert_code_actions, convert_completion_response, convert_document_symbols,
    convert_goto_response, convert_hover, convert_inlay_hints, convert_references_response,
    convert_signature_help, convert_workspace_symbols,
};
pub use types::{
    CodeActionPreview, CodeActionPreviewState, CodeActionSnapshot, CompletionItemKind,
    CompletionItemSnapshot, DiagnosticPickerEntry, DiagnosticSeverity, DiagnosticSnapshot,
    DiffChangeKind, DiffHunk, DiffLine, FileDiff, HoverSnapshot, InlayHintKind, InlayHintSnapshot,
    LocationSnapshot, LspResponse, LspServerSnapshot, LspServerStatus, ParameterSnapshot,
    SignatureHelpSnapshot, SignatureSnapshot, StoredCodeAction, SymbolKind, SymbolSnapshot,
};
