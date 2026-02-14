//! Shell integration operations.
//!
//! Provides the `|`, `!`, `A-|`, `A-!` commands that pipe selections through
//! shell commands.

use std::io::Write;
use std::process::{Command, Stdio};

use helix_core::Transaction;
use helix_view::DocumentId;

use crate::state::{EditorContext, NotificationSeverity, ShellBehavior};

/// Extension trait for shell operations.
pub trait ShellOps {
    /// Execute the current shell command on each selection.
    fn execute_shell_command(&mut self, doc_id: DocumentId, view_id: helix_view::ViewId);
}

impl ShellOps for EditorContext {
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

        let doc = match self.editor.document(doc_id) {
            Some(doc) => doc,
            None => return,
        };
        let text = doc.text().clone();
        let selection = doc.selection(view_id).clone();

        // Process each selection range
        let mut changes: Vec<(usize, usize, Option<helix_core::Tendril>)> = Vec::new();

        for range in selection.iter() {
            let from = range.from();
            let to = range.to();
            let selection_text: String = text.slice(from..to).into();
            let had_trailing_newline = selection_text.ends_with('\n');

            // Build the shell command
            let mut cmd = Command::new(&shell[0]);
            for arg in &shell[1..] {
                cmd.arg(arg);
            }
            cmd.arg(&command_str);

            // Set up stdin/stdout based on behavior
            let needs_stdin = matches!(behavior, ShellBehavior::Replace | ShellBehavior::Ignore);
            if needs_stdin {
                cmd.stdin(Stdio::piped());
            } else {
                cmd.stdin(Stdio::null());
            }
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let result = cmd.spawn().and_then(|mut child| {
                // Write selection text to stdin if needed
                if needs_stdin {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(selection_text.as_bytes());
                        // Drop stdin to signal EOF
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

                    match behavior {
                        ShellBehavior::Replace => {
                            changes.push((from, to, Some(stdout.into())));
                        }
                        ShellBehavior::Insert => {
                            // Insert stdout before the selection (at `from`, empty delete)
                            changes.push((from, from, Some(stdout.into())));
                        }
                        ShellBehavior::Append => {
                            // Append stdout after the selection (at `to`, empty delete)
                            changes.push((to, to, Some(stdout.into())));
                        }
                        ShellBehavior::Ignore => {
                            // Discard stdout, keep selection unchanged
                        }
                    }
                }
                Err(e) => {
                    self.show_notification(
                        format!("Failed to run shell command: {e}"),
                        NotificationSeverity::Error,
                    );
                    return;
                }
            }
        }

        // Apply changes via transaction
        if !changes.is_empty() {
            let doc = self.editor.document_mut(doc_id).expect("doc exists");
            let transaction = Transaction::change(doc.text(), changes.into_iter());
            doc.apply(&transaction, view_id);
        }
    }
}
