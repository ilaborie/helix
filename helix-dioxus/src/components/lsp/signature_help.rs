//! Signature help popup component.
//!
//! Displays function signature and parameter information.

use dioxus::prelude::*;

use crate::components::inline_dialog::{DialogConstraints, DialogPosition, InlineDialogContainer};
use crate::lsp::SignatureHelpSnapshot;

/// Render the signature label with the active parameter highlighted.
fn render_signature_label(
    label: &str,
    parameters: &[crate::lsp::ParameterSnapshot],
    active_param: Option<usize>,
) -> Element {
    // For now, simple rendering - highlight by wrapping active param
    // TODO: Parse the label to find parameter positions

    let active_idx = active_param.unwrap_or(0);

    if parameters.is_empty() {
        return rsx! { span { "{label}" } };
    }

    // Try to find and highlight the active parameter in the label
    if let Some(param) = parameters.get(active_idx) {
        if let Some(pos) = label.find(&param.label) {
            let before = &label[..pos];
            let param_text = &param.label;
            let after = &label[pos + param.label.len()..];

            return rsx! {
                span { "{before}" }
                span {
                    class: "signature-param-active",
                    "{param_text}"
                }
                span { "{after}" }
            };
        }
    }

    // Fallback: just render the label
    rsx! { span { "{label}" } }
}

/// Signature help popup that displays function signatures.
#[component]
pub fn SignatureHelpPopup(
    signature_help: SignatureHelpSnapshot,
    cursor_line: usize,
    cursor_col: usize,
) -> Element {
    let constraints = DialogConstraints {
        min_width: None,
        max_width: Some(600),
        max_height: None,
    };

    // Get the active signature
    let active_sig = signature_help
        .signatures
        .get(signature_help.active_signature)
        .or_else(|| signature_help.signatures.first());

    rsx! {
        InlineDialogContainer {
            cursor_line,
            cursor_col,
            position: DialogPosition::Above,
            class: "signature-help-popup",
            constraints,

            if let Some(sig) = active_sig {
                // Signature label with highlighted active parameter
                div {
                    class: "signature-label",
                    {render_signature_label(&sig.label, &sig.parameters, signature_help.active_parameter)}
                }

                // Documentation
                if let Some(ref docs) = sig.documentation {
                    div {
                        class: "signature-docs",
                        "{docs}"
                    }
                }

                // Show signature index if multiple signatures
                if signature_help.signatures.len() > 1 {
                    div {
                        style: "color: #5c6370; font-size: 11px; margin-top: 4px;",
                        "Signature {signature_help.active_signature + 1} of {signature_help.signatures.len()}"
                    }
                }
            }
        }
    }
}
