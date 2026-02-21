//! Scrollbar for the picker item list.

use dioxus::prelude::*;

use crate::state::scrollbar_thumb_geometry;

/// Thin scrollbar track + thumb indicating scroll position in the picker list.
///
/// Returns nothing when the full list fits within the visible window.
#[component]
pub fn PickerScrollbar(visible_count: usize, window_offset: usize, filtered_count: usize) -> Element {
    let Some((top_pct, height_pct)) = scrollbar_thumb_geometry(visible_count, window_offset, filtered_count) else {
        return rsx! {};
    };

    rsx! {
        div {
            class: "picker-scrollbar-track",
            div {
                class: "picker-scrollbar-thumb",
                style: "top: {top_pct:.2}%; height: {height_pct:.2}%;",
            }
        }
    }
}
