//! Reusable modal overlay component.
//!
//! Encapsulates the overlay+backdrop+container pattern shared by all modal dialogs
//! (confirmation, LSP status, pickers, location picker).

use dioxus::prelude::*;

/// Modal overlay that provides a backdrop and centered container.
///
/// Clicking the backdrop triggers `on_backdrop_click`. Clicks inside the
/// container are stopped from propagating to the backdrop.
///
/// Set `align_top` to `true` for picker-style layout (top-aligned with padding).
#[component]
pub fn ModalOverlay(
    class: Option<&'static str>,
    z_index: Option<&'static str>,
    align_top: Option<bool>,
    on_backdrop_click: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    let z = z_index.unwrap_or("--z-dropdown");
    let container_class = match class {
        Some(c) => format!("modal-container {c}"),
        None => "modal-container".to_string(),
    };
    let overlay_class = if align_top.unwrap_or(false) {
        "modal-overlay modal-overlay-top"
    } else {
        "modal-overlay"
    };

    rsx! {
        div {
            class: "{overlay_class}",
            style: "z-index: var({z});",
            onmousedown: move |evt| on_backdrop_click.call(evt),

            div {
                class: "{container_class}",
                onmousedown: move |evt| evt.stop_propagation(),
                {children}
            }
        }
    }
}
