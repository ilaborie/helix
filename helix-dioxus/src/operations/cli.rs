//! CLI command execution operations.

use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use crate::operations::{
    BufferOps, ClipboardOps, EditingOps, JumpOps, PickerOps, ShellOps, TextManipulationOps, ThemeOps, VcsOps,
};
use crate::state::{EditorContext, NotificationSeverity, ShellBehavior};

/// A command completion entry: name, aliases, and human-readable description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandCompletion {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}

/// Return the static list of all known commands (single source of truth).
#[must_use]
pub fn command_completions() -> &'static [CommandCompletion] {
    static COMMANDS: &[CommandCompletion] = &[
        CommandCompletion {
            name: "open",
            aliases: &["o"],
            description: "Open a file",
        },
        CommandCompletion {
            name: "quit",
            aliases: &["q"],
            description: "Quit the editor",
        },
        CommandCompletion {
            name: "quit!",
            aliases: &["q!"],
            description: "Force quit",
        },
        CommandCompletion {
            name: "write",
            aliases: &["w"],
            description: "Save the current file",
        },
        CommandCompletion {
            name: "write!",
            aliases: &["w!"],
            description: "Force save",
        },
        CommandCompletion {
            name: "wq",
            aliases: &["x"],
            description: "Save and quit",
        },
        CommandCompletion {
            name: "wq!",
            aliases: &["x!"],
            description: "Force save and quit",
        },
        CommandCompletion {
            name: "new",
            aliases: &["n"],
            description: "Create a new buffer",
        },
        CommandCompletion {
            name: "buffer",
            aliases: &["b"],
            description: "Switch buffer",
        },
        CommandCompletion {
            name: "bnext",
            aliases: &["bn"],
            description: "Next buffer",
        },
        CommandCompletion {
            name: "bprev",
            aliases: &["bp", "bprevious"],
            description: "Previous buffer",
        },
        CommandCompletion {
            name: "bdelete",
            aliases: &["bd"],
            description: "Close current buffer",
        },
        CommandCompletion {
            name: "bdelete!",
            aliases: &["bd!"],
            description: "Force close buffer",
        },
        CommandCompletion {
            name: "reload",
            aliases: &["rl"],
            description: "Reload document from disk",
        },
        CommandCompletion {
            name: "write-all",
            aliases: &["wa"],
            description: "Save all buffers",
        },
        CommandCompletion {
            name: "quit-all",
            aliases: &["qa"],
            description: "Quit all buffers",
        },
        CommandCompletion {
            name: "quit-all!",
            aliases: &["qa!"],
            description: "Force quit all",
        },
        CommandCompletion {
            name: "buffer-close-all",
            aliases: &["bca"],
            description: "Close all buffers",
        },
        CommandCompletion {
            name: "buffer-close-all!",
            aliases: &["bca!"],
            description: "Force close all",
        },
        CommandCompletion {
            name: "buffer-close-others",
            aliases: &["bco"],
            description: "Close other buffers",
        },
        CommandCompletion {
            name: "cd",
            aliases: &["change-current-directory"],
            description: "Change directory",
        },
        CommandCompletion {
            name: "pwd",
            aliases: &[],
            description: "Print working directory",
        },
        CommandCompletion {
            name: "registers",
            aliases: &["reg"],
            description: "Show registers",
        },
        CommandCompletion {
            name: "commands",
            aliases: &["cmd"],
            description: "Open command panel",
        },
        CommandCompletion {
            name: "theme",
            aliases: &[],
            description: "Set or pick theme",
        },
        CommandCompletion {
            name: "earlier",
            aliases: &[],
            description: "Undo N steps",
        },
        CommandCompletion {
            name: "later",
            aliases: &[],
            description: "Redo N steps",
        },
        CommandCompletion {
            name: "pipe",
            aliases: &["sh"],
            description: "Pipe selection through shell",
        },
        CommandCompletion {
            name: "insert-output",
            aliases: &[],
            description: "Insert shell output",
        },
        CommandCompletion {
            name: "append-output",
            aliases: &[],
            description: "Append shell output",
        },
        CommandCompletion {
            name: "pipe-to",
            aliases: &[],
            description: "Pipe selection, discard output",
        },
        CommandCompletion {
            name: "run-shell-command",
            aliases: &["run"],
            description: "Run shell command",
        },
        CommandCompletion {
            name: "sort",
            aliases: &[],
            description: "Sort selections",
        },
        CommandCompletion {
            name: "reflow",
            aliases: &[],
            description: "Reflow text to width",
        },
        CommandCompletion {
            name: "config-open",
            aliases: &[],
            description: "Open config file",
        },
        CommandCompletion {
            name: "log-open",
            aliases: &[],
            description: "Open log file",
        },
        CommandCompletion {
            name: "encoding",
            aliases: &[],
            description: "Show/set encoding",
        },
        CommandCompletion {
            name: "set-line-ending",
            aliases: &["line-ending"],
            description: "Show/set line ending",
        },
        CommandCompletion {
            name: "jumplist-clear",
            aliases: &[],
            description: "Clear jump list",
        },
        CommandCompletion {
            name: "config-reload",
            aliases: &[],
            description: "Reload config",
        },
        CommandCompletion {
            name: "set",
            aliases: &[],
            description: "Set config option",
        },
        CommandCompletion {
            name: "toggle",
            aliases: &[],
            description: "Toggle config option",
        },
        CommandCompletion {
            name: "format",
            aliases: &["fmt"],
            description: "Format document",
        },
        CommandCompletion {
            name: "lsp-restart",
            aliases: &[],
            description: "Restart LSP server",
        },
        CommandCompletion {
            name: "tree-sitter-scopes",
            aliases: &[],
            description: "Show TS scopes at cursor",
        },
        CommandCompletion {
            name: "emoji",
            aliases: &[],
            description: "Open emoji picker",
        },
        // --- Batch 1: Buffer/File Aliases & Force Variants ---
        CommandCompletion {
            name: "buffer-close",
            aliases: &["bc", "bclose"],
            description: "Close current buffer",
        },
        CommandCompletion {
            name: "buffer-close!",
            aliases: &["bc!", "bclose!"],
            description: "Force close current buffer",
        },
        CommandCompletion {
            name: "buffer-close-others!",
            aliases: &["bco!"],
            description: "Force close other buffers",
        },
        CommandCompletion {
            name: "exit",
            aliases: &["xit"],
            description: "Save and quit",
        },
        CommandCompletion {
            name: "exit!",
            aliases: &["xit!"],
            description: "Force save and quit",
        },
        CommandCompletion {
            name: "write-all!",
            aliases: &["wa!"],
            description: "Force save all buffers",
        },
        CommandCompletion {
            name: "write-buffer-close",
            aliases: &["wbc"],
            description: "Save and close current buffer",
        },
        CommandCompletion {
            name: "write-buffer-close!",
            aliases: &["wbc!"],
            description: "Force save and close buffer",
        },
        CommandCompletion {
            name: "write-quit-all",
            aliases: &["wqa", "xa"],
            description: "Save all and quit",
        },
        CommandCompletion {
            name: "write-quit-all!",
            aliases: &["wqa!", "xa!"],
            description: "Force save all and quit",
        },
        CommandCompletion {
            name: "update",
            aliases: &["u"],
            description: "Save if modified",
        },
        CommandCompletion {
            name: "reload-all",
            aliases: &["rla"],
            description: "Reload all files from disk",
        },
        CommandCompletion {
            name: "read",
            aliases: &["r"],
            description: "Read file into buffer",
        },
        CommandCompletion {
            name: "move",
            aliases: &["mv"],
            description: "Move/rename current file",
        },
        CommandCompletion {
            name: "move!",
            aliases: &["mv!"],
            description: "Force move/rename file",
        },
        // --- Batch 2: Config & Language Commands ---
        CommandCompletion {
            name: "get-option",
            aliases: &["get"],
            description: "Get config option value",
        },
        CommandCompletion {
            name: "set-language",
            aliases: &["lang"],
            description: "Set document language",
        },
        CommandCompletion {
            name: "indent-style",
            aliases: &[],
            description: "Show/set indent style",
        },
        CommandCompletion {
            name: "config-open-workspace",
            aliases: &[],
            description: "Open workspace config",
        },
        CommandCompletion {
            name: "tutor",
            aliases: &[],
            description: "Open tutorial",
        },
        // --- Batch 3: Register & Clipboard Commands ---
        CommandCompletion {
            name: "clipboard-yank",
            aliases: &[],
            description: "Yank to system clipboard",
        },
        CommandCompletion {
            name: "clipboard-paste-after",
            aliases: &[],
            description: "Paste from clipboard after",
        },
        CommandCompletion {
            name: "clipboard-paste-before",
            aliases: &[],
            description: "Paste from clipboard before",
        },
        CommandCompletion {
            name: "clipboard-paste-replace",
            aliases: &[],
            description: "Replace with clipboard",
        },
        CommandCompletion {
            name: "primary-clipboard-yank",
            aliases: &[],
            description: "Yank to primary clipboard",
        },
        CommandCompletion {
            name: "primary-clipboard-paste-after",
            aliases: &[],
            description: "Paste from primary after",
        },
        CommandCompletion {
            name: "primary-clipboard-paste-before",
            aliases: &[],
            description: "Paste from primary before",
        },
        CommandCompletion {
            name: "primary-clipboard-paste-replace",
            aliases: &[],
            description: "Replace with primary clipboard",
        },
        CommandCompletion {
            name: "show-clipboard-provider",
            aliases: &[],
            description: "Show clipboard provider",
        },
        CommandCompletion {
            name: "yank-join",
            aliases: &[],
            description: "Yank joined selections",
        },
        CommandCompletion {
            name: "yank-diagnostic",
            aliases: &[],
            description: "Yank diagnostic at cursor",
        },
        CommandCompletion {
            name: "clear-register",
            aliases: &[],
            description: "Clear a register",
        },
        CommandCompletion {
            name: "set-register",
            aliases: &[],
            description: "Set register content",
        },
        CommandCompletion {
            name: "character-info",
            aliases: &[],
            description: "Show character info at cursor",
        },
        CommandCompletion {
            name: "echo",
            aliases: &[],
            description: "Echo message",
        },
        CommandCompletion {
            name: "goto",
            aliases: &[],
            description: "Go to line number",
        },
        // --- Batch 4: LSP & Misc ---
        CommandCompletion {
            name: "lsp-stop",
            aliases: &[],
            description: "Stop LSP servers",
        },
        CommandCompletion {
            name: "reset-diff-change",
            aliases: &["diffget", "diffg"],
            description: "Revert diff hunk at cursor",
        },
        CommandCompletion {
            name: "tree-sitter-highlight-name",
            aliases: &[],
            description: "Show TS highlight at cursor",
        },
        CommandCompletion {
            name: "tree-sitter-subtree",
            aliases: &["ts-subtree"],
            description: "Show TS subtree at cursor",
        },
        CommandCompletion {
            name: "cquit",
            aliases: &["cq"],
            description: "Quit with exit code",
        },
        CommandCompletion {
            name: "cquit!",
            aliases: &["cq!"],
            description: "Force quit with exit code",
        },
    ];
    COMMANDS
}

