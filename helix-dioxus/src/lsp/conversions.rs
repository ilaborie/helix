//! LSP type conversions for helix-dioxus.
//!
//! This module provides conversion functions from LSP types to our thread-safe
//! snapshot types for UI rendering.

use helix_lsp::lsp::{self, MarkedString};
use helix_lsp::OffsetEncoding;

use super::{
    CodeActionSnapshot, CompletionItemKind, CompletionItemSnapshot, HoverSnapshot, InlayHintKind,
    InlayHintSnapshot, LocationSnapshot, ParameterSnapshot, SignatureHelpSnapshot,
    StoredCodeAction,
};

/// Convert LSP completion response to completion item snapshots.
pub fn convert_completion_response(
    response: lsp::CompletionResponse,
) -> Vec<CompletionItemSnapshot> {
    let items = match response {
        lsp::CompletionResponse::Array(items) => items,
        lsp::CompletionResponse::List(list) => list.items,
    };

    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| convert_completion_item(item, index))
        .collect()
}

/// Convert a single LSP completion item to a snapshot.
fn convert_completion_item(item: lsp::CompletionItem, index: usize) -> CompletionItemSnapshot {
    let kind = item.kind.map(CompletionItemKind::from).unwrap_or_default();

    // Determine insert text: prefer text_edit, then insert_text, then label
    let insert_text = if let Some(edit) = &item.text_edit {
        match edit {
            lsp::CompletionTextEdit::Edit(edit) => edit.new_text.clone(),
            lsp::CompletionTextEdit::InsertAndReplace(edit) => edit.new_text.clone(),
        }
    } else {
        item.insert_text
            .clone()
            .unwrap_or_else(|| item.label.clone())
    };

    // Extract documentation
    let documentation = item.documentation.as_ref().map(|doc| match doc {
        lsp::Documentation::String(s) => s.clone(),
        lsp::Documentation::MarkupContent(content) => content.value.clone(),
    });

    // Check deprecated flag
    #[allow(deprecated)]
    let deprecated = item.deprecated.unwrap_or(false)
        || item
            .tags
            .as_ref()
            .is_some_and(|tags| tags.contains(&lsp::CompletionItemTag::DEPRECATED));

    CompletionItemSnapshot {
        label: item.label,
        detail: item.detail,
        kind,
        insert_text,
        documentation,
        deprecated,
        filter_text: item.filter_text,
        sort_text: item.sort_text,
        index,
    }
}

/// Convert LSP hover to a hover snapshot.
pub fn convert_hover(hover: lsp::Hover) -> HoverSnapshot {
    let contents = match hover.contents {
        lsp::HoverContents::Scalar(marked) => extract_marked_string(marked),
        lsp::HoverContents::Array(marked_strings) => marked_strings
            .into_iter()
            .map(extract_marked_string)
            .collect::<Vec<_>>()
            .join("\n\n"),
        lsp::HoverContents::Markup(content) => content.value,
    };

    // Convert range if present
    let range = hover.range.map(|r| {
        let start = r.start.line as usize;
        let end = r.end.line as usize;
        (start, end)
    });

    HoverSnapshot { contents, range }
}

/// Extract text from a MarkedString.
fn extract_marked_string(marked: MarkedString) -> String {
    match marked {
        MarkedString::String(s) => s,
        MarkedString::LanguageString(ls) => {
            // Format as a code block
            format!("```{}\n{}\n```", ls.language, ls.value)
        }
    }
}

/// Convert LSP goto definition response to location snapshots.
pub fn convert_goto_response(
    response: lsp::GotoDefinitionResponse,
    offset_encoding: OffsetEncoding,
) -> Vec<LocationSnapshot> {
    match response {
        lsp::GotoDefinitionResponse::Scalar(location) => {
            convert_location(location, offset_encoding)
                .into_iter()
                .collect()
        }
        lsp::GotoDefinitionResponse::Array(locations) => locations
            .into_iter()
            .filter_map(|loc| convert_location(loc, offset_encoding))
            .collect(),
        lsp::GotoDefinitionResponse::Link(links) => links
            .into_iter()
            .filter_map(|link| {
                let location = lsp::Location::new(link.target_uri, link.target_selection_range);
                convert_location(location, offset_encoding)
            })
            .collect(),
    }
}

/// Convert LSP references response to location snapshots.
pub fn convert_references_response(
    locations: Vec<lsp::Location>,
    offset_encoding: OffsetEncoding,
) -> Vec<LocationSnapshot> {
    locations
        .into_iter()
        .filter_map(|loc| convert_location(loc, offset_encoding))
        .collect()
}

