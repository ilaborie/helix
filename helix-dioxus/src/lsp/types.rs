//! LSP snapshot types for thread-safe UI rendering.
//!
//! These types are simplified, Clone + Send + Sync versions of LSP types
//! that can be safely used in Dioxus components.

use std::path::PathBuf;

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum DiagnosticSeverity {
    /// A hint - lowest priority.
    Hint,
    /// An informational message.
    Info,
    /// A warning.
    #[default]
    Warning,
    /// An error - highest priority.
    Error,
}

impl DiagnosticSeverity {
    /// Returns the CSS color class for this severity.
    #[must_use]
    pub fn css_color(&self) -> &'static str {
        match self {
            Self::Error => "#e06c75",   // Red
            Self::Warning => "#e5c07b", // Yellow
            Self::Info => "#61afef",    // Blue
            Self::Hint => "#56b6c2",    // Cyan
        }
    }

}

impl From<helix_core::diagnostic::Severity> for DiagnosticSeverity {
    fn from(severity: helix_core::diagnostic::Severity) -> Self {
        match severity {
            helix_core::diagnostic::Severity::Error => Self::Error,
            helix_core::diagnostic::Severity::Warning => Self::Warning,
            helix_core::diagnostic::Severity::Info => Self::Info,
            helix_core::diagnostic::Severity::Hint => Self::Hint,
        }
    }
}

/// A diagnostic snapshot for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticSnapshot {
    /// The line number (1-indexed) where the diagnostic appears.
    pub line: usize,
    /// The start column (0-indexed) of the diagnostic range.
    pub start_col: usize,
    /// The end column (0-indexed, exclusive) of the diagnostic range.
    pub end_col: usize,
    /// The diagnostic message.
    pub message: String,
    /// The severity of the diagnostic.
    pub severity: DiagnosticSeverity,
    /// The source of the diagnostic (e.g., "rustc", "clippy").
    pub source: Option<String>,
    /// The diagnostic code (e.g., "E0308").
    pub code: Option<String>,
}

/// Kind of completion item for icon display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompletionItemKind {
    #[default]
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl CompletionItemKind {
    /// Returns a short display string for this kind.
    #[must_use]
    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Method => "fn",
            Self::Function => "fn",
            Self::Constructor => "new",
            Self::Field => "fld",
            Self::Variable => "var",
            Self::Class => "cls",
            Self::Interface => "ifc",
            Self::Module => "mod",
            Self::Property => "prp",
            Self::Unit => "unt",
            Self::Value => "val",
            Self::Enum => "enm",
            Self::Keyword => "kw",
            Self::Snippet => "snp",
            Self::Color => "clr",
            Self::File => "fil",
            Self::Reference => "ref",
            Self::Folder => "dir",
            Self::EnumMember => "mem",
            Self::Constant => "cst",
            Self::Struct => "str",
            Self::Event => "evt",
            Self::Operator => "op",
            Self::TypeParameter => "typ",
        }
    }

    /// Returns the CSS color for this kind.
    #[must_use]
    pub fn css_color(&self) -> &'static str {
        match self {
            Self::Function | Self::Method | Self::Constructor => "#61afef", // Blue
            Self::Variable | Self::Field | Self::Property => "#e06c75",     // Red
            Self::Class | Self::Struct | Self::Interface => "#e5c07b",      // Yellow
            Self::Module | Self::Folder => "#c678dd",                       // Purple
            Self::Keyword => "#c678dd",                                     // Purple
            Self::Constant | Self::EnumMember => "#d19a66",                 // Orange
            Self::Enum => "#e5c07b",                                        // Yellow
            Self::Snippet => "#98c379",                                     // Green
            _ => "#abb2bf",                                                 // Default gray
        }
    }
}