/// Extension trait for CLI command operations.
pub trait CliOps {
    fn execute_command(&mut self);
    fn reload_config(&mut self);
    fn set_option(&mut self, key: &str, value: &str);
    fn toggle_option(&mut self, args: &str);
}

impl CliOps for EditorContext {
    /// Execute the current command input.
    #[allow(clippy::indexing_slicing)] // parts[] access is guarded by splitn(2, ..) guarantees
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
            "wq" | "x" | "exit" | "xit" => {
                self.save_document(None, false);
                self.try_quit(false);
            }
            "wq!" | "x!" | "exit!" | "xit!" => {
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
            "bd" | "bdelete" | "bc" | "bclose" | "buffer-close" => {
                self.close_current_buffer(false);
            }
            "bd!" | "bdelete!" | "bc!" | "bclose!" | "buffer-close!" => {
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
                self.buffer_close_others_impl(false);
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
                        self.show_notification(format!("Theme error: {e}"), NotificationSeverity::Error);
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
                    self.show_notification("Usage: :pipe <command>".to_string(), NotificationSeverity::Warning);
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
                    self.show_notification("Usage: :pipe-to <command>".to_string(), NotificationSeverity::Warning);
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
                                Err(err) => (format!("Invalid encoding: {err}"), NotificationSeverity::Error),
                            }
                        }
                        None => (doc.encoding().name().to_string(), NotificationSeverity::Info),
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

