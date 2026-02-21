//! Reusable popup menu component.
//!
//! A generic context menu positioned at screen coordinates with a backdrop
//! for click-to-dismiss behavior. Used by `BufferBar` for tab context menus.

use dioxus::prelude::*;

/// A single entry in a popup menu.
#[derive(Clone)]
pub enum PopupMenuEntry {
    Item {
        label: &'static str,
        disabled: bool,
        on_click: EventHandler<MouseEvent>,
    },
    Separator,
}

// Always return false so Dioxus re-renders when the menu is rebuilt.
impl PartialEq for PopupMenuEntry {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

/// Popup menu positioned at screen coordinates with a click-to-dismiss backdrop.
#[component]
pub fn PopupMenu(x: f64, y: f64, entries: Vec<PopupMenuEntry>, on_close: EventHandler) -> Element {
    rsx! {
        div {
            class: "context-menu-backdrop",
            onmousedown: move |_| on_close.call(()),
        }
        div {
            class: "context-menu",
            style: "left: {x}px; top: {y}px;",
            for entry in entries.iter() {
                match entry {
                    PopupMenuEntry::Separator => rsx! {
                        div { class: "context-menu-separator" }
                    },
                    PopupMenuEntry::Item { label, disabled, on_click } => {
                        let disabled = *disabled;
                        let on_click = *on_click;
                        let cls = if disabled {
                            "context-menu-item context-menu-item-disabled"
                        } else {
                            "context-menu-item"
                        };
                        rsx! {
                            div {
                                class: "{cls}",
                                onmousedown: move |evt: Event<MouseData>| {
                                    evt.stop_propagation();
                                    if !disabled {
                                        on_click.call(evt);
                                    }
                                    on_close.call(());
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
