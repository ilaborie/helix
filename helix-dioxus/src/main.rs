//! Helix Dioxus - A GUI frontend for Helix editor
//!
//! This crate provides a Dioxus-based desktop GUI for the Helix text editor,
//! reusing helix-core and helix-view for the editing engine.
//!
//! ## Architecture
//!
//! Since `helix_view::Editor` contains non-Send/Sync types (Cell, trait objects, etc.),
//! we cannot share it directly via Dioxus context. Instead:
//!
//! 1. EditorContext lives on the main thread and is never shared
//! 2. We create snapshots of editor state for rendering
//! 3. Commands are sent via channels and processed on the main thread
//! 4. The Dioxus app runs in a single-threaded context

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::mpsc;

use anyhow::Result;

mod app;
mod editor_view;
mod input;
mod picker;
mod prompt;
mod state;
mod statusline;

use crate::state::{EditorCommand, EditorContext, EditorSnapshot};

/// Custom HTML head content with CSS styles.
const CUSTOM_HEAD: &str = include_str!("../assets/head.html");

fn main() -> Result<()> {
    // Initialize logging
    setup_logging()?;

    log::info!("Starting helix-dioxus");

    // Initialize helix runtime (grammars, queries, themes)
    helix_loader::initialize_config_file(None);
    helix_loader::initialize_log_file(None);

    // Create tokio runtime for helix async operations (required by word_index::Handler::spawn())
    let runtime = tokio::runtime::Runtime::new()?;
    let _guard = runtime.enter();

    // Get file to open from command line args
    let args: Vec<String> = std::env::args().collect();
    let file_to_open = args.get(1).map(PathBuf::from);

    // Create command channel
    let (command_tx, command_rx) = mpsc::channel::<EditorCommand>();

    // Initialize editor context
    let editor_ctx = EditorContext::new(file_to_open, command_rx)?;

    // Create initial snapshot
    let initial_snapshot = editor_ctx.snapshot(40);

    // Wrap editor context in Rc<RefCell> for single-threaded access
    let editor_ctx = Rc::new(RefCell::new(editor_ctx));

    // Create app state that can be shared with Dioxus
    // Note: AppState is Clone + Send + Sync because it only contains the command sender and snapshot
    let app_state = AppState {
        command_tx,
        snapshot: std::sync::Arc::new(parking_lot::Mutex::new(initial_snapshot)),
    };

    // Clone for the closure
    let editor_ctx_clone = editor_ctx.clone();
    let snapshot_ref = app_state.snapshot.clone();

    // Launch Dioxus desktop app
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Helix")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0)),
                )
                .with_custom_head(CUSTOM_HEAD.to_string())
                .with_custom_event_handler(move |_event, _target| {
                    // Process commands on each event loop iteration
                    if let Ok(mut ctx) = editor_ctx_clone.try_borrow_mut() {
                        ctx.process_commands();
                        let new_snapshot = ctx.snapshot(40);

                        // Check if we should quit
                        if new_snapshot.should_quit {
                            std::process::exit(0);
                        }

                        *snapshot_ref.lock() = new_snapshot;
                    }
                }),
        )
        .with_context(app_state)
        .launch(app::App);

    Ok(())
}

/// Application state that can be shared with Dioxus.
/// This is Clone + Send + Sync because it only contains thread-safe types.
#[derive(Clone)]
pub struct AppState {
    pub command_tx: mpsc::Sender<EditorCommand>,
    pub snapshot: std::sync::Arc<parking_lot::Mutex<EditorSnapshot>>,
}

impl AppState {
    /// Send a command to the editor.
    pub fn send_command(&self, cmd: EditorCommand) {
        let _ = self.command_tx.send(cmd);
    }

    /// Get the current snapshot.
    pub fn get_snapshot(&self) -> EditorSnapshot {
        self.snapshot.lock().clone()
    }
}

fn setup_logging() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            // Filter out noisy Dioxus webview "SelectionDidChange" errors
            let msg = message.to_string();
            if msg.contains("SelectionDidChange") {
                return;
            }
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                msg
            ));
        })
        .level(log::LevelFilter::Info)
        .chain(std::io::stderr())
        .apply()?;
    Ok(())
}
