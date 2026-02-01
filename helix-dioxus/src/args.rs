//! Command-line argument parsing.

use std::path::PathBuf;

/// Determines what action to take based on command line arguments.
#[derive(Debug, Clone)]
pub enum StartupAction {
    /// No argument provided - open scratch buffer.
    None,
    /// Single file to open.
    OpenFile(PathBuf),
    /// Multiple files to open (from glob pattern or multiple args).
    OpenFiles(Vec<PathBuf>),
    /// Directory argument - open file picker in that directory.
    OpenFilePicker,
}

/// Parse command-line arguments and determine the startup action.
pub fn parse_args() -> StartupAction {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return StartupAction::None;
    }

    if args.len() > 1 {
        // Multiple arguments (shell-expanded glob or multiple files)
        return parse_multiple_args(&args);
    }

    // Single argument
    parse_single_arg(&args[0])
}

/// Parse multiple command-line arguments.
fn parse_multiple_args(args: &[String]) -> StartupAction {
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
}

/// Parse a single command-line argument.
fn parse_single_arg(arg: &str) -> StartupAction {
    // Check if it's a glob pattern (contains * or ?) - for shells that don't expand globs
    if arg.contains('*') || arg.contains('?') {
        return parse_glob_pattern(arg);
    }

    let path = PathBuf::from(arg);

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

/// Parse a glob pattern argument.
fn parse_glob_pattern(pattern: &str) -> StartupAction {
    let files: Vec<PathBuf> = glob::glob(pattern)
        .ok()
        .map(|paths| {
            paths
                .filter_map(Result::ok)
                .filter(|path| path.is_file())
                .collect()
        })
        .unwrap_or_default();

    if files.is_empty() {
        log::warn!("No files match pattern: {pattern}");
        StartupAction::None
    } else {
        log::info!("Opening {} files from glob pattern", files.len());
        StartupAction::OpenFiles(files)
    }
}
