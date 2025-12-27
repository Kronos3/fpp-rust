mod context;
mod dispatcher;
mod global_state;
mod handlers;
mod lsp_ext;
mod notification;
mod progress;
mod request;
mod semantic_tokens;
mod task;
mod util;

mod vfs;

pub use vfs::*;

use crate::global_state::GlobalState;
use lsp_server::Connection;
use lsp_types::{
    SemanticTokenModifier, SemanticTokenType, SemanticTokensFullOptions, SemanticTokensLegend,
    SemanticTokensOptions, SemanticTokensServerCapabilities, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind,
};
use std::error::Error;

use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, Layer,
};

fn setup_stderr_logging() -> anyhow::Result<()> {
    let stderr_log_level = tracing_subscriber::filter::LevelFilter::DEBUG;
    let stderr_layer = tracing_subscriber::fmt::layer()
    .with_writer(std::io::stderr);

    tracing_subscriber::registry()
        .with(
            stderr_layer
            .with_ansi(false)
            .without_time()
            .with_file(true)
            .with_line_number(true)
            .with_filter(stderr_log_level),
        )
        .try_init()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    setup_stderr_logging()?;

    // transport
    let (connection, io_thread) = Connection::stdio();

    // advertised capabilities
    let caps = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        // completion_provider: Some(CompletionOptions::default()),
        // definition_provider: Some(OneOf::Left(true)),
        // type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
        // hover_provider: Some(HoverProviderCapability::Simple(true)),
        // references_provider: Some(OneOf::Left(true)),
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
                        SemanticTokenType::EVENT,
                        SemanticTokenType::MODIFIER,
                        SemanticTokenType::KEYWORD,
                        SemanticTokenType::COMMENT,
                        SemanticTokenType::STRING,
                        SemanticTokenType::NUMBER,
                    ],
                    token_modifiers: vec![
                        SemanticTokenModifier::READONLY,
                        SemanticTokenModifier::DOCUMENTATION,
                    ],
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

    let _ = connection.initialize(init_value)?;

    tracing::info!("server is starting up");
    GlobalState::run(connection);
    io_thread.join()?;
    log::info!("shutting down server");

    Ok(())
}
