//! Command-line argument parsing for the dhx binary.

use std::path::{Path, PathBuf};

use clap::Parser;
use helix_core::Position;
use helix_dioxus::{DhxConfig, StartupAction};

#[derive(Parser)]
#[command(
    name = "dhx",
    about = "Dioxus GUI frontend for the Helix text editor",
    long_about = "dhx opens files, directories, or glob patterns.\n\n\
        File arguments support optional line/column suffixes:\n\
        file.rs           — open at the beginning\n\
        file.rs:42        — open at line 42\n\
        file.rs:42:5      — open at line 42, column 5\n\n\
        Passing a directory opens the file picker inside it.\n\
        Passing a glob pattern (e.g. \"src/*.rs\") opens all matching files.",
    version
)]
struct Args {
    /// Files, directories, or glob patterns to open.
    /// Supports `file:row` and `file:row:col` syntax.
    #[arg(value_name = "FILE")]
    files: Vec<String>,

    /// Override the color theme (e.g. gruvbox, catppuccin-mocha).
    #[arg(long, value_name = "THEME")]
    theme: Option<String>,

    /// Override the log file path.
    #[arg(long, value_name = "FILE")]
    log: Option<PathBuf>,

    /// Set log level: error, warn, info, debug, trace. Overrides dhx.toml.
    #[arg(long, value_name = "LEVEL")]
    log_level: Option<String>,

    /// Enable verbose (debug) logging. Shorthand for --log-level debug.
    #[arg(short, long)]
    verbose: bool,

    /// Override the font size in pixels.
    #[arg(long, value_name = "SIZE")]
    font_size: Option<f64>,
}

/// Overrides parsed from CLI flags, applied on top of the loaded `DhxConfig`.
pub struct CliOverrides {
    pub theme: Option<String>,
    pub log: Option<PathBuf>,
    pub log_level: Option<String>,
    pub verbose: bool,
    pub font_size: Option<f64>,
}

impl CliOverrides {
    /// Apply CLI overrides on top of a loaded [`DhxConfig`], returning the patched config.
    pub fn apply(self, mut config: DhxConfig) -> DhxConfig {
        if let Some(log_file) = self.log {
            config.logging.log_file = Some(log_file);
        }
        // --verbose sets debug level; --log-level takes precedence if both given
        if self.verbose {
            config.logging.level = "debug".to_string();
        }
        if let Some(level) = self.log_level {
            config.logging.level = level;
        }
        if let Some(theme) = self.theme {
            config.initial_theme = Some(theme);
        }
        if let Some(size) = self.font_size {
            config.font.size = size;
        }
        config
    }
}

/// Result of argument parsing: startup file action plus config overrides.
pub struct ParsedArgs {
    pub startup_action: StartupAction,
    pub overrides: CliOverrides,
}

/// Parse command-line arguments and determine the startup action.
pub fn parse_args() -> ParsedArgs {
    let args = Args::parse();

    let startup_action = match args.files.as_slice() {
        [] => StartupAction::None,
        [arg] => parse_single_arg(arg),
        files => parse_multiple_args(files),
    };

    let overrides = CliOverrides {
        theme: args.theme,
        log: args.log,
        log_level: args.log_level,
        verbose: args.verbose,
        font_size: args.font_size,
    };

    ParsedArgs {
        startup_action,
        overrides,
    }
}

/// Parse multiple command-line arguments.
fn parse_multiple_args(args: &[String]) -> StartupAction {
    let files: Vec<(PathBuf, Position)> = args.iter().map(|a| parse_file(a)).collect();

    // Keep existing files and non-existent paths (new buffers), but skip directories.
    let files: Vec<(PathBuf, Position)> = files
        .into_iter()
        .filter(|(path, _)| !path.exists() || path.is_file())
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

    let (path, pos) = parse_file(arg);

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
        StartupAction::OpenFile(path, pos)
    }
}

