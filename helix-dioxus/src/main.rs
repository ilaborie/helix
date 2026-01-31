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
use dioxus::desktop::tao::window::Icon;

// Thread-local storage for EditorContext to allow synchronous command processing
thread_local! {
    static EDITOR_CTX: RefCell<Option<Rc<RefCell<EditorContext>>>> = const { RefCell::new(None) };
}

mod app;
mod buffer_bar;
mod editor_view;
mod input;
mod picker;
mod prompt;
mod state;
mod statusline;
mod tracing;

use crate::state::{EditorCommand, EditorContext, EditorSnapshot};

/// Load the helix icon from embedded PNG.
fn load_icon() -> Option<Icon> {
    let icon_bytes = include_bytes!("../../contrib/helix.png");
    let image = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).ok()
}

/// Custom HTML head content with CSS styles.
const CUSTOM_HEAD: &str = include_str!("../assets/head.html");

/// Determines what action to take based on command line argument.
enum StartupAction {
    /// No argument provided - open scratch buffer.
    None,
    /// Single file to open.
    OpenFile(PathBuf),
    /// Multiple files to open (from glob pattern).
    OpenFiles(Vec<PathBuf>),
    /// Directory argument - open file picker in that directory.
    OpenFilePicker,
}

fn main() -> Result<()> {
    // Set up tracing subscriber BEFORE Dioxus to prevent dioxus-logger from setting its own.
    // This uses a custom filter to suppress noisy webview messages.
    // The tracing crate has log compatibility, so log::info! etc. will work.
    tracing::init();

    log::info!("Starting helix-dioxus");

    // Initialize helix runtime (grammars, queries, themes)
    helix_loader::initialize_config_file(None);
    helix_loader::initialize_log_file(None);

    // Create tokio runtime for helix async operations (required by word_index::Handler::spawn())
    let runtime = tokio::runtime::Runtime::new()?;
    let _guard = runtime.enter();

    // Get file(s) to open from command line args
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Determine what to do with the arguments
    let startup_action = if args.is_empty() {
        StartupAction::None
    } else if args.len() > 1 {
        // Multiple arguments (shell-expanded glob or multiple files)
        let files: Vec<PathBuf> = args
            .iter()
            .map(PathBuf::from)
            .filter(|path| path.is_file())
            .collect();

        if files.is_empty() {
            log::warn!("No valid files in arguments");
            StartupAction::None
        } else {
            log::info!("Opening {} files", files.len());
            StartupAction::OpenFiles(files)
        }
    } else {
        // Single argument
        let path_str = &args[0];

        // Check if it's a glob pattern (contains * or ?) - for shells that don't expand globs
        if path_str.contains('*') || path_str.contains('?') {
            let files: Vec<PathBuf> = glob::glob(path_str)
                .ok()
                .map(|paths| {
                    paths
                        .filter_map(Result::ok)
                        .filter(|path| path.is_file())
                        .collect()
                })
                .unwrap_or_default();

            if files.is_empty() {
                log::warn!("No files match pattern: {path_str}");
                StartupAction::None
            } else {
                log::info!("Opening {} files from glob pattern", files.len());
                StartupAction::OpenFiles(files)
            }
        } else {
            let path = PathBuf::from(path_str);
            if path.is_dir() {
                // Change to directory and open file picker
                if std::env::set_current_dir(&path).is_ok() {
                    log::info!("Changed to directory: {}", path.display());
                    StartupAction::OpenFilePicker
                } else {
                    log::error!("Cannot change to directory: {}", path.display());
                    StartupAction::None
                }
            } else {
                StartupAction::OpenFile(path)
            }
        }
    };

    // Create command channel
    let (command_tx, command_rx) = mpsc::channel::<EditorCommand>();

    // Initialize editor context based on startup action
    let (mut editor_ctx, pending_commands) = match &startup_action {
        StartupAction::None | StartupAction::OpenFilePicker => {
            (EditorContext::new(None, command_rx)?, Vec::new())
        }
        StartupAction::OpenFile(path) => (
            EditorContext::new(Some(path.clone()), command_rx)?,
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
            (EditorContext::new(first, command_rx)?, rest)
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
                        .with_title("helix-dioxus")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0))
                        .with_window_icon(load_icon()),
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
