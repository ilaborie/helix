//! Reusable keyboard key component.
//!
//! Renders a `<kbd>` element with physical key styling.

use dioxus::prelude::*;

/// A styled keyboard key element.
///
/// Use `class: "kbd-key-compact"` for the 20px-tall help bar variant.
#[component]
pub fn KbdKey(label: &'static str, #[props(default)] class: &'static str) -> Element {
    if class.is_empty() {
        rsx! { kbd { "{label}" } }
    } else {
        rsx! { kbd { class: "{class}", "{label}" } }
    }
}