impl From<helix_lsp::lsp::CompletionItemKind> for CompletionItemKind {
    fn from(kind: helix_lsp::lsp::CompletionItemKind) -> Self {
        match kind {
            k if k == helix_lsp::lsp::CompletionItemKind::TEXT => Self::Text,
            k if k == helix_lsp::lsp::CompletionItemKind::METHOD => Self::Method,
            k if k == helix_lsp::lsp::CompletionItemKind::FUNCTION => Self::Function,
            k if k == helix_lsp::lsp::CompletionItemKind::CONSTRUCTOR => Self::Constructor,
            k if k == helix_lsp::lsp::CompletionItemKind::FIELD => Self::Field,
            k if k == helix_lsp::lsp::CompletionItemKind::VARIABLE => Self::Variable,
            k if k == helix_lsp::lsp::CompletionItemKind::CLASS => Self::Class,
            k if k == helix_lsp::lsp::CompletionItemKind::INTERFACE => Self::Interface,
            k if k == helix_lsp::lsp::CompletionItemKind::MODULE => Self::Module,
            k if k == helix_lsp::lsp::CompletionItemKind::PROPERTY => Self::Property,
            k if k == helix_lsp::lsp::CompletionItemKind::UNIT => Self::Unit,
            k if k == helix_lsp::lsp::CompletionItemKind::VALUE => Self::Value,
            k if k == helix_lsp::lsp::CompletionItemKind::ENUM => Self::Enum,
            k if k == helix_lsp::lsp::CompletionItemKind::KEYWORD => Self::Keyword,
            k if k == helix_lsp::lsp::CompletionItemKind::SNIPPET => Self::Snippet,
            k if k == helix_lsp::lsp::CompletionItemKind::COLOR => Self::Color,
            k if k == helix_lsp::lsp::CompletionItemKind::FILE => Self::File,
            k if k == helix_lsp::lsp::CompletionItemKind::REFERENCE => Self::Reference,
            k if k == helix_lsp::lsp::CompletionItemKind::FOLDER => Self::Folder,
            k if k == helix_lsp::lsp::CompletionItemKind::ENUM_MEMBER => Self::EnumMember,
            k if k == helix_lsp::lsp::CompletionItemKind::CONSTANT => Self::Constant,
            k if k == helix_lsp::lsp::CompletionItemKind::STRUCT => Self::Struct,
            k if k == helix_lsp::lsp::CompletionItemKind::EVENT => Self::Event,
            k if k == helix_lsp::lsp::CompletionItemKind::OPERATOR => Self::Operator,
            k if k == helix_lsp::lsp::CompletionItemKind::TYPE_PARAMETER => Self::TypeParameter,
            _ => Self::Text,
        }
    }
}

/// A completion item snapshot for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CompletionItemSnapshot {
    /// The label shown in the completion menu.
    pub label: String,
    /// Additional details shown after the label (e.g., type signature).
    pub detail: Option<String>,
    /// The kind of completion item.
    pub kind: CompletionItemKind,
    /// The text to insert when this item is selected.
    pub insert_text: String,
    /// Documentation for this item (rendered as text).
    pub documentation: Option<String>,
    /// Whether this item is deprecated.
    pub deprecated: bool,
    /// Filter text used for matching (if different from label).
    pub filter_text: Option<String>,
    /// Sort text used for ordering (if different from label).
    pub sort_text: Option<String>,
    /// Index in the original completion list (for applying edits).
    pub index: usize,
}

/// A hover information snapshot for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HoverSnapshot {
    /// The hover content as rendered text/markdown.
    pub contents: String,
    /// The range of text that this hover applies to.
    pub range: Option<(usize, usize)>,
}

/// A signature help snapshot for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SignatureHelpSnapshot {
    /// The function signatures.
    pub signatures: Vec<SignatureSnapshot>,
    /// The index of the active signature.
    pub active_signature: usize,
    /// The index of the active parameter.
    pub active_parameter: Option<usize>,
}

/// A single signature in signature help.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SignatureSnapshot {
    /// The signature label (e.g., "fn foo(a: i32, b: String) -> bool").
    pub label: String,
    /// Documentation for the signature.
    pub documentation: Option<String>,
    /// Parameter information.
    pub parameters: Vec<ParameterSnapshot>,
}

/// A parameter in a signature.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParameterSnapshot {
    /// The parameter label (e.g., "a: i32").
    pub label: String,
    /// Documentation for the parameter.
    pub documentation: Option<String>,
}

/// Kind of inlay hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InlayHintKind {
    /// A type hint (e.g., `: i32`).
    #[default]
    Type,
    /// A parameter hint (e.g., `name:`).
    Parameter,
}

