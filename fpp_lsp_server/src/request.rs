use crate::dispatcher::RequestDispatcher;
use crate::global_state::GlobalState;
use crate::{handlers, lsp_ext};
use lsp_server::Request;

impl GlobalState {
    /// Handles a request.
    pub(crate) fn on_request(&mut self, req: Request) {
        let mut dispatcher = RequestDispatcher {
            req: Some(req),
            global_state: self,
        };

        use lsp_types::request as lsp_request;

        #[rustfmt::skip]
        dispatcher
            .on_sync_mut::<lsp_request::Shutdown>(|s, ()| {
                s.shutdown_requested = true;
                Ok(())
            })
            // Request handlers that must run on the main thread
            // because they mutate GlobalState:
            .on_sync_mut::<lsp_ext::ReloadWorkspace>(handlers::handle_workspace_reload)
            // .on_sync::<lsp_request::SelectionRangeRequest>(handlers::handle_selection_range)
            // .on::<lsp_request::Completion>(handlers::handle_completion)
            // .on::<lsp_request::ResolveCompletionItem>(handlers::handle_completion_resolve)
            .on::<lsp_request::SemanticTokensFullRequest>(handlers::handle_semantic_tokens_full)
            .on::<lsp_request::SemanticTokensFullDeltaRequest>(handlers::handle_semantic_tokens_full_delta)
            .on::<lsp_request::SemanticTokensRangeRequest>(handlers::handle_semantic_tokens_range)
            .on::<lsp_request::DocumentDiagnosticRequest>(handlers::handle_document_diagnostics)
            // .on::<lsp_request::DocumentSymbolRequest>(handlers::handle_document_symbol)
            // .on::<lsp_request::FoldingRangeRequest>(handlers::handle_folding_range)
            // .on::<lsp_request::SignatureHelpRequest>(handlers::handle_signature_help)
            // .on::<lsp_request::WillRenameFiles>(handlers::handle_will_rename_files)
            // .on::<lsp_request::GotoDefinition>(handlers::handle_goto_definition)
            // .on::<lsp_request::GotoDeclaration>(handlers::handle_goto_declaration)
            // .on::<lsp_request::GotoImplementation>(handlers::handle_goto_implementation)
            // .on::<lsp_request::GotoTypeDefinition>(handlers::handle_goto_type_definition)
            // .on::<lsp_request::InlayHintRequest>(handlers::handle_inlay_hints)
            // .on_identity::<lsp_request::InlayHintResolveRequest, _>(handlers::handle_inlay_hints_resolve)
            // .on::<lsp_request::CodeLensRequest>(handlers::handle_code_lens)
            // .on_identity::<NO_RETRY, lsp_request::CodeLensResolve, _>(handlers::handle_code_lens_resolve)
            // .on::<lsp_request::PrepareRenameRequest>(handlers::handle_prepare_rename)
            // .on::<lsp_request::Rename>(handlers::handle_rename)
            // .on::<lsp_request::References>(handlers::handle_references)
            // .on::<lsp_request::DocumentHighlightRequest>(handlers::handle_document_highlight)
            // .on::<lsp_request::CallHierarchyPrepare>(handlers::handle_call_hierarchy_prepare)
            // .on::<lsp_request::CallHierarchyIncomingCalls>(handlers::handle_call_hierarchy_incoming)
            // .on::<lsp_request::CallHierarchyOutgoingCalls>(handlers::handle_call_hierarchy_outgoing)
            .finish();
    }
}
