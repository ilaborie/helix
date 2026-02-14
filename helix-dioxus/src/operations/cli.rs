//! CLI command execution operations.

use std::path::PathBuf;
use std::sync::Arc;

use crate::operations::{
    BufferOps, EditingOps, JumpOps, PickerOps, ShellOps, TextManipulationOps, ThemeOps,
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
                        self.show_notification(
                            "Usage: :set <key> <value>".to_string(),
                            NotificationSeverity::Warning,
                        );
                    }
                }
                None => {
                    self.show_notification(
                        "Usage: :set <key> <value>".to_string(),
                        NotificationSeverity::Warning,
                    );
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
                            .map(|config| {
                                config
                                    .language_servers
                                    .iter()
                                    .map(|ls| ls.name.clone())
                                    .collect()
                            })
                            .unwrap_or_default()
                    };
                    for name in &server_names {
                        self.restart_lsp_server(name);
                    }
                }
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
                log::warn!("Unknown command: {cmd}");
                self.show_notification(
                    format!("Unknown command: {cmd}"),
                    NotificationSeverity::Error,
                );
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
                self.show_notification(
                    format!("Language config error: {err}"),
                    NotificationSeverity::Warning,
                );
            }
        }

        // 3. Reload theme if changed in config.toml
        if let Some(theme_name) = crate::state::load_theme_name() {
            if theme_name != self.editor.theme.name() {
                if let Err(err) = self.apply_theme(&theme_name) {
                    log::warn!("Failed to apply theme '{theme_name}': {err}");
                    self.show_notification(
                        format!("Theme error: {err}"),
                        NotificationSeverity::Warning,
                    );
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
            self.show_notification(
                format!("Unknown config key: {key}"),
                NotificationSeverity::Error,
            );
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
                self.show_notification(
                    format!("Invalid config value: {err}"),
                    NotificationSeverity::Error,
                );
                return;
            }
        };

        // Apply the new config
        self.editor.config = Arc::new(arc_swap::ArcSwap::from_pointee(new_config));
        self.editor.refresh_config(&old_config);
        self.show_notification(format!("Set {key} = {value}"), NotificationSeverity::Info);
    }

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
            self.show_notification(
                format!("Unknown config key: {key}"),
                NotificationSeverity::Error,
            );
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
                self.show_notification(
                    format!("Invalid config value: {err}"),
                    NotificationSeverity::Error,
                );
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
        self.show_notification(
            format!("Toggled {key} = {display_val}"),
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
            assert!(
                !cmd.description.is_empty(),
                "Command '{}' has no description",
                cmd.name
            );
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
