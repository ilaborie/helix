//! Notification toast component.
//!
//! Displays toast notifications in the bottom-right corner of the editor.

use dioxus::prelude::*;

use crate::state::{NotificationSeverity, NotificationSnapshot};
use crate::AppState;

/// Container for notification toasts.
/// Renders in bottom-right corner, above the statusline.
#[component]
pub fn NotificationContainer(notifications: Vec<NotificationSnapshot>) -> Element {
    let app_state = use_context::<AppState>();

    if notifications.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "notification-container",

            for notification in notifications.iter().rev() {
                NotificationToast {
                    key: "{notification.id}",
                    notification: notification.clone(),
                    on_dismiss: {
                        let app_state = app_state.clone();
                        let id = notification.id;
                        move |()| {
                            app_state.send_command(crate::state::EditorCommand::DismissNotification(id));
                        }
                    },
                }
            }
        }
    }
}

/// A single notification toast.
#[component]
fn NotificationToast(notification: NotificationSnapshot, on_dismiss: EventHandler<()>) -> Element {
    let severity_class = match notification.severity {
        NotificationSeverity::Error => "notification-error",
        NotificationSeverity::Warning => "notification-warning",
        NotificationSeverity::Info => "notification-info",
        NotificationSeverity::Success => "notification-success",
    };

    let icon = match notification.severity {
        NotificationSeverity::Error => rsx! {
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                // X Circle icon
                circle { cx: "12", cy: "12", r: "10" }
                line { x1: "15", y1: "9", x2: "9", y2: "15" }
                line { x1: "9", y1: "9", x2: "15", y2: "15" }
            }
        },
        NotificationSeverity::Warning => rsx! {
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                // Alert Triangle icon
                path { d: "M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" }
                line { x1: "12", y1: "9", x2: "12", y2: "13" }
                line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
            }
        },
        NotificationSeverity::Info => rsx! {
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                // Info icon
                circle { cx: "12", cy: "12", r: "10" }
                line { x1: "12", y1: "16", x2: "12", y2: "12" }
                line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
            }
        },
        NotificationSeverity::Success => rsx! {
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                width: "16",
                height: "16",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                // Check Circle icon
                path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
                polyline { points: "22 4 12 14.01 9 11.01" }
            }
        },
    };

    rsx! {
        div {
            class: "notification-toast {severity_class}",
            onclick: move |_| on_dismiss.call(()),

            div {
                class: "notification-icon",
                {icon}
            }

            div {
                class: "notification-message",
                "{notification.message}"
            }

            button {
                class: "notification-close",
                onclick: move |e| {
                    e.stop_propagation();
                    on_dismiss.call(());
                },

                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    width: "14",
                    height: "14",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    // X icon
                    line { x1: "18", y1: "6", x2: "6", y2: "18" }
                    line { x1: "6", y1: "6", x2: "18", y2: "18" }
                }
            }
        }
    }
}
