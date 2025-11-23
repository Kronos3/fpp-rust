mod context;
mod dispatcher;
mod handlers;
mod reader;
mod semantic_tokens;
mod global_state;

use crate::context::{LspContext, LspDiagnosticsEmitter};
use fpp_analysis::Analysis;
use fpp_ast::TransUnit;
use fpp_core::{CompilerContext, SourceFile};
use fpp_fs::FsReader;
use lsp_server::{Connection, Message, Request as ServerRequest, RequestId, Response};
use lsp_types::notification::{
    DidChangeTextDocument, DidChangeWatchedFiles, DidOpenTextDocument, Notification,
};
use lsp_types::request::*;
use lsp_types::{
    CompletionItem, CompletionOptions, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    DocumentLinkOptions, HoverProviderCapability, InitializeParams, OneOf, SemanticTokenType,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensServerCapabilities,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TypeDefinitionProviderCapability, Uri,
};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

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
                full: None,
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

    let diagnostics = Rc::new(RefCell::new(LspDiagnosticsEmitter::new()));
    let mut compiler_ctx = CompilerContext::new(diagnostics.clone());
    let mut lsp_ctx = LspContext::new(diagnostics.clone());

    fpp_core::run(&mut compiler_ctx, || {
        main_loop(&mut lsp_ctx, connection, init_params)
    })??;

    io_thread.join()?;
    log::error!("shutting down server");
    Ok(())
}

// =====================================================================
// event loop
// =====================================================================

fn main_loop(
    ctx: &mut LspContext,
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let init: InitializeParams = serde_json::from_value(params)?;
    log::info!(
        "[lsp] initializing with options: {:?}",
        init.initialization_options
    );

    // Keep track of the ASTs we have cached
    let mut docs: FxHashMap<SourceFile, fpp_ast::TransUnit> = Default::default();
    loop {
        match main_loop_ast(&connection, &mut docs)? {
            None => break,
            Some(reprocess) => {
                docs.remove(&reprocess);
            }
        }
    }

    Ok(())
}

enum Reprocess {
    /// Shutdown the server
    Shutdown,

    Remove,
}

fn main_loop_ast(
    connection: &Connection,
    docs: &FxHashMap<SourceFile, TransUnit>,
) -> Result<Option<SourceFile>, Box<dyn Error + Sync + Send>> {
    let mut analysis = Analysis::new();

    // Combine all translation units into a single unit
    let mut tu = TransUnit(docs.values().map(|v| v.0).flatten().collect());

    let reader = Box::new(FsReader {});
    let _ = fpp_analysis::passes::check_semantics(&mut analysis, reader, &mut tu);

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    break;
                }
                if let Err(err) = handle_request(&connection, &req, &mut docs) {
                    log::error!("[lsp] request {} failed: {err}", &req.method);
                }
            }
            Message::Notification(note) => {
                if let Err(err) = handle_notification(&connection, &note, &mut docs) {
                    log::error!("[lsp] notification {} failed: {err}", note.method);
                }
            }
            Message::Response(resp) => log::error!("[lsp] response: {resp:?}"),
        }
    }

    // Shutdown
    Ok(None)
}

// =====================================================================
// notifications
// =====================================================================

fn handle_notification(
    conn: &Connection,
    note: &lsp_server::Notification,
    ctx: &mut LspContext,
) -> Result<(), E> {
    match note.method.as_str() {
        DidOpenTextDocument::METHOD => {
            let p: DidOpenTextDocumentParams = serde_json::from_value(note.params.clone())?;
            let uri = p.text_document.uri;
            publish_dummy_diag(conn, &uri)?;
        }
        DidChangeTextDocument::METHOD => {
            let p: DidChangeTextDocumentParams = serde_json::from_value(note.params.clone())?;
            if let Some(change) = p.content_changes.into_iter().next() {
                let uri = p.text_document.uri;
                ctx.update(&uri, change.text);
                publish_dummy_diag(conn, &uri)?;
            }
        }
        DidChangeWatchedFiles::METHOD => {}
        _ => {}
    }
    Ok(())
}

fn handle_request(
    conn: &Connection,
    req: &ServerRequest,
    docs: &mut FxHashMap<Uri, String>,
) -> Result<()> {
    match req.method.as_str() {
        GotoDefinition::METHOD => {
            send_ok(
                conn,
                req.id.clone(),
                &lsp_types::GotoDefinitionResponse::Array(Vec::new()),
            )?;
        }
        Completion::METHOD => {
            let item = CompletionItem {
                label: "HelloFromLSP".into(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("dummy completion".into()),
                ..Default::default()
            };
            send_ok(conn, req.id.clone(), &CompletionResponse::Array(vec![item]))?;
        }
        HoverRequest::METHOD => {
            let hover = Hover {
                contents: HoverContents::Scalar(MarkedString::String(
                    "Hello from *minimal_lsp*".into(),
                )),
                range: None,
            };
            send_ok(conn, req.id.clone(), &hover)?;
        }
        Formatting::METHOD => {
            let p: DocumentFormattingParams = serde_json::from_value(req.params.clone())?;
            let uri = p.text_document.uri;
            let text = docs
                .get(&uri)
                .ok_or_else(|| anyhow!("document not in cache – did you send DidOpen?"))?;
            let formatted = run_rustfmt(text)?;
            let edit = TextEdit {
                range: full_range(text),
                new_text: formatted,
            };
            send_ok(conn, req.id.clone(), &vec![edit])?;
        }
        _ => send_err(
            conn,
            req.id.clone(),
            lsp_server::ErrorCode::MethodNotFound,
            "unhandled method",
        )?,
    }
    Ok(())
}

// =====================================================================
// diagnostics
// =====================================================================
fn publish_dummy_diag(conn: &Connection, uri: &Url) -> Result<()> {
    let diag = Diagnostic {
        range: Range::new(Position::new(0, 0), Position::new(0, 1)),
        severity: Some(DiagnosticSeverity::INFORMATION),
        code: None,
        code_description: None,
        source: Some("minimal_lsp".into()),
        message: "dummy diagnostic".into(),
        related_information: None,
        tags: None,
        data: None,
    };
    let params = PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: vec![diag],
        version: None,
    };
    conn.sender
        .send(Message::Notification(lsp_server::Notification::new(
            PublishDiagnostics::METHOD.to_owned(),
            params,
        )))?;
    Ok(())
}
