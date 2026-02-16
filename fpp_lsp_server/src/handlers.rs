use crate::global_state::{GlobalState, GlobalStateSnapshot, Task};
use crate::lsp;
use crate::lsp::utils::semantic_token_delta;
use anyhow::Result;
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticReportResult, SemanticTokensFullDeltaResult, SemanticTokensRangeResult,
    SemanticTokensResult, Uri,
};

pub fn handle_did_open_text_document(
    state: &mut GlobalState,
    not: DidOpenTextDocumentParams,
) -> Result<()> {
    tracing::info!(uri = %not.text_document.uri.as_str(), "DidOpenTextDocument");
    let uri = not.text_document.uri.clone();
    state.vfs.did_open(not);
    state.task(Task::Update(uri));
    Ok(())
}

pub fn handle_did_change_text_document(
    state: &mut GlobalState,
    not: DidChangeTextDocumentParams,
) -> Result<()> {
    tracing::info!(uri = %not.text_document.uri.as_str(), "DidChangeTextDocument");
    let uri = not.text_document.uri.clone();
    state
        .vfs
        .did_change(not, state.capabilities.negotiated_encoding());
    state.task(Task::Update(uri));
    Ok(())
}

pub fn handle_did_close_text_document(
    state: &mut GlobalState,
    not: DidCloseTextDocumentParams,
) -> Result<()> {
    tracing::info!(uri = %not.text_document.uri.as_str(), "DidCloseTextDocument");
    let uri = not.text_document.uri.clone();

    state
        .semantic_tokens
        .lock()
        .unwrap()
        .remove(&not.text_document.uri);

    state.vfs.did_close(not);
    state.task(Task::Update(uri));

    Ok(())
}

pub fn handle_exit(state: &mut GlobalState, _: ()) -> Result<()> {
    state.shutdown_requested = true;
    Ok(())
}

fn parse_text_document(
    state: &GlobalStateSnapshot,
    uri: &Uri,
) -> Result<(String, fpp_lsp_parser::Parse)> {
    let uri_c = &uri.clone();
    let uri_s = uri_c.as_str();
    let text: String = state.vfs.read_sync(uri_s)?;

    let parse_kind = match state.files.get(uri_s) {
        None => fpp_parser::IncludeParentKind::Module,
        Some(source_files) => {
            // This file may have been included in multiple spots
            // We should choose the most 'permissive' syntax entry point
            source_files
                .iter()
                .map(|f| match state.analysis.include_context_map.get(f) {
                    Some(kind) => kind.clone(),
                    None => fpp_parser::IncludeParentKind::Module,
                })
                .max()
                .unwrap_or(fpp_parser::IncludeParentKind::Module)
        }
    };

    let entry_kind = match parse_kind {
        fpp_parser::IncludeParentKind::Component => fpp_lsp_parser::TopEntryPoint::Component,
        fpp_parser::IncludeParentKind::Module => fpp_lsp_parser::TopEntryPoint::Module,
        fpp_parser::IncludeParentKind::TlmPacket => fpp_lsp_parser::TopEntryPoint::TlmPacket,
        fpp_parser::IncludeParentKind::TlmPacketSet => fpp_lsp_parser::TopEntryPoint::TlmPacketSet,
        fpp_parser::IncludeParentKind::Topology => fpp_lsp_parser::TopEntryPoint::Topology,
    };

    let parse = fpp_lsp_parser::parse(&text, entry_kind);
    Ok((text, parse))
}

pub fn handle_semantic_tokens_full(
    state: GlobalStateSnapshot,
    request: lsp_types::SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    tracing::info!(uri = %request.text_document.uri.as_str(), "SemanticTokens");

    // TODO(tumbar) We probably don't need to run a reparse here
    let (text, parse) = parse_text_document(&state, &request.text_document.uri)?;
    let semantic_tokens = lsp::semantic_tokens::compute(&text, &parse).finish(None);

    // Unconditionally cache the tokens
    state
        .semantic_tokens
        .lock()
        .unwrap()
        .insert(request.text_document.uri, semantic_tokens.clone());

    Ok(Some(semantic_tokens.into()))
}

pub fn handle_semantic_tokens_range(
    state: GlobalStateSnapshot,
    request: lsp_types::SemanticTokensRangeParams,
) -> Result<Option<SemanticTokensRangeResult>> {
    tracing::info!(uri = %request.text_document.uri.as_str(), "SemanticTokens");

    // TODO(tumbar) We probably don't need to run a reparse here
    let (text, parse) = parse_text_document(&state, &request.text_document.uri)?;

    Ok(Some(SemanticTokensRangeResult::Tokens(
        lsp::semantic_tokens::compute(&text, &parse).finish(Some(request.range)),
    )))
}

pub fn handle_semantic_tokens_full_delta(
    state: GlobalStateSnapshot,
    request: lsp_types::SemanticTokensDeltaParams,
) -> Result<Option<SemanticTokensFullDeltaResult>> {
    tracing::info!(uri = %request.text_document.uri.as_str(), "SemanticTokens");

    // TODO(tumbar) We probably don't need to run a reparse here
    let (text, parse) = parse_text_document(&state, &request.text_document.uri)?;

    let semantic_tokens = lsp::semantic_tokens::compute(&text, &parse).finish(None);

    let cached_tokens = state
        .semantic_tokens
        .lock()
        .unwrap()
        .remove(&request.text_document.uri);

    if let Some(
        cached_tokens @ lsp_types::SemanticTokens {
            result_id: Some(prev_id),
            ..
        },
    ) = &cached_tokens
        && *prev_id == request.previous_result_id
    {
        let delta = semantic_token_delta(cached_tokens, &semantic_tokens);
        state
            .semantic_tokens
            .lock()
            .unwrap()
            .insert(request.text_document.uri, semantic_tokens);
        return Ok(Some(delta.into()));
    }

    // Clone first to keep the lock short
    let semantic_tokens_clone = semantic_tokens.clone();
    state
        .semantic_tokens
        .lock()
        .unwrap()
        .insert(request.text_document.uri, semantic_tokens_clone);

    Ok(Some(semantic_tokens.into()))
}

pub fn handle_document_diagnostics(
    state: GlobalStateSnapshot,
    request: lsp_types::DocumentDiagnosticParams,
) -> Result<DocumentDiagnosticReportResult> {
    tracing::info!(uri = %request.text_document.uri.as_str(), "document diagnostics");

    Ok(DocumentDiagnosticReportResult::Report(
        lsp_types::DocumentDiagnosticReport::Full(lsp_types::RelatedFullDocumentDiagnosticReport {
            full_document_diagnostic_report: lsp_types::FullDocumentDiagnosticReport {
                items: state
                    .diagnostics
                    .lock()
                    .unwrap()
                    .get(&request.text_document.uri.as_str()),
                ..Default::default()
            },
            ..Default::default()
        }),
    ))
}
