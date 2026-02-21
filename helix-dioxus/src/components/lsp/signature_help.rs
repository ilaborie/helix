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
    let active_idx = active_param.unwrap_or(0);

    if let Some((start, end)) = parameters
        .get(active_idx)
        .and_then(|p| p.label_range)
        .filter(|&(s, e)| s <= e && e <= label.len())
    {
        let before = &label[..start];
        let highlighted = &label[start..end];
        let after = &label[end..];

        return rsx! {
            span { "{before}" }
            span {
                class: "signature-param-active",
                "{highlighted}"
            }
            span { "{after}" }
        };
    }

    // Fallback: no active parameter or range out of bounds
    rsx! { span { "{label}" } }
}

/// Signature help popup that displays function signatures.
#[component]
pub fn SignatureHelpPopup(signature_help: SignatureHelpSnapshot) -> Element {
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
            position: DialogPosition::Above,
            class: "signature-help-popup",
            constraints,

            if let Some(sig) = active_sig {
                // Signature label with highlighted active parameter
                div {
                    class: "signature-label",
                    {render_signature_label(&sig.label, &sig.parameters, signature_help.active_parameter)}
                }

                // Documentation (rendered as markdown)
                if let Some(ref docs) = sig.documentation {
                    div {
                        class: "signature-docs",
                        dangerous_inner_html: super::markdown::markdown_to_html(docs, None),
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
