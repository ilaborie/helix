//! Command-line argument parsing for the dhx binary.

use std::path::{Path, PathBuf};

use helix_core::Position;
use helix_dioxus::StartupAction;

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

    // Single argument — len == 1 after the checks above
    let Some(arg) = args.first() else {
        unreachable!("args is non-empty after is_empty check");
    };
    parse_single_arg(arg)
}

/// Parse multiple command-line arguments.
fn parse_multiple_args(args: &[String]) -> StartupAction {
    let files: Vec<(PathBuf, Position)> = args.iter().map(|a| parse_file(a)).collect();

    // Filter to files that actually exist on disk
    let files: Vec<(PathBuf, Position)> = files.into_iter().filter(|(path, _)| path.is_file()).collect();

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
}