            // Config reload
            "config-reload" => {
                self.reload_config();
            }

            // Set/toggle config option at runtime
            "set" => match args {
                Some(set_args) => {
                    let set_args = set_args.to_string();
                    let parts: Vec<&str> = set_args.splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        let key = parts[0].to_string();
                        let val = parts[1].trim().to_string();
                        self.set_option(&key, &val);
                    } else {
                        self.show_notification("Usage: :set <key> <value>".to_string(), NotificationSeverity::Warning);
                    }
                }
                None => {
                    self.show_notification("Usage: :set <key> <value>".to_string(), NotificationSeverity::Warning);
                }
            },
            "toggle" => {
                let toggle_args = args.map(ToString::to_string);
                match toggle_args {
                    Some(toggle_args) => {
                        self.toggle_option(&toggle_args);
                    }
                    None => {
                        self.show_notification(
                            "Usage: :toggle <key> [val1 val2 ...]".to_string(),
                            NotificationSeverity::Warning,
                        );
                    }
                }
            }

            // Format document
            "format" | "fmt" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.format_document(doc_id, view_id);
            }

            // LSP restart
            "lsp-restart" => {
                if let Some(server_name) = args {
                    let name = server_name.to_string();
                    self.restart_lsp_server(&name);
                } else {
                    // Restart all servers for the current document's language
                    let server_names: Vec<String> = {
                        let (_view, doc) = helix_view::current_ref!(self.editor);
                        doc.language_config()
                            .map(|config| config.language_servers.iter().map(|ls| ls.name.clone()).collect())
                            .unwrap_or_default()
                    };
                    for name in &server_names {
                        self.restart_lsp_server(name);
                    }
                }
            }

            // Emoji picker
            "emoji" => {
                self.show_emoji_picker();
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

            // --- Batch 1: Buffer/File Aliases & Force Variants ---
            "bco!" | "buffer-close-others!" => {
                self.buffer_close_others_impl(true);
            }
            "wa!" | "write-all!" => {
                self.write_all_impl(true);
            }
            "wbc" | "write-buffer-close" => {
                self.write_buffer_close(false);
            }
            "wbc!" | "write-buffer-close!" => {
                self.write_buffer_close(true);
            }
            "wqa" | "xa" | "write-quit-all" => {
                self.write_all();
                self.quit_all(false);
            }
            "wqa!" | "xa!" | "write-quit-all!" => {
                self.write_all_impl(true);
                self.quit_all(true);
            }
            "u" | "update" => {
                self.update_document();
            }
            "rla" | "reload-all" => {
                self.reload_all();
            }
            "r" | "read" => {
                if let Some(filename) = args {
                    let path = PathBuf::from(filename);
                    self.read_file(&path);
                } else {
                    self.show_notification("Usage: :read <file>".to_string(), NotificationSeverity::Warning);
                }
            }
            "mv" | "move" => {
                if let Some(filename) = args {
                    self.move_file(PathBuf::from(filename), false);
                } else {
                    self.show_notification("Usage: :move <path>".to_string(), NotificationSeverity::Warning);
                }
            }
            "mv!" | "move!" => {
                if let Some(filename) = args {
                    self.move_file(PathBuf::from(filename), true);
                } else {
                    self.show_notification("Usage: :move! <path>".to_string(), NotificationSeverity::Warning);
                }
            }

            // --- Batch 2: Config & Language Commands ---
            "get" | "get-option" => {
                if let Some(key) = args {
                    let key = key.to_string();
                    self.get_option(&key);
                } else {
                    self.show_notification("Usage: :get-option <key>".to_string(), NotificationSeverity::Warning);
                }
            }
            "lang" | "set-language" => {
                if let Some(lang_id) = args {
                    let lang_id = lang_id.to_string();
                    self.set_language(&lang_id);
                } else {
                    // Show current language
                    let lang = {
                        let (_view, doc) = helix_view::current_ref!(self.editor);
                        doc.language_name().unwrap_or("plaintext").to_string()
                    };
                    self.show_notification(lang, NotificationSeverity::Info);
                }
            }
            "indent-style" => {
                let args_owned = args.map(str::to_string);
                self.indent_style_command(args_owned.as_deref());
            }
            "config-open-workspace" => {
                self.open_file(&helix_loader::workspace_config_file());
            }
            "tutor" => {
                self.open_file(&helix_loader::runtime_file("tutor"));
            }

            // --- Batch 3: Register & Clipboard Commands ---
            "clipboard-yank" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('+');
                self.yank(doc_id, view_id);
            }
            "clipboard-paste-after" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('+');
                self.paste(doc_id, view_id, false);
            }
            "clipboard-paste-before" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('+');
                self.paste(doc_id, view_id, true);
            }
            "clipboard-paste-replace" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('+');
                self.replace_with_yanked(doc_id, view_id);
            }
            "primary-clipboard-yank" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('*');
                self.yank(doc_id, view_id);
            }
            "primary-clipboard-paste-after" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('*');
                self.paste(doc_id, view_id, false);
            }
            "primary-clipboard-paste-before" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('*');
                self.paste(doc_id, view_id, true);
            }
            "primary-clipboard-paste-replace" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                self.editor.selected_register = Some('*');
                self.replace_with_yanked(doc_id, view_id);
            }
            "show-clipboard-provider" => {
                let name = self.editor.registers.clipboard_provider_name();
                self.show_notification(format!("Clipboard provider: {name}"), NotificationSeverity::Info);
            }
            "yank-join" => {
                let separator = args.unwrap_or(" ").to_string();
                self.yank_join(&separator);
            }
            "yank-diagnostic" => {
                // Capture diagnostic messages before mutable borrow
                let diag_messages: Vec<String> = {
                    let (view, doc) = helix_view::current_ref!(self.editor);
                    let primary = doc.selection(view.id).primary();
                    doc.diagnostics()
                        .iter()
                        .filter(|d| primary.overlaps(&helix_core::Range::new(d.range.start, d.range.end)))
                        .map(|d| d.message.clone())
                        .collect()
                };
                self.yank_diagnostic_impl(diag_messages);
            }
            "clear-register" => {
                if let Some(reg) = args.and_then(|a| a.chars().next()) {
                    self.editor.registers.remove(reg);
                    self.show_notification(format!("Cleared register '{reg}'"), NotificationSeverity::Info);
                } else {
                    self.show_notification(
                        "Usage: :clear-register <char>".to_string(),
                        NotificationSeverity::Warning,
                    );
                }
            }
            "set-register" => {
                if let Some(set_args) = args {
                    let set_args = set_args.to_string();
                    let mut chars = set_args.chars();
                    if let Some(reg) = chars.next() {
                        let content: String = chars.skip_while(|c| c.is_whitespace()).collect();
                        if let Err(e) = self.editor.registers.write(reg, vec![content]) {
                            self.show_notification(format!("Failed to set register: {e}"), NotificationSeverity::Error);
                        }
                    } else {
                        self.show_notification(
                            "Usage: :set-register <char> <content>".to_string(),
                            NotificationSeverity::Warning,
                        );
                    }
                } else {
                    self.show_notification(
                        "Usage: :set-register <char> <content>".to_string(),
                        NotificationSeverity::Warning,
                    );
                }
            }
            "character-info" => {
                self.show_character_info();
            }
            "echo" => {
                let message = args.unwrap_or("").to_string();
                self.show_notification(message, NotificationSeverity::Info);
            }
            "goto" => {
                if let Some(line_str) = args {
                    if let Ok(line) = line_str.parse::<usize>() {
                        let target = line.saturating_sub(1); // 1-indexed to 0-indexed
                        self.goto_line_column(target, 0);
                    } else {
                        self.show_notification(
                            format!("Invalid line number: {line_str}"),
                            NotificationSeverity::Error,
                        );
                    }
                } else {
                    self.show_notification("Usage: :goto <line>".to_string(), NotificationSeverity::Warning);
                }
            }

            // --- Batch 4: LSP & Misc ---
            "lsp-stop" => {
                self.lsp_stop();
            }
            "reset-diff-change" | "diffget" | "diffg" => {
                let view_id = self.editor.tree.focus;
                let doc_id = self.editor.tree.get(view_id).doc;
                let cursor_line = {
                    let doc = self.editor.document(doc_id).expect("doc exists");
                    let text = doc.text().slice(..);
                    let pos = doc.selection(view_id).primary().cursor(text);
                    text.char_to_line(pos) + 1 // 1-indexed
                };
                self.revert_hunk_at_line(doc_id, view_id, cursor_line);
            }
            "tree-sitter-highlight-name" => {
                let msg = {
                    let (view, doc) = helix_view::current!(self.editor);
                    let text = doc.text().slice(..);
                    let pos = doc.selection(view.id).primary().cursor(text);
                    #[allow(clippy::cast_possible_truncation)]
                    let byte_pos = text.char_to_byte(pos) as u32;
                    doc.syntax()
                        .and_then(|syntax| {
                            syntax
                                .descendant_for_byte_range(byte_pos, byte_pos + 1)
                                .map(|node| node.kind().to_string())
                        })
                        .unwrap_or_else(|| "No syntax tree available".to_string())
                };
                self.show_notification(msg, NotificationSeverity::Info);
            }
            "tree-sitter-subtree" | "ts-subtree" => {
                let msg = {
                    let (view, doc) = helix_view::current!(self.editor);
                    let text = doc.text().slice(..);
                    let primary = doc.selection(view.id).primary();
                    #[allow(clippy::cast_possible_truncation)]
                    let from = text.char_to_byte(primary.from()) as u32;
                    #[allow(clippy::cast_possible_truncation)]
                    let to = text.char_to_byte(primary.to()) as u32;
                    doc.syntax()
                        .and_then(|syntax| {
                            syntax.descendant_for_byte_range(from, to).map(|node| {
                                let mut contents = String::new();
                                match helix_core::syntax::pretty_print_tree(&mut contents, node) {
                                    Ok(()) => contents,
                                    Err(_) => "Failed to print subtree".to_string(),
                                }
                            })
                        })
                        .unwrap_or_else(|| "No syntax tree available".to_string())
                };
                self.show_notification(msg, NotificationSeverity::Info);
            }
            "cq" | "cquit" => {
                let code = args.and_then(|a| a.parse::<i32>().ok()).unwrap_or(1);
                self.should_quit = true;
                std::process::exit(code);
            }
            "cq!" | "cquit!" => {
                let code = args.and_then(|a| a.parse::<i32>().ok()).unwrap_or(1);
                std::process::exit(code);
            }

            _ => {
                log::warn!("Unknown command: {cmd}");
                self.show_notification(format!("Unknown command: {cmd}"), NotificationSeverity::Error);
            }
        }

        self.command_mode = false;
        self.command_input.clear();
    }

    fn reload_config(&mut self) {
        use crate::state::load_editor_config;

        // Save old config for refresh_config comparison
        let old_config = self.editor.config().clone();

        // 1. Reload editor config from config.toml [editor] section
        let new_config = load_editor_config();
        self.editor.config = Arc::new(arc_swap::ArcSwap::from_pointee(new_config));

        // 2. Reload syntax language loader (languages.toml)
        match helix_core::config::user_lang_loader() {
            Ok(lang_loader) => {
                self.editor.syn_loader.store(Arc::new(lang_loader));
            }
            Err(err) => {
                log::warn!("Failed to reload language config: {err}");
                self.show_notification(format!("Language config error: {err}"), NotificationSeverity::Warning);
            }
        }

        // 3. Reload theme if changed in config.toml
        if let Some(theme_name) = crate::state::load_theme_name() {
            if theme_name != self.editor.theme.name() {
                if let Err(err) = self.apply_theme(&theme_name) {
                    log::warn!("Failed to apply theme '{theme_name}': {err}");
                    self.show_notification(format!("Theme error: {err}"), NotificationSeverity::Warning);
                }
            }
        }

        // 4. Update syntax highlighting scopes from theme
        let scopes = self.editor.theme.scopes();
        self.editor.syn_loader.load().set_scopes(scopes.to_vec());

        // 5. Re-detect language config for all open documents
        let lang_loader = self.editor.syn_loader.load();
        for document in self.editor.documents.values_mut() {
            document.detect_language(&lang_loader);
        }

        // 6. Notify the editor about config changes
        self.editor.refresh_config(&old_config);

        self.show_notification("Config reloaded".to_string(), NotificationSeverity::Info);
    }

    fn set_option(&mut self, key: &str, value: &str) {
        let key = key.to_lowercase();
        let old_config = self.editor.config().clone();

        // Serialize current config to JSON, update the field, deserialize back
        let mut config = match serde_json::to_value(&old_config) {
            Ok(v) => v,
            Err(err) => {
                self.show_notification(
                    format!("Config serialization error: {err}"),
                    NotificationSeverity::Error,
                );
                return;
            }
        };

        let pointer = format!("/{}", key.replace('.', "/"));
        let Some(field) = config.pointer_mut(&pointer) else {
            self.show_notification(format!("Unknown config key: {key}"), NotificationSeverity::Error);
            return;
        };

        // Parse the value based on the existing field type
        let new_value = if field.is_string() {
            serde_json::Value::String(value.to_string())
        } else {
            match value.parse::<serde_json::Value>() {
                Ok(v) => v,
                Err(err) => {
                    self.show_notification(
                        format!("Could not parse value '{value}': {err}"),
                        NotificationSeverity::Error,
                    );
                    return;
                }
            }
        };
        *field = new_value;

        // Deserialize back to Config
        let new_config: helix_view::editor::Config = match serde_json::from_value(config) {
            Ok(c) => c,
            Err(err) => {
                self.show_notification(format!("Invalid config value: {err}"), NotificationSeverity::Error);
                return;
            }
        };

        // Apply the new config
        self.editor.config = Arc::new(arc_swap::ArcSwap::from_pointee(new_config));
        self.editor.refresh_config(&old_config);
        self.show_notification(format!("Set {key} = {value}"), NotificationSeverity::Info);
    }

    #[allow(clippy::indexing_slicing)] // parts[] access is guarded by splitn guarantees and len checks
    fn toggle_option(&mut self, args: &str) {
        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        let key = parts[0].to_lowercase();
        let old_config = self.editor.config().clone();

        let mut config = match serde_json::to_value(&old_config) {
            Ok(v) => v,
            Err(err) => {
                self.show_notification(
                    format!("Config serialization error: {err}"),
                    NotificationSeverity::Error,
                );
                return;
            }
        };

        let pointer = format!("/{}", key.replace('.', "/"));
        let Some(field) = config.pointer_mut(&pointer) else {
            self.show_notification(format!("Unknown config key: {key}"), NotificationSeverity::Error);
            return;
        };

        // Toggle based on current field type
        match field {
            serde_json::Value::Bool(b) => {
                *field = serde_json::Value::Bool(!*b);
            }
            serde_json::Value::String(ref current) => {
                // Cycle through provided values: `:toggle key val1 val2 ...`
                if parts.len() < 2 {
                    self.show_notification(
                        format!("Usage: :toggle {key} val1 val2 ..."),
                        NotificationSeverity::Warning,
                    );
                    return;
                }
                let values: Vec<&str> = parts[1].split_whitespace().collect();
                let next = values
                    .iter()
                    .skip_while(|v| **v != current.as_str())
                    .nth(1)
                    .unwrap_or(&values[0]);
                *field = serde_json::Value::String((*next).to_string());
            }
            _ => {
                self.show_notification(
                    format!("Cannot toggle {key} (not a boolean or string)"),
                    NotificationSeverity::Warning,
                );
                return;
            }
        }

        // Deserialize back and apply
        let new_config: helix_view::editor::Config = match serde_json::from_value(config) {
            Ok(c) => c,
            Err(err) => {
                self.show_notification(format!("Invalid config value: {err}"), NotificationSeverity::Error);
                return;
            }
        };

        let display_val = {
            let json = serde_json::to_value(&new_config).ok();
            json.and_then(|v| v.pointer(&pointer).cloned())
                .map(|v| v.to_string())
                .unwrap_or_default()
        };

        self.editor.config = Arc::new(arc_swap::ArcSwap::from_pointee(new_config));
        self.editor.refresh_config(&old_config);
        self.show_notification(format!("Toggled {key} = {display_val}"), NotificationSeverity::Info);
    }
}

