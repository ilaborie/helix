//! CLI command execution operations.

use std::path::PathBuf;

use crate::operations::{BufferOps, PickerOps};
use crate::state::EditorContext;

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

            _ => {
                log::warn!("Unknown command: {}", cmd);
            }
        }

        self.command_mode = false;
        self.command_input.clear();
    }
}
