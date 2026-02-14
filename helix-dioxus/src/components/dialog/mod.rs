//! Dialog and prompt UI components.
//!
//! Components for user interaction: confirmation dialogs, input prompts,
//! LSP status, notifications, and command/search prompts.

mod confirmation;
mod input;
mod lsp_status;
mod notification;
mod prompt;

pub use confirmation::ConfirmationDialog;
pub use input::InputDialog;
pub use lsp_status::LspStatusDialog;
pub use notification::NotificationContainer;
pub use prompt::{CommandPrompt, RegexPrompt, SearchPrompt};