/// Additional helper methods for CLI commands.
impl EditorContext {
    fn get_option(&mut self, key: &str) {
        let key = key.to_lowercase();
        let config = self.editor.config();

        let json = match serde_json::to_value(&*config) {
            Ok(v) => v,
            Err(err) => {
                self.show_notification(
                    format!("Config serialization error: {err}"),
                    NotificationSeverity::Error,
                );
                return;
            }
        };

        let pointer = format!("/{}", key.replace('.', "/"));
        match json.pointer(&pointer) {
            Some(value) => {
                self.show_notification(format!("{key} = {value}"), NotificationSeverity::Info);
            }
            None => {
                self.show_notification(format!("Unknown config key: {key}"), NotificationSeverity::Error);
            }
        }
    }

    fn set_language(&mut self, lang_id: &str) {
        let lang_id = lang_id.to_string();
        let loader = self.editor.syn_loader.load();

        let doc_id = {
            let (view, _doc) = helix_view::current_ref!(self.editor);
            self.editor.tree.get(view.id).doc
        };

        let doc = self.editor.document_mut(doc_id).expect("doc exists");

        if lang_id == "text" || lang_id == "plaintext" {
            doc.set_language(None, &loader);
        } else if let Err(e) = doc.set_language_by_language_id(&lang_id, &loader) {
            self.show_notification(format!("Unknown language: {e}"), NotificationSeverity::Error);
            return;
        }

        let doc = self.editor.document_mut(doc_id).expect("doc exists");
        doc.detect_indent_and_line_ending();
        self.editor.refresh_language_servers(doc_id);

        self.show_notification(format!("Language set to {lang_id}"), NotificationSeverity::Info);
    }

