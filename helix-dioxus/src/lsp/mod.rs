//! LSP integration for helix-dioxus.
//!
//! This module provides thread-safe snapshot types for LSP data and
//! handles the async bridge between LSP operations and the sync UI.

mod types;

#[allow(unused_imports)]
pub use types::{
    CodeActionSnapshot, CompletionItemKind, CompletionItemSnapshot, DiagnosticSeverity,
    DiagnosticSnapshot, HoverSnapshot, InlayHintKind, InlayHintSnapshot, LocationSnapshot,
    LspResponse, ParameterSnapshot, SignatureHelpSnapshot,
};

// Note: SignatureSnapshot is available but not re-exported until LSP client integration.
