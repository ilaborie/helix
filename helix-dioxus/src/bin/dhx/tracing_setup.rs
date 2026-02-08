//! Tracing configuration for the dhx binary.
//!
//! This module sets up the tracing subscriber with custom filtering to suppress
//! noisy webview events like `SelectionDidChange` that pollute the console output.
//!
//! Must be initialized BEFORE Dioxus launch to prevent dioxus-logger from
//! setting its own subscriber.

use std::fs::File;
use std::io;
use std::sync::Mutex;

use helix_dioxus::config::LoggingConfig;
use tracing::{Event, Subscriber};
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{self, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

/// Custom event formatter that filters out messages containing suppressed patterns.
struct FilteringFormatter {
    inner: fmt::format::Format,
    suppressed_patterns: Vec<String>,
}

impl FilteringFormatter {
    fn new(suppressed_patterns: Vec<String>) -> Self {
        Self {
            inner: fmt::format::Format::default(),
            suppressed_patterns,
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
        let should_suppress = self
            .suppressed_patterns
            .iter()
            .any(|pattern| message_buf.contains(pattern.as_str()));

        if should_suppress {
            // Don't write anything - effectively suppresses the message
            Ok(())
        } else {
            // Write the formatted message
            write!(writer, "{message_buf}")
        }
    }
}

/// Initialize the tracing subscriber with configuration from `LoggingConfig`.
///
/// This sets up:
/// - Environment-based filtering via `RUST_LOG` (defaults to configured level)
/// - Custom message filtering to suppress noisy webview events
/// - Output to configured log file (falls back to stderr)
///
/// # Panics
///
/// Panics if a global subscriber has already been set.
pub fn init(config: &LoggingConfig) {
    // Create a base filter from RUST_LOG env var, defaulting to configured level
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let suppressed = config.suppressed_patterns.clone();

    // Try to create log file, fall back to stderr if it fails
    let log_file = config
        .log_file
        .as_ref()
        .and_then(|path| File::create(path).ok());

    if let Some(log_file) = log_file {
        let path_display = config
            .log_file
            .as_ref()
            .map_or_else(String::new, |p| p.display().to_string());

        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .with_writer(Mutex::new(log_file))
            .event_format(FilteringFormatter::new(suppressed));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();

        eprintln!("Logging to {path_display}");
    } else {
        // Fall back to stderr
        let fmt_layer = fmt::layer()
            .with_target(false)
            .with_writer(io::stderr)
            .event_format(FilteringFormatter::new(suppressed));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }
}