/// Parse a glob pattern argument.
fn parse_glob_pattern(pattern: &str) -> StartupAction {
    let files: Vec<(PathBuf, Position)> = glob::glob(pattern)
        .ok()
        .map(|paths| {
            paths
                .filter_map(Result::ok)
                .filter(|path| path.is_file())
                .map(|path| (path, Position::default()))
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

/// Parse a file argument into a [`PathBuf`] and [`Position`].
///
/// Supports `file`, `file:row`, and `file:row:col` syntax.
/// If the path exists as-is on disk, returns it with a default position.
/// Otherwise, tries to split off `:row:col` or `:row` from the end.
fn parse_file(s: &str) -> (PathBuf, Position) {
    let def = || (PathBuf::from(s), Position::default());
    if Path::new(s).exists() {
        return def();
    }
    split_path_row_col(s).or_else(|| split_path_row(s)).unwrap_or_else(def)
}

/// Split `file.rs:10:2` into [`PathBuf`], row and col.
fn split_path_row_col(s: &str) -> Option<(PathBuf, Position)> {
    let mut s = s.trim_end_matches(':').rsplitn(3, ':');
    let col: usize = s.next()?.parse().ok()?;
    let row: usize = s.next()?.parse().ok()?;
    let path = s.next()?.into();
    let pos = Position::new(row.saturating_sub(1), col.saturating_sub(1));
    Some((path, pos))
}

/// Split `file.rs:10` into [`PathBuf`] and row.
fn split_path_row(s: &str) -> Option<(PathBuf, Position)> {
    let (path, row) = s.trim_end_matches(':').rsplit_once(':')?;
    let row: usize = row.parse().ok()?;
    let path = path.into();
    let pos = Position::new(row.saturating_sub(1), 0);
    Some((path, pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- split_path_row_col ---

    #[test]
    fn split_path_row_col_valid() {
        let result = split_path_row_col("file.rs:10:5");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(9, 4))));
    }

    #[test]
    fn split_path_row_col_trailing_colon() {
        let result = split_path_row_col("file.rs:10:5:");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(9, 4))));
    }

    #[test]
    fn split_path_row_col_row_one_col_one() {
        let result = split_path_row_col("file.rs:1:1");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(0, 0))));
    }

    #[test]
    fn split_path_row_col_row_zero() {
        let result = split_path_row_col("file.rs:0:0");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(0, 0))));
    }

    #[test]
    fn split_path_row_col_invalid_col() {
        assert_eq!(split_path_row_col("file.rs:10:abc"), None);
    }

    #[test]
    fn split_path_row_col_invalid_row() {
        assert_eq!(split_path_row_col("file.rs:abc:5"), None);
    }

    #[test]
    fn split_path_row_col_no_colons() {
        assert_eq!(split_path_row_col("file.rs"), None);
    }

    // --- split_path_row ---

    #[test]
    fn split_path_row_valid() {
        let result = split_path_row("file.rs:42");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(41, 0))));
    }

    #[test]
    fn split_path_row_trailing_colon() {
        let result = split_path_row("file.rs:42:");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(41, 0))));
    }

    #[test]
    fn split_path_row_one() {
        let result = split_path_row("file.rs:1");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(0, 0))));
    }

    #[test]
    fn split_path_row_zero() {
        let result = split_path_row("file.rs:0");
        assert_eq!(result, Some((PathBuf::from("file.rs"), Position::new(0, 0))));
    }

    #[test]
    fn split_path_row_invalid_number() {
        assert_eq!(split_path_row("file.rs:abc"), None);
    }

    #[test]
    fn split_path_row_no_colon() {
        assert_eq!(split_path_row("file.rs"), None);
    }

    // --- parse_file ---

    #[test]
    fn parse_file_plain_nonexistent_path() {
        // Non-existent path with no colon → returned as-is
        let (path, pos) = parse_file("nonexistent_file.rs");
        assert_eq!(path, PathBuf::from("nonexistent_file.rs"));
        assert_eq!(pos, Position::default());
    }

    #[test]
    fn parse_file_with_row() {
        let (path, pos) = parse_file("nonexistent_file.rs:42");
        assert_eq!(path, PathBuf::from("nonexistent_file.rs"));
        assert_eq!(pos, Position::new(41, 0));
    }

    #[test]
    fn parse_file_with_row_and_col() {
        let (path, pos) = parse_file("nonexistent_file.rs:42:5");
        assert_eq!(path, PathBuf::from("nonexistent_file.rs"));
        assert_eq!(pos, Position::new(41, 4));
    }

    #[test]
    fn parse_file_with_trailing_colons() {
        let (path, pos) = parse_file("nonexistent_file.rs:42:5:");
        assert_eq!(path, PathBuf::from("nonexistent_file.rs"));
        assert_eq!(pos, Position::new(41, 4));
    }

    #[test]
    fn parse_file_invalid_row_treated_as_plain() {
        let (path, pos) = parse_file("nonexistent_file.rs:abc");
        assert_eq!(path, PathBuf::from("nonexistent_file.rs:abc"));
        assert_eq!(pos, Position::default());
    }

    // --- parse_multiple_args ---

    #[test]
    fn parse_multiple_args_keeps_nonexistent_files() {
        let dir = tempfile::tempdir().expect("tempdir");
        let existing = dir.path().join("existing.rs");
        std::fs::write(&existing, "").expect("write");
        let missing = dir.path().join("missing.rs");

        let args = vec![
            existing.to_string_lossy().into_owned(),
            missing.to_string_lossy().into_owned(),
        ];

        let StartupAction::OpenFiles(files) = parse_multiple_args(&args) else {
            panic!("expected OpenFiles");
        };

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|(path, _)| path == &existing));
        assert!(files.iter().any(|(path, _)| path == &missing));
    }

    #[test]
    fn parse_multiple_args_still_skips_directories() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("file.rs");
        std::fs::write(&file, "").expect("write");

        let args = vec![
            dir.path().to_string_lossy().into_owned(),
            file.to_string_lossy().into_owned(),
        ];

        let StartupAction::OpenFiles(files) = parse_multiple_args(&args) else {
            panic!("expected OpenFiles");
        };

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, file);
    }

    // --- CliOverrides::apply ---

    #[test]
    fn overrides_apply_log_file() {
        let config = DhxConfig::default();
        let overrides = CliOverrides {
            log: Some(PathBuf::from("/tmp/test.log")),
            log_level: None,
            verbose: false,
            theme: None,
            font_size: None,
        };
        let config = overrides.apply(config);
        assert_eq!(config.logging.log_file, Some(PathBuf::from("/tmp/test.log")));
    }

    #[test]
    fn overrides_apply_verbose_sets_debug() {
        let config = DhxConfig::default();
        let overrides = CliOverrides {
            log: None,
            log_level: None,
            verbose: true,
            theme: None,
            font_size: None,
        };
        let config = overrides.apply(config);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn overrides_log_level_takes_precedence_over_verbose() {
        let config = DhxConfig::default();
        let overrides = CliOverrides {
            log: None,
            log_level: Some("trace".to_string()),
            verbose: true,
            theme: None,
            font_size: None,
        };
        let config = overrides.apply(config);
        assert_eq!(config.logging.level, "trace");
    }

    #[test]
    fn overrides_apply_theme() {
        let config = DhxConfig::default();
        let overrides = CliOverrides {
            log: None,
            log_level: None,
            verbose: false,
            theme: Some("gruvbox".to_string()),
            font_size: None,
        };
        let config = overrides.apply(config);
        assert_eq!(config.initial_theme, Some("gruvbox".to_string()));
    }

    #[test]
    fn overrides_apply_font_size() {
        let config = DhxConfig::default();
        let overrides = CliOverrides {
            log: None,
            log_level: None,
            verbose: false,
            theme: None,
            font_size: Some(18.0),
        };
        let config = overrides.apply(config);
        assert!((config.font.size - 18.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overrides_empty_leaves_config_unchanged() {
        let config = DhxConfig::default();
        let original_level = config.logging.level.clone();
        let original_size = config.font.size;
        let overrides = CliOverrides {
            log: None,
            log_level: None,
            verbose: false,
            theme: None,
            font_size: None,
        };
        let config = overrides.apply(config);
        assert_eq!(config.logging.level, original_level);
        assert!((config.font.size - original_size).abs() < f64::EPSILON);
        assert!(config.initial_theme.is_none());
    }
}
