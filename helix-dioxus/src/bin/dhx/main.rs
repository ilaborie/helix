//! Entry point for the dhx binary.

mod args;
mod tracing_setup;

use anyhow::Result;

fn main() -> Result<()> {
    // Load GUI-specific config (dhx.toml)
    let config = helix_dioxus::DhxConfig::load_default().unwrap_or_else(|err| {
        eprintln!("Warning: failed to load dhx.toml: {err}");
        eprintln!("Using default configuration");
        helix_dioxus::DhxConfig::default()
    });

    // Set up tracing subscriber BEFORE Dioxus to prevent dioxus-logger from setting its own.
    tracing_setup::init(&config.logging);

    log::info!("Starting dhx");

    // Initialize helix runtime (grammars, queries, themes)
    helix_loader::initialize_config_file(None);
    helix_loader::initialize_log_file(None);

    // Create tokio runtime for helix async operations
    let runtime = tokio::runtime::Runtime::new()?;
    let _guard = runtime.enter();

    // Parse command-line arguments and launch the application
    let startup_action = args::parse_args();
    helix_dioxus::launch(config, startup_action)
}