/// Convert a single LSP location to a location snapshot.
fn convert_location(
    location: lsp::Location,
    _offset_encoding: OffsetEncoding,
) -> Option<LocationSnapshot> {
    let path = location.uri.to_file_path().ok()?;
    let line = location.range.start.line as usize + 1; // 1-indexed for display
    let column = location.range.start.character as usize + 1; // 1-indexed for display

    Some(LocationSnapshot {
        path,
        line,
        column,
        preview: None, // Preview would require reading the file
    })
}

/// Convert LSP signature help to a signature help snapshot.
pub fn convert_signature_help(help: lsp::SignatureHelp) -> SignatureHelpSnapshot {
    let signatures = help.signatures.into_iter().map(convert_signature).collect();

    SignatureHelpSnapshot {
        signatures,
        active_signature: help.active_signature.unwrap_or(0) as usize,
        active_parameter: help.active_parameter.map(|p| p as usize),
    }
}

/// Convert a single LSP signature to a signature snapshot.
fn convert_signature(sig: lsp::SignatureInformation) -> super::SignatureSnapshot {
    let documentation = sig.documentation.map(|doc| match doc {
        lsp::Documentation::String(s) => s,
        lsp::Documentation::MarkupContent(content) => content.value,
    });

    let parameters = sig
        .parameters
        .unwrap_or_default()
        .into_iter()
        .map(convert_parameter)
        .collect();

    super::SignatureSnapshot {
        label: sig.label,
        documentation,
        parameters,
    }
}

/// Convert a single LSP parameter to a parameter snapshot.
fn convert_parameter(param: lsp::ParameterInformation) -> ParameterSnapshot {
    let label = match param.label {
        lsp::ParameterLabel::Simple(s) => s,
        lsp::ParameterLabel::LabelOffsets([start, end]) => {
            format!("[{}:{}]", start, end)
        }
    };

    let documentation = param.documentation.map(|doc| match doc {
        lsp::Documentation::String(s) => s,
        lsp::Documentation::MarkupContent(content) => content.value,
    });

    ParameterSnapshot {
        label,
        documentation,
    }
}

/// Convert LSP code actions to stored code actions with full data for execution.
pub fn convert_code_actions(
    actions: Vec<lsp::CodeActionOrCommand>,
    language_server_id: helix_lsp::LanguageServerId,
    offset_encoding: OffsetEncoding,
) -> Vec<StoredCodeAction> {
    actions
        .into_iter()
        .enumerate()
        .map(|(index, action)| {
            let snapshot = convert_code_action_snapshot(&action, index);
            StoredCodeAction {
                snapshot,
                lsp_item: action,
                language_server_id,
                offset_encoding,
            }
        })
        .collect()
}

/// Convert a single LSP code action or command to a snapshot for display.
fn convert_code_action_snapshot(
    action: &lsp::CodeActionOrCommand,
    index: usize,
) -> CodeActionSnapshot {
    match action {
        lsp::CodeActionOrCommand::Command(cmd) => CodeActionSnapshot {
            title: cmd.title.clone(),
            kind: None,
            is_preferred: false,
            disabled: None,
            index,
        },
        lsp::CodeActionOrCommand::CodeAction(action) => CodeActionSnapshot {
            title: action.title.clone(),
            kind: action.kind.as_ref().map(|k| k.as_str().to_string()),
            is_preferred: action.is_preferred.unwrap_or(false),
            disabled: action.disabled.as_ref().map(|d| d.reason.clone()),
            index,
        },
    }
}

/// Convert LSP inlay hints to inlay hint snapshots.
pub fn convert_inlay_hints(
    hints: Vec<lsp::InlayHint>,
    _offset_encoding: OffsetEncoding,
) -> Vec<InlayHintSnapshot> {
    hints.into_iter().map(convert_inlay_hint).collect()
}

/// Convert a single LSP inlay hint to a snapshot.
fn convert_inlay_hint(hint: lsp::InlayHint) -> InlayHintSnapshot {
    let label = match hint.label {
        lsp::InlayHintLabel::String(s) => s,
        lsp::InlayHintLabel::LabelParts(parts) => {
            parts.into_iter().map(|p| p.value).collect::<String>()
        }
    };

    let kind = hint
        .kind
        .map(InlayHintKind::from)
        .unwrap_or(InlayHintKind::Type);

    InlayHintSnapshot {
        line: hint.position.line as usize + 1, // 1-indexed for display
        column: hint.position.character as usize,
        label,
        kind,
        padding_left: hint.padding_left.unwrap_or(false),
        padding_right: hint.padding_right.unwrap_or(false),
    }
}
