//! LSP event handling for EditorContext.
//!
//! This module handles incoming LSP notifications and method calls,
//! including diagnostics, progress messages, and other LSP events.

use helix_lsp::lsp;

use super::EditorContext;

/// LSP event handling operations for EditorContext.
pub trait LspEventOps {
    /// Poll for LSP events non-blockingly and handle them.
    fn poll_lsp_events(&mut self);

    /// Handle an LSP message (notification or method call).
    fn handle_lsp_message(&mut self, server_id: helix_lsp::LanguageServerId, call: helix_lsp::Call);

    /// Handle publishDiagnostics notification from LSP.
    fn handle_publish_diagnostics(
        &mut self,
        server_id: helix_lsp::LanguageServerId,
        params: lsp::PublishDiagnosticsParams,
    );

    /// Handle progress message from LSP.
    fn handle_progress_message(
        &mut self,
        server_id: helix_lsp::LanguageServerId,
        params: lsp::ProgressParams,
    );
}

impl LspEventOps for EditorContext {
    fn poll_lsp_events(&mut self) {
        use futures::stream::StreamExt;
        use std::task::{Context, Poll};

        // Create a noop waker for non-blocking polling
        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Poll the incoming stream for LSP messages
        loop {
            let incoming = &mut self.editor.language_servers.incoming;
            match incoming.poll_next_unpin(&mut cx) {
                Poll::Ready(Some((server_id, call))) => {
                    self.handle_lsp_message(server_id, call);
                }
                Poll::Ready(None) | Poll::Pending => break,
            }
        }
    }

    fn handle_lsp_message(
        &mut self,
        server_id: helix_lsp::LanguageServerId,
        call: helix_lsp::Call,
    ) {
        use helix_lsp::{Call, Notification};

        match call {
            Call::Notification(notification) => {
                let method = &notification.method;
                let params = notification.params;

                match Notification::parse(method, params) {
                    Ok(Notification::PublishDiagnostics(params)) => {
                        self.handle_publish_diagnostics(server_id, params);
                    }
                    Ok(Notification::ProgressMessage(params)) => {
                        self.handle_progress_message(server_id, params);
                    }
                    Ok(Notification::Initialized) => {
                        log::info!("LSP server {server_id:?} initialized");

                        // Send didChangeConfiguration if the server has config
                        if let Some(ls) = self.editor.language_servers.get_by_id(server_id) {
                            if let Some(config) = ls.config() {
                                ls.did_change_configuration(config.clone());
                            }
                        }

                        // Dispatch the event so hooks can send textDocument/didOpen for all documents
                        helix_event::dispatch(helix_view::events::LanguageServerInitialized {
                            editor: &mut self.editor,
                            server_id,
                        });
                    }
                    Ok(Notification::Exit) => {
                        log::info!("LSP server {server_id:?} exited");

                        // Dispatch the event so hooks can clean up
                        helix_event::dispatch(helix_view::events::LanguageServerExited {
                            editor: &mut self.editor,
                            server_id,
                        });

                        // Remove the language server from the registry
                        self.editor.language_servers.remove_by_id(server_id);
                    }
                    Ok(notification) => {
                        log::trace!("Unhandled LSP notification: {notification:?}");
                    }
                    Err(err) => {
                        log::warn!("Failed to parse LSP notification {method}: {err}");
                    }
                }
            }
            Call::MethodCall(method_call) => {
                // Handle method calls that require a response
                log::trace!(
                    "Received LSP method call: {} (id: {:?})",
                    method_call.method,
                    method_call.id
                );
            }
            Call::Invalid { id } => {
                log::error!("Invalid LSP call id={id:?}");
            }
        }
    }

    fn handle_publish_diagnostics(
        &mut self,
        server_id: helix_lsp::LanguageServerId,
        params: lsp::PublishDiagnosticsParams,
    ) {
        let uri = match helix_core::Uri::try_from(params.uri.clone()) {
            Ok(uri) => uri,
            Err(err) => {
                log::error!("Invalid URI in publishDiagnostics: {err}");
                return;
            }
        };

        // Check if the language server is initialized
        let Some(ls) = self.editor.language_servers.get_by_id(server_id) else {
            log::warn!("Received diagnostics from unknown server {server_id:?}");
            return;
        };

        if !ls.is_initialized() {
            log::warn!(
                "Discarding diagnostics from uninitialized server: {}",
                ls.name()
            );
            return;
        }

        let provider = helix_core::diagnostic::DiagnosticProvider::Lsp {
            server_id,
            identifier: None,
        };

        log::info!(
            "Received {} diagnostics for {:?}",
            params.diagnostics.len(),
            params.uri
        );

        // Log first few diagnostics for debugging
        for (i, diag) in params.diagnostics.iter().take(3).enumerate() {
            log::info!(
                "  Diagnostic {}: line {}, message: {}",
                i,
                diag.range.start.line,
                &diag.message[..diag.message.len().min(60)]
            );
        }

        self.editor
            .handle_lsp_diagnostics(&provider, uri, params.version, params.diagnostics);
    }

    fn handle_progress_message(
        &mut self,
        server_id: helix_lsp::LanguageServerId,
        params: lsp::ProgressParams,
    ) {
        let token = params.token;
        let lsp::ProgressParamsValue::WorkDone(work) = params.value;

        match work {
            lsp::WorkDoneProgress::Begin(begin) => {
                log::info!(
                    "LSP progress begin: {} (server {:?})",
                    begin.title,
                    server_id
                );
                self.lsp_progress.begin(server_id, token, begin);
            }
            lsp::WorkDoneProgress::Report(report) => {
                if let Some(msg) = &report.message {
                    log::trace!("LSP progress: {msg}");
                }
                self.lsp_progress.update(server_id, token, report);
            }
            lsp::WorkDoneProgress::End(end) => {
                log::info!("LSP progress end (server {server_id:?})");
                self.lsp_progress.end_progress(server_id, &token);
                // Log the message if present
                if let Some(msg) = end.message {
                    log::info!("LSP progress completed: {msg}");
                }
            }
        }
    }
}