    fn indent_style_command(&mut self, args: Option<&str>) {
        use helix_core::indent::IndentStyle;

        let (_view, doc) = helix_view::current!(self.editor);
        if let Some(arg) = args {
            let style = match arg.to_ascii_lowercase().as_str() {
                "tabs" | "tab" => Some(IndentStyle::Tabs),
                _ => arg.parse::<u8>().ok().filter(|&n| n > 0 && n <= 16).map(IndentStyle::Spaces),
            };
            if let Some(s) = style {
                doc.indent_style = s;
                self.show_notification(format!("Indent style: {s:?}"), NotificationSeverity::Info);
            } else {
                self.show_notification(
                    "Invalid indent style. Use 'tabs' or a number 1-16".to_string(),
                    NotificationSeverity::Error,
                );
            }
        } else {
            let style = doc.indent_style;
            self.show_notification(format!("{style:?}"), NotificationSeverity::Info);
        }
    }

    fn yank_join(&mut self, separator: &str) {
        let register = self.take_register();
        let (view_id, text, selections_text) = {
            let (view, doc) = helix_view::current_ref!(self.editor);
            let text = doc.text().slice(..);
            let selection = doc.selection(view.id);
            let count = selection.len();
            let joined = selection
                .fragments(text)
                .fold(String::new(), |mut acc, fragment| {
                    if !acc.is_empty() {
                        acc.push_str(separator);
                    }
                    acc.push_str(&fragment);
                    acc
                });
            (view.id, joined, count)
        };
        let _ = view_id; // used for context

        match self.editor.registers.write(register, vec![text.clone()]) {
            Ok(()) => {
                let s = if selections_text == 1 { "" } else { "s" };
                self.show_notification(
                    format!("Joined and yanked {selections_text} selection{s} to register '{register}'"),
                    NotificationSeverity::Info,
                );
            }
            Err(e) => {
                self.show_notification(format!("Failed to yank: {e}"), NotificationSeverity::Error);
            }
        }
    }