impl From<helix_lsp::lsp::InlayHintKind> for InlayHintKind {
    fn from(kind: helix_lsp::lsp::InlayHintKind) -> Self {
        match kind {
            k if k == helix_lsp::lsp::InlayHintKind::TYPE => Self::Type,
            k if k == helix_lsp::lsp::InlayHintKind::PARAMETER => Self::Parameter,
            _ => Self::Type,
        }
    }
}

/// An inlay hint snapshot for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InlayHintSnapshot {
    /// The line number (1-indexed) where the hint appears.
    pub line: usize,
    /// The column (0-indexed) where the hint should be rendered.
    pub column: usize,
    /// The hint label text.
    pub label: String,
    /// The kind of hint.
    pub kind: InlayHintKind,
    /// Whether this is a padding hint (adds space before/after).
    pub padding_left: bool,
    /// Whether to add padding after the hint.
    pub padding_right: bool,
}

/// A location snapshot for goto operations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocationSnapshot {
    /// The file path.
    pub path: PathBuf,
    /// The line number (1-indexed).
    pub line: usize,
    /// The column number (1-indexed).
    pub column: usize,
    /// A preview of the line content.
    pub preview: Option<String>,
}

/// A code action snapshot for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CodeActionSnapshot {
    /// The title of the code action.
    pub title: String,
    /// The kind of action (e.g., "quickfix", "refactor").
    pub kind: Option<String>,
    /// Whether this is a preferred action.
    pub is_preferred: bool,
    /// Whether this action is disabled.
    pub disabled: Option<String>,
    /// Index in the original list (for execution).
    pub index: usize,
}

/// Stored code action data for execution.
/// This stores the original LSP data needed to apply the action.
#[derive(Debug, Clone)]
pub struct StoredCodeAction {
    /// The snapshot for display.
    pub snapshot: CodeActionSnapshot,
    /// The original LSP code action or command.
    pub lsp_item: helix_lsp::lsp::CodeActionOrCommand,
    /// The language server ID that provided this action.
    pub language_server_id: helix_lsp::LanguageServerId,
    /// The offset encoding for this language server.
    pub offset_encoding: helix_lsp::OffsetEncoding,
}

/// Status of a language server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LspServerStatus {
    /// Server is starting up.
    #[default]
    Starting,
    /// Server is initialized but still indexing/loading the project.
    Indexing,
    /// Server is running and ready.
    Running,
    /// Server has stopped.
    Stopped,
}

impl LspServerStatus {
    /// Returns the CSS color for this status.
    #[must_use]
    pub fn css_color(&self) -> &'static str {
        match self {
            Self::Starting => "#e5c07b", // Yellow
            Self::Indexing => "#61afef", // Blue (indexing)
            Self::Running => "#98c379",  // Green
            Self::Stopped => "#5c6370",  // Gray
        }
    }
}

/// Snapshot of a language server for UI display.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LspServerSnapshot {
    /// The server name (e.g., "rust-analyzer").
    pub name: String,
    /// Current status of the server.
    pub status: LspServerStatus,
    /// Document types this server handles (e.g., "rust", "python").
    pub languages: Vec<String>,
    /// Whether this server is active for the current document.
    pub active_for_current: bool,
    /// Current progress message (e.g., "Loading workspace", "Building proc-macros").
    pub progress_message: Option<String>,
}

/// Response types from async LSP operations.
#[derive(Debug, Clone)]
pub enum LspResponse {
    /// Completion items received.
    Completions(Vec<CompletionItemSnapshot>),
    /// Hover information received.
    Hover(Option<HoverSnapshot>),
    /// Signature help received.
    SignatureHelp(Option<SignatureHelpSnapshot>),
    /// Inlay hints received.
    InlayHints(Vec<InlayHintSnapshot>),
    /// Goto definition locations received.
    GotoDefinition(Vec<LocationSnapshot>),
    /// References locations received.
    References(Vec<LocationSnapshot>),
    /// Code actions received (with full data for execution).
    CodeActions(Vec<StoredCodeAction>),
    /// Code actions availability check result (for lightbulb indicator).
    /// Contains whether actions are available and the cached actions.
    CodeActionsAvailable(bool, Vec<StoredCodeAction>),
    /// Diagnostics updated.
    DiagnosticsUpdated,
    /// Format edits received (applied directly).
    FormatApplied,
    /// Workspace edit applied (from code action).
    WorkspaceEditApplied,
    /// Error from LSP operation.
    Error(String),
}
