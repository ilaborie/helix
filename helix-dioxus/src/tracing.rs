//! Tracing configuration for helix-dioxus.
//!
//! This module sets up the tracing subscriber with custom filtering to suppress
//! noisy webview events like `SelectionDidChange` that pollute the console output.
//!
//! Must be initialized BEFORE Dioxus launch to prevent dioxus-logger from
//! setting its own subscriber.

use std::fs::File;
use std::io;
use std::sync::Mutex;

use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{self, format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Patterns to filter out from log output.
/// These are substrings that, if found in a log message, will cause it to be suppressed.
const SUPPRESSED_PATTERNS: &[&str] = &[
    "SelectionDidChange",
    "Dispatched unknown event",
    "mousemove",
    "mouseenter",
    "mouseleave",
    "pointermove",
    "pointerenter",
    "pointerleave",
    // Add more patterns here as needed
];

/// Log file path
const LOG_FILE: &str = "/tmp/helix-dioxus.log";

/// Custom event formatter that filters out messages containing suppressed patterns.
struct FilteringFormatter {
    inner: fmt::format::Format,
}

impl FilteringFormatter {
    fn new() -> Self {
        Self {
            inner: fmt::format::Format::default(),
        }
    }
}

impl<S, N> FormatEvent<S, N> for FilteringFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        // Capture the formatted message to check for suppressed patterns
        let mut message_buf = String::new();
        let capture_writer = Writer::new(&mut message_buf);

        // Format using inner formatter to capture the message
        self.inner.format_event(ctx, capture_writer, event)?;

        // Check if message contains any suppressed patterns
        let should_suppress = SUPPRESSED_PATTERNS
            .iter()
            .any(|pattern| message_buf.contains(pattern));

        if should_suppress {
            // Don't write anything - effectively suppresses the message
            Ok(())
        } else {
            // Write the formatted message
            write!(writer, "{message_buf}")
        }
    }
}

/// Initialize the tracing subscriber with custom filtering.
///
/// This sets up:
/// - Environment-based filtering via `RUST_LOG` (defaults to `info`)
/// - Custom message filtering to suppress noisy webview events
/// - Output to file at `/tmp/helix-dioxus.log`
///
/// # Panics
///
/// Panics if a global subscriber has already been set.
pub fn init() {
    // Create a base filter from RUST_LOG env var, defaulting to info level
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Try to create log file, fall back to stderr if it fails
    if let Ok(log_file) = File::create(LOG_FILE) {
        // Log to file
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .with_writer(Mutex::new(log_file))
            .event_format(FilteringFormatter::new());

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();

        eprintln!("Logging to {LOG_FILE}");
    } else {
        // Fall back to stderr
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_writer(io::stderr)
            .event_format(FilteringFormatter::new());

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suppressed_patterns_contains_selection_did_change() {
        assert!(SUPPRESSED_PATTERNS.contains(&"SelectionDidChange"));
    }

    #[test]
    fn suppressed_patterns_contains_dispatched_unknown_event() {
        assert!(SUPPRESSED_PATTERNS.contains(&"Dispatched unknown event"));
    }
}
