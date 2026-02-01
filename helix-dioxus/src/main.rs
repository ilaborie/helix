//! Helix Dioxus entry point.

use anyhow::Result;

fn main() -> Result<()> {
    // Set up tracing subscriber BEFORE Dioxus to prevent dioxus-logger from setting its own.
    helix_dioxus::tracing::init();

    log::info!("Starting helix-dioxus");

    // Initialize helix runtime (grammars, queries, themes)
    helix_loader::initialize_config_file(None);
    helix_loader::initialize_log_file(None);

    // Create tokio runtime for helix async operations
    let runtime = tokio::runtime::Runtime::new()?;
    let _guard = runtime.enter();

    // Parse command-line arguments and launch the application
    let startup_action = helix_dioxus::args::parse_args();
    helix_dioxus::launch(startup_action)
}
