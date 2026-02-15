//! Shell integration operations.
//!
//! Provides the `|`, `!`, `A-|`, `A-!` commands that pipe selections through
//! shell commands.

use std::io::Write;
use std::process::{Command, Stdio};

use helix_core::{Range, Selection, Transaction};
use helix_view::DocumentId;

use crate::state::{EditorContext, NotificationSeverity, ShellBehavior};

/// Extension trait for shell operations.
pub trait ShellOps {
    /// Execute the current shell command on each selection.
    fn execute_shell_command(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId);
}

impl ShellOps for EditorContext {
    #[allow(clippy::indexing_slicing)] // shell[] access is guarded by len >= 2 check above
    fn execute_shell_command(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId) {
        let command_str = self.shell_input.trim().to_string();
        if command_str.is_empty() {
            return;
        }
        let behavior = self.shell_behavior;

        // Get the shell from editor config (default: ["sh", "-c"])
        let shell = self.editor.config().shell.clone();
        if shell.len() < 2 {
            self.show_notification(
                "Invalid shell config: expected [\"sh\", \"-c\"]".to_string(),
                NotificationSeverity::Error,
            );
            return;
        }

        let Some(doc) = self.editor.document(doc_id) else {
            return;
        };
        let text = doc.text().clone();
        let selection = doc.selection(view_id).clone();

        let pipe = matches!(behavior, ShellBehavior::Replace | ShellBehavior::Ignore);

        // Process each selection range, tracking offset for selection repositioning
        let mut changes: Vec<(usize, usize, Option<helix_core::Tendril>)> = Vec::new();
        let mut ranges: Vec<Range> = Vec::with_capacity(selection.len());
        let mut offset = 0_isize;
        let mut shell_output: Option<helix_core::Tendril> = None;

        for range in &selection {
            let from = range.from();
            let to = range.to();
            let selection_text: String = text.slice(from..to).into();
            let had_trailing_newline = selection_text.ends_with('\n');

            // For Insert/Append, reuse the same output for all ranges
            let output = if let Some(ref cached) = shell_output {
                cached.clone()
            } else {
                // Build the shell command
                let mut cmd = Command::new(&shell[0]);
                for arg in &shell[1..] {
                    cmd.arg(arg);
                }
                cmd.arg(&command_str);

                if pipe {
                    cmd.stdin(Stdio::piped());
                } else {
                    cmd.stdin(Stdio::null());
                }
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let result = cmd.spawn().and_then(|mut child| {
                    if pipe {
                        if let Some(mut stdin) = child.stdin.take() {
                            let _ = stdin.write_all(selection_text.as_bytes());
                        }
                    }
                    child.wait_with_output()
                });

                match result {
                    Ok(output) => {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let msg = if stderr.is_empty() {
                                format!("Shell command failed with exit code {}", output.status)
                            } else {
                                format!("Shell error: {}", stderr.trim())
                            };
                            self.show_notification(msg, NotificationSeverity::Error);
                            return;
                        }

                        let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();

                        // Strip trailing newline from output when input didn't have one
                        if !had_trailing_newline && stdout.ends_with('\n') {
                            stdout.pop();
                            if stdout.ends_with('\r') {
                                stdout.pop();
                            }
                        }

                        let tendril: helix_core::Tendril = stdout.into();
                        if !pipe {
                            shell_output = Some(tendril.clone());
                        }
                        tendril
                    }
                    Err(e) => {
                        self.show_notification(
                            format!("Failed to run shell command: {e}"),
                            NotificationSeverity::Error,
                        );
                        return;
                    }
                }
            };

            let output_len = output.chars().count();

            let (change_from, change_to, deleted_len) = match behavior {
                ShellBehavior::Replace => (from, to, range.len()),
                ShellBehavior::Insert | ShellBehavior::Ignore => (from, from, 0),
                ShellBehavior::Append => (to, to, 0),
            };

            // Compute new selection range with offset tracking (matching helix-term)
            let anchor = change_to
                .checked_add_signed(offset)
                .expect("selection ranges cannot overlap")
                .checked_sub(deleted_len)
                .expect("selection ranges cannot overlap");
            let new_range = Range::new(anchor, anchor + output_len).with_direction(range.direction());
            ranges.push(new_range);
            offset = offset
                .checked_add_unsigned(output_len)
                .expect("selection ranges cannot overlap")
                .checked_sub_unsigned(deleted_len)
                .expect("selection ranges cannot overlap");

            if behavior != ShellBehavior::Ignore {
                changes.push((change_from, change_to, Some(output)));
            }
        }

        // Apply changes via transaction with explicit selection
        if !changes.is_empty() {
            let doc = self.editor.document_mut(doc_id).expect("doc exists");
            let transaction = Transaction::change(doc.text(), changes.into_iter())
                .with_selection(Selection::new(ranges.into(), selection.primary_index()));
            doc.apply(&transaction, view_id);
        }
    }
}
