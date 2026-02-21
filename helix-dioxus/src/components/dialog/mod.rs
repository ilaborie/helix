//! Dialog and prompt UI components.
//!
//! Components for user interaction: confirmation dialogs, input prompts,
//! LSP status, notifications, command completion, and command/search prompts.

mod command_completion;
mod confirmation;
mod input;
mod lsp_status;
mod prompt;

pub use command_completion::CommandCompletionPopup;
pub use confirmation::ConfirmationDialog;
pub use input::InputDialog;
pub use lsp_status::LspStatusDialog;
pub use prompt::{CommandPrompt, RegexPrompt, SearchPrompt, ShellPrompt};
