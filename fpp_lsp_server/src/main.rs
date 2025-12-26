mod context;
mod dispatcher;
mod global_state;
mod handlers;
mod lsp_ext;
mod notification;
mod progress;
mod request;
mod semantic_tokens;
mod util;
mod task;

mod vfs;

pub use vfs::*;

use crate::context::{LspContext, LspDiagnosticsEmitter};
use fpp_core::CompilerContext;
use lsp_server::{Connection, Response};
use lsp_types::notification::{
    DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument,
    Notification,
};
use lsp_types::{
    CompletionItem, CompletionOptions, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DocumentLinkOptions, HoverProviderCapability, InitializeParams, OneOf, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, TypeDefinitionProviderCapability, Uri,
};
use std::cell::RefCell;
use std::error::Error;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    // transport
    let (connection, io_thread) = Connection::stdio();

    // advertised capabilities
    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(CompletionOptions::default()),
        definition_provider: Some(OneOf::Left(true)),
        type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        references_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                work_done_progress_options: Default::default(),
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::NAMESPACE,
                        SemanticTokenType::TYPE,
                        SemanticTokenType::ENUM,
                        SemanticTokenType::CLASS,
                        SemanticTokenType::INTERFACE,
                        SemanticTokenType::STRUCT,
                        SemanticTokenType::PARAMETER,
                        SemanticTokenType::VARIABLE,
                        SemanticTokenType::ENUM_MEMBER,
                        SemanticTokenType::FUNCTION,
                        SemanticTokenType::KEYWORD,
                        SemanticTokenType::COMMENT,
                        SemanticTokenType::STRING,
                        SemanticTokenType::NUMBER,
                    ],
                    token_modifiers: vec![],
                },
                range: None,
                full: Some(SemanticTokensFullOptions::Bool(true)),
            },
        )),
        // document_formatting_provider: Some(OneOf::Left(true)),
        ..Default::default()
    };

    let init_value = serde_json::json!({
        "capabilities": caps,
        "offsetEncoding": ["utf-8"],
    });

    let init_params = connection.initialize(init_value)?;

    let diagnostics = Arc::new(RefCell::new(LspDiagnosticsEmitter::new()));
    let mut compiler_ctx = CompilerContext::new(diagnostics.clone());
    let mut lsp_ctx = LspContext::new(diagnostics.clone());

    fpp_core::run(&mut compiler_ctx, || {
        main_loop(&mut lsp_ctx, connection, init_params)
    })??;

    io_thread.join()?;
    log::error!("shutting down server");
    Ok(())
}
