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
use std::rc::Rc;
use std::sync::mpsc;

use anyhow::Result;
use dioxus::desktop::tao::window::Icon;

mod app;
pub mod args;
mod components;
mod events;
mod keybindings;
mod lsp;
mod operations;
mod state;
pub mod tracing;

use args::StartupAction;
use state::{EditorCommand, EditorContext, EditorSnapshot};

// Thread-local storage for EditorContext to allow synchronous command processing
thread_local! {
    static EDITOR_CTX: RefCell<Option<Rc<RefCell<EditorContext>>>> = const { RefCell::new(None) };
}

/// Custom JavaScript for the webview.
const CUSTOM_SCRIPT: &str = include_str!("../assets/script.js");

/// Load the helix icon from embedded PNG.
fn load_icon() -> Option<Icon> {
    let icon_bytes = include_bytes!("../../contrib/helix.png");
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).ok()
}

/// Launch the Dioxus desktop application.
///
/// This function initializes the editor context based on the startup action,
/// sets up the Dioxus application, and starts the event loop.
pub fn launch(startup_action: StartupAction) -> Result<()> {
    // Register helix-view events with helix_event.
    // This must be done before creating handlers that register hooks for these events.
    events::register();

    // Create command channel
    let (command_tx, command_rx) = mpsc::channel::<EditorCommand>();

    // Initialize editor context based on startup action
    let (mut editor_ctx, pending_commands) = match &startup_action {
        StartupAction::None | StartupAction::OpenFilePicker => (
            EditorContext::new(None, command_rx, command_tx.clone())?,
            Vec::new(),
        ),
        StartupAction::OpenFile(path) => (
            EditorContext::new(Some(path.clone()), command_rx, command_tx.clone())?,
            Vec::new(),
        ),
        StartupAction::OpenFiles(files) => {
            // Open first file, then queue commands to open the rest
            let first = files.first().cloned();
            let rest: Vec<EditorCommand> = files
                .iter()
                .skip(1)
                .cloned()
                .map(EditorCommand::OpenFile)
                .collect();
            (
                EditorContext::new(first, command_rx, command_tx.clone())?,
                rest,
            )
        }
    };

    // Send pending commands (for glob pattern - open remaining files)
    for cmd in pending_commands {
        let _ = command_tx.send(cmd);
    }

    // Send command to show file picker if directory was specified
    if matches!(startup_action, StartupAction::OpenFilePicker) {
        let _ = command_tx.send(EditorCommand::ShowFilesRecursivePicker);
    }

    // Create initial snapshot
    let initial_snapshot = editor_ctx.snapshot(40);

    // Wrap editor context in Rc<RefCell> for single-threaded access
    let editor_ctx = Rc::new(RefCell::new(editor_ctx));

    // Store in thread-local for synchronous command processing from Dioxus components
    EDITOR_CTX.with(|ctx| {
        *ctx.borrow_mut() = Some(editor_ctx.clone());
    });

    // Create app state that can be shared with Dioxus
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
                        .with_title("helix-dioxus")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                        .with_window_icon(load_icon()),
                )
                .with_custom_head(format!("<script>{CUSTOM_SCRIPT}</script>"))
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

    /// Process pending commands and update the snapshot synchronously.
    /// This should be called after sending commands but before triggering a re-render.
    pub fn process_commands_sync(&self) {
        EDITOR_CTX.with(|ctx| {
            if let Some(ref editor_ctx) = *ctx.borrow() {
                if let Ok(mut editor) = editor_ctx.try_borrow_mut() {
                    editor.process_commands();
                    let new_snapshot = editor.snapshot(40);

                    // Check if we should quit
                    if new_snapshot.should_quit {
                        std::process::exit(0);
                    }

                    *self.snapshot.lock() = new_snapshot;
                }
            }
        });
    }

    /// Get the current snapshot.
    pub fn get_snapshot(&self) -> EditorSnapshot {
        self.snapshot.lock().clone()
    }
}