    fn yank_diagnostic_impl(&mut self, diag_messages: Vec<String>) {
        if diag_messages.is_empty() {
            self.show_notification(
                "No diagnostics under primary selection".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }

        let n = diag_messages.len();
        match self.editor.registers.write('+', diag_messages) {
            Ok(()) => {
                let s = if n == 1 { "" } else { "s" };
                self.show_notification(
                    format!("Yanked {n} diagnostic{s} to clipboard"),
                    NotificationSeverity::Info,
                );
            }
            Err(e) => {
                self.show_notification(format!("Failed to yank: {e}"), NotificationSeverity::Error);
            }
        }
    }

    fn show_character_info(&mut self) {
        use helix_core::graphemes;

        let (view, doc) = helix_view::current_ref!(self.editor);
        let text = doc.text().slice(..);
        let pos = doc.selection(view.id).primary().cursor(text);
        let end = graphemes::next_grapheme_boundary(text, pos);

        if pos == end {
            return;
        }

        let grapheme: String = text.slice(pos..end).into();
        let mut info = String::new();

        for (i, ch) in grapheme.chars().enumerate() {
            if i > 0 {
                info.push_str(", ");
            }
            let printable = match ch {
                '\0' => "\\0".to_string(),
                '\t' => "\\t".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                _ => ch.to_string(),
            };
            let codepoint = ch as u32;
            let _ = write!(info, "'{printable}' U+{codepoint:04X}");
            if ch.is_ascii() {
                let _ = write!(info, " Dec {}", ch as u8);
            }
        }

        self.show_notification(info, NotificationSeverity::Info);
    }

    fn lsp_stop(&mut self) {
        let server_names: Vec<String> = {
            let (_view, doc) = helix_view::current_ref!(self.editor);
            doc.language_servers()
                .map(|ls| ls.name().to_string())
                .collect()
        };

        if server_names.is_empty() {
            self.show_notification("No language servers running".to_string(), NotificationSeverity::Info);
            return;
        }

        for ls_name in &server_names {
            self.editor.language_servers.stop(ls_name);

            for doc in self.editor.documents_mut() {
                if let Some(client) = doc.remove_language_server_by_name(ls_name) {
                    doc.clear_diagnostics_for_language_server(client.id());
                    doc.reset_all_inlay_hints();
                    doc.inlay_hints_oudated = true;
                }
            }
        }

        self.show_notification(
            format!("Stopped {} language server(s)", server_names.len()),
            NotificationSeverity::Info,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_completions_is_non_empty() {
        let cmds = command_completions();
        assert!(!cmds.is_empty());
    }

    #[test]
    fn command_completions_has_unique_names() {
        let cmds = command_completions();
        let mut names: Vec<&str> = cmds.iter().map(|c| c.name).collect();
        let count_before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), count_before, "Duplicate command names found");
    }

    #[test]
    fn command_completions_contains_common_commands() {
        let cmds = command_completions();
        let names: Vec<&str> = cmds.iter().map(|c| c.name).collect();
        assert!(names.contains(&"write"), "missing 'write'");
        assert!(names.contains(&"quit"), "missing 'quit'");
        assert!(names.contains(&"open"), "missing 'open'");
        assert!(names.contains(&"theme"), "missing 'theme'");
        assert!(names.contains(&"format"), "missing 'format'");
    }

    #[test]
    fn command_completions_have_descriptions() {
        for cmd in command_completions() {
            assert!(!cmd.description.is_empty(), "Command '{}' has no description", cmd.name);
        }
    }

    #[test]
    fn write_has_alias_w() {
        let write = command_completions()
            .iter()
            .find(|c| c.name == "write")
            .expect("write command should exist");
        assert!(write.aliases.contains(&"w"), "write should have alias 'w'");
    }
}
