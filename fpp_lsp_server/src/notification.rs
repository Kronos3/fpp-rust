use crate::dispatcher::NotificationDispatcher;
use crate::global_state::GlobalState;
use crate::handlers;
use lsp_server::Notification;

impl GlobalState {
    /// Handles a request.
    pub(crate) fn on_notification(&mut self, not: Notification) {
        let mut dispatcher = NotificationDispatcher {
            not: Some(not),
            global_state: self,
        };

        use lsp_types::notification as lsp_notification;

        #[rustfmt::skip]
        dispatcher
            .on_sync_mut::<lsp_notification::DidOpenTextDocument>(handlers::handle_did_open_text_document)
            .on_sync_mut::<lsp_notification::DidChangeTextDocument>(handlers::handle_did_change_text_document)
            .on_sync_mut::<lsp_notification::DidCloseTextDocument>(handlers::handle_did_close_text_document)
            .on_sync_mut::<lsp_notification::Exit>(handlers::handle_exit)
            .finish();
    }
}
