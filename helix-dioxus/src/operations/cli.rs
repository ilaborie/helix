//! CLI command execution operations.

use std::path::PathBuf;

use crate::operations::{
    BufferOps, EditingOps, JumpOps, PickerOps, ShellOps, TextManipulationOps, ThemeOps,
};
use crate::state::{EditorContext, NotificationSeverity, ShellBehavior};

/// Extension trait for CLI command operations.
pub trait CliOps {
    fn execute_command(&mut self);
}

impl CliOps for EditorContext {
    /// Execute the current command input.
    fn execute_command(&mut self) {
        let input = self.command_input.trim();
        if input.is_empty() {
            self.command_mode = false;
            return;
        }

        // Parse the command
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0];
        let args = parts.get(1).map(|s| s.trim());

        match cmd {
            "o" | "open" => {
                if let Some(filename) = args {
                    // Open specific file
                    let path = PathBuf::from(filename);
                    self.open_file(&path);
                } else {
                    // Show file picker
                    self.show_file_picker();
                }
            }
            "q" | "quit" => {
                self.try_quit(false);
            }
            "q!" | "quit!" => {
                self.try_quit(true);
            }
            "w" | "write" => {
                let path = args.map(PathBuf::from);
                if path.is_none() {
                    // Check if current document is a scratch buffer (no path)
                    let view_id = self.editor.tree.focus;
                    let doc_id = self.editor.tree.get(view_id).doc;
                    if let Some(doc) = self.editor.document(doc_id) {
                        if doc.path().is_none() {
                            // Show Save As dialog instead of erroring
                            self.show_save_as_dialog();
                            self.command_mode = false;
                            self.command_input.clear();
                            return;
                        }
                    }
                }
                self.save_document(path, false);
            }
            "w!" | "write!" => {
                let path = args.map(PathBuf::from);
                self.save_document(path, true);
            }
            "wq" | "x" => {
                self.save_document(None, false);
                self.try_quit(false);
            }
            "wq!" | "x!" => {
                self.save_document(None, true);
                self.try_quit(true);
            }

            // New file command
            "n" | "new" => {
                self.create_new_buffer();
            }

            // Buffer commands
            "b" | "buffer" => {
                self.show_buffer_picker();
            }
            "bn" | "bnext" => {
                self.cycle_buffer(1);
            }
            "bp" | "bprev" | "bprevious" => {
                self.cycle_buffer(-1);
            }
            "bd" | "bdelete" => {
                self.close_current_buffer(false);
            }
            "bd!" | "bdelete!" => {
                self.close_current_buffer(true);
            }

            // Reload
            "reload" | "rl" => {
                self.reload_document();
            }

            // Write all
            "write-all" | "wa" => {
                self.write_all();
            }

            // Quit all
            "quit-all" | "qa" => {
                self.quit_all(false);
            }
            "quit-all!" | "qa!" => {
                self.quit_all(true);
            }

            // Buffer close all
            "buffer-close-all" | "bca" => {
                self.buffer_close_all(false);
            }
            "buffer-close-all!" | "bca!" => {
                self.buffer_close_all(true);
            }

            // Buffer close others
            "buffer-close-others" | "bco" => {
                self.buffer_close_others();
            }

            // Directory commands
            "cd" | "change-current-directory" => {
                if let Some(path_str) = args {
                    self.change_directory(&PathBuf::from(path_str));
                } else {
                    // cd with no args goes to home
                    if let Ok(home) = helix_stdx::path::home_dir() {
                        self.change_directory(&home);
                    }
                }
            }
            "pwd" => {
                self.print_working_directory();
            }

            // Registers
            "reg" | "registers" => {
                self.show_register_picker();
            }

            // Command panel
            "cmd" | "commands" => {
                self.show_command_panel();
            }

            // Theme
            "theme" => match args {
                Some(name) => {
                    let name = name.to_string();
                    if let Err(e) = self.apply_theme(&name) {
                        self.show_notification(
                            format!("Theme error: {e}"),
                            NotificationSeverity::Error,
                        );
                    }
                }
                None => {
                    self.show_theme_picker();
                }
            },

            // History navigation
            "earlier" => {
                let steps = args.and_then(|a| a.parse().ok()).unwrap_or(1);
                self.earlier(steps);
            }
            "later" => {
                let steps = args.and_then(|a| a.parse().ok()).unwrap_or(1);
                self.later(steps);
            }

            // Shell commands
            "pipe" | "sh" => {
                if let Some(shell_cmd) = args {
                    self.shell_input = shell_cmd.to_string();
                    self.shell_behavior = ShellBehavior::Replace;
                    let view_id = self.editor.tree.focus;
                    let doc_id = self.editor.tree.get(view_id).doc;
                    self.execute_shell_command(doc_id, view_id);
                    self.shell_input.clear();
                } else {
                    self.show_notification(
                        "Usage: :pipe <command>".to_string(),
                        NotificationSeverity::Warning,
                    );
                }
            }
            "insert-output" => {
                if let Some(shell_cmd) = args {
                    self.shell_input = shell_cmd.to_string();
                    self.shell_behavior = ShellBehavior::Insert;
                    let view_id = self.editor.tree.focus;
                    let doc_id = self.editor.tree.get(view_id).doc;
                    self.execute_shell_command(doc_id, view_id);
                    self.shell_input.clear();
                } else {
                    self.show_notification(
                        "Usage: :insert-output <command>".to_string(),
                        NotificationSeverity::Warning,
                    );
                }
            }
            "append-output" => {
                if let Some(shell_cmd) = args {
                    self.shell_input = shell_cmd.to_string();
                    self.shell_behavior = ShellBehavior::Append;
                    let view_id = self.editor.tree.focus;
                    let doc_id = self.editor.tree.get(view_id).doc;
                    self.execute_shell_command(doc_id, view_id);
                    self.shell_input.clear();
                } else {
                    self.show_notification(
                        "Usage: :append-output <command>".to_string(),
                        NotificationSeverity::Warning,
                    );
                }
            }
            "pipe-to" => {
                if let Some(shell_cmd) = args {
                    self.shell_input = shell_cmd.to_string();
                    self.shell_behavior = ShellBehavior::Ignore;
                    let view_id = self.editor.tree.focus;
                    let doc_id = self.editor.tree.get(view_id).doc;
                    self.execute_shell_command(doc_id, view_id);
                    self.shell_input.clear();
                } else {
                    self.show_notification(
                        "Usage: :pipe-to <command>".to_string(),
                        NotificationSeverity::Warning,
                    );
                }
            }
            "run-shell-command" | "run" => {
                if let Some(shell_cmd) = args {
                    self.shell_input = shell_cmd.to_string();
                    self.shell_behavior = ShellBehavior::Insert;
                    let view_id = self.editor.tree.focus;
                    let doc_id = self.editor.tree.get(view_id).doc;
                    self.execute_shell_command(doc_id, view_id);
                    self.shell_input.clear();
                } else {
                    self.show_notification(
                        "Usage: :run-shell-command <command>".to_string(),
                        NotificationSeverity::Warning,
                    );
                }
            }

            // Sort selections
            "sort" => {
                self.sort_selections();
            }

            // Reflow text
            "reflow" => {
                let width = args.and_then(|arg| arg.parse::<usize>().ok());
                self.reflow_selections(width);
            }

            // Open config/log files
            "config-open" => {
                self.open_file(&helix_loader::config_file());
            }
            "log-open" => {
                self.open_file(&helix_loader::log_file());
            }

            // Encoding
            "encoding" => {
                let (msg, severity) = {
                    let (_view, doc) = helix_view::current!(self.editor);
                    match args {
                        Some(label) => {
                            let result = doc.set_encoding(label);
                            match result {
                                Ok(()) => (
                                    format!("Encoding set to {}", doc.encoding().name()),
                                    NotificationSeverity::Info,
                                ),
                                Err(err) => (
                                    format!("Invalid encoding: {err}"),
                                    NotificationSeverity::Error,
                                ),
                            }
                        }
                        None => (
                            doc.encoding().name().to_string(),
                            NotificationSeverity::Info,
                        ),
                    }
                };
                self.show_notification(msg, severity);
            }

            // Line ending
            "set-line-ending" | "line-ending" => {
                use helix_core::LineEnding;

                if let Some(arg) = args {
                    let arg = arg.to_ascii_lowercase();
                    let line_ending = match arg.as_str() {
                        "crlf" => Some(LineEnding::Crlf),
                        "lf" => Some(LineEnding::LF),
                        _ => None,
                    };
                    if let Some(le) = line_ending {
                        let (view, doc) = helix_view::current!(self.editor);
                        doc.line_ending = le;

                        let mut pos = 0;
                        let transaction = helix_core::Transaction::change(
                            doc.text(),
                            doc.text().lines().filter_map(|line| {
                                pos += line.len_chars();
                                match helix_core::line_ending::get_line_ending(&line) {
                                    Some(ending) if ending != le => {
                                        let start = pos - ending.len_chars();
                                        let end = pos;
                                        Some((start, end, Some(le.as_str().into())))
                                    }
                                    _ => None,
                                }
                            }),
                        );
                        doc.apply(&transaction, view.id);
                        doc.append_changes_to_history(view);
                    } else {
                        self.show_notification(
                            "Invalid line ending. Use 'lf' or 'crlf'".to_string(),
                            NotificationSeverity::Error,
                        );
                    }
                } else {
                    let msg = {
                        let (_view, doc) = helix_view::current!(self.editor);
                        match doc.line_ending {
                            LineEnding::Crlf => "crlf",
                            LineEnding::LF => "lf",
                            #[allow(unreachable_patterns)]
                            _ => "unknown",
                        }
                        .to_string()
                    };
                    self.show_notification(msg, NotificationSeverity::Info);
                }
            }

            // Jump list
            "jumplist-clear" => {
                self.clear_jumplist();
            }

            // Tree-sitter scopes
            "tree-sitter-scopes" => {
                let msg = {
                    let (view, doc) = helix_view::current!(self.editor);
                    let text = doc.text().slice(..);
                    let pos = doc.selection(view.id).primary().cursor(text);
                    let scopes = helix_core::indent::get_scopes(doc.syntax(), text, pos);
                    format!("{scopes:?}")
                };
                self.show_notification(msg, NotificationSeverity::Info);
            }

            _ => {
                log::warn!("Unknown command: {}", cmd);
                self.show_notification(
                    format!("Unknown command: {}", cmd),
                    NotificationSeverity::Error,
                );
            }
        }

        self.command_mode = false;
        self.command_input.clear();
    }
}
