//! Entry point for the dhx binary.

mod args;
mod tracing_setup;

use anyhow::Result;

#[allow(clippy::print_stderr)] // pre-logging: tracing not yet initialized
fn main() -> Result<()> {
    // Parse arguments first so --help/--version exit before any initialization.
    let parsed = args::parse_args();

    // Load GUI-specific config (dhx.toml), then apply CLI overrides on top.
    let config = helix_dioxus::DhxConfig::load_default().unwrap_or_else(|err| {
        eprintln!("Warning: failed to load dhx.toml: {err}");
        eprintln!("Using default configuration");
        helix_dioxus::DhxConfig::default()
    });
    let config = parsed.overrides.apply(config);

    // Set up tracing subscriber BEFORE Dioxus to prevent dioxus-logger from setting its own.
    tracing_setup::init(&config.logging);

    log::info!("Starting dhx");

    // Initialize helix runtime (grammars, queries, themes)
    helix_loader::initialize_config_file(None);
    helix_loader::initialize_log_file(None);

    // Create tokio runtime for helix async operations
    let runtime = tokio::runtime::Runtime::new()?;
    let _guard = runtime.enter();

    helix_dioxus::launch(config, parsed.startup_action)
}
