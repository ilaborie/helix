//! Location picker component.
//!
//! Displays a picker for selecting from multiple goto locations.

use dioxus::prelude::*;

use crate::lsp::LocationSnapshot;

/// Location picker that displays multiple goto results.
#[component]
pub fn LocationPicker(title: String, locations: Vec<LocationSnapshot>, selected: usize) -> Element {
    rsx! {
        div {
            class: "location-picker-overlay",

            div {
                class: "location-picker-container",

                // Header
                div {
                    class: "location-picker-header",
                    "{title}"
                    span {
                        style: "color: #5c6370; margin-left: 8px;",
                        "({locations.len()} locations)"
                    }
                }

                // Location list
                div {
                    class: "location-picker-list",

                    for (idx, loc) in locations.iter().enumerate() {
                        {
                            let is_selected = idx == selected;
                            let item_class = if is_selected {
                                "location-item location-item-selected"
                            } else {
                                "location-item"
                            };

                            let file_name = loc.path.file_name()
                                .map_or_else(
                                    || loc.path.to_string_lossy().to_string(),
                                    |name| name.to_string_lossy().to_string(),
                                );

                            let dir_path = loc.path.parent()
                                .map_or_else(String::new, |parent| parent.to_string_lossy().to_string());

                            rsx! {
                                div {
                                    key: "{idx}",
                                    class: "{item_class}",

                                    // File path and position
                                    div {
                                        span {
                                            class: "location-path",
                                            "{file_name}"
                                        }
                                        span {
                                            class: "location-position",
                                            ":{loc.line}:{loc.column}"
                                        }
                                        if !dir_path.is_empty() {
                                            span {
                                                style: "color: #5c6370; font-size: 11px; margin-left: 8px;",
                                                "{dir_path}"
                                            }
                                        }
                                    }

                                    // Preview (if available)
                                    if let Some(ref preview) = loc.preview {
                                        div {
                                            class: "location-preview",
                                            "{preview}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Empty state
                    if locations.is_empty() {
                        div {
                            style: "padding: 16px; color: #5c6370; text-align: center;",
                            "No locations found"
                        }
                    }
                }
            }
        }
    }
}
