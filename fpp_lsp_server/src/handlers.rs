use crate::global_state::{GlobalState, GlobalStateSnapshot, Task};
use crate::lsp;
use crate::lsp::utils::semantic_token_delta;
use anyhow::Result;
use fpp_core::LineIndex;
use fpp_lsp_parser::{SyntaxKind, SyntaxNode, SyntaxToken, TextRange, VisitorResult};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticReportResult, DocumentLink, Position, Range, SemanticTokensFullDeltaResult,
    SemanticTokensRangeResult, SemanticTokensResult, Uri,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

    tracing::info!(uri = %uri_s, entry = ?entry_kind, "parsing document for semantic tokens");

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

// fn span_to_range(span: fpp_core::Span) -> Range {
//     let start = span.start();
//     let end = span.end();
//
//     Range {
//         start: Position {
//             line: start.line(),
//             character: start.column(),
//         },
//         end: Position {
//             line: end.line(),
//             character: end.column(),
//         },
//     }
// }

fn text_range_to_range(lines: &LineIndex, text_range: TextRange) -> Range {
    let start = lines.line_col(text_range.start());
    let end = lines.line_col(text_range.end());

    Range {
        start: Position {
            line: start.line,
            character: start.col,
        },
        end: Position {
            line: end.line,
            character: end.col,
        },
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentLinkData {
    pub origin_uri: Uri,
    pub relative_path: String,
}

struct DocumentLinksVisitor<'a> {
    uri: Uri,
    lines: &'a LineIndex,
    text: &'a str,
}
impl<'a> fpp_lsp_parser::Visitor for DocumentLinksVisitor<'a> {
    type State = Vec<DocumentLink>;

    fn visit_node(&self, state: &mut Self::State, node: &SyntaxNode) -> VisitorResult {
        match node.kind() {
            SyntaxKind::ROOT => VisitorResult::Recurse,
            SyntaxKind::DEF_MODULE => VisitorResult::Recurse,
            SyntaxKind::MODULE_MEMBER_LIST => VisitorResult::Recurse,
            SyntaxKind::DEF_COMPONENT => VisitorResult::Recurse,
            SyntaxKind::COMPONENT_MEMBER_LIST => VisitorResult::Recurse,
            SyntaxKind::DEF_STATE_MACHINE => VisitorResult::Recurse,
            SyntaxKind::STATE_MACHINE_MEMBER_LIST => VisitorResult::Recurse,
            SyntaxKind::DEF_STATE => VisitorResult::Recurse,
            SyntaxKind::STATE_MEMBER_LIST => VisitorResult::Recurse,
            SyntaxKind::DEF_TOPOLOGY => VisitorResult::Recurse,
            SyntaxKind::TOPOLOGY_MEMBER_LIST => VisitorResult::Recurse,
            SyntaxKind::TLM_PACKET_SET => VisitorResult::Recurse,
            SyntaxKind::TLM_PACKET_SET_MEMBER_LIST => VisitorResult::Recurse,
            SyntaxKind::SPEC_TLM_PACKET => VisitorResult::Recurse,
            SyntaxKind::TLM_PACKET_MEMBER_LIST => VisitorResult::Recurse,

            SyntaxKind::SPEC_INCLUDE => {
                // Get the string literal noting the file to include
                match node.first_child_or_token_by_kind(&|t| t == SyntaxKind::LITERAL_STRING) {
                    None => {}
                    Some(file) => {
                        // This token is either "file.fppi" or """file.fppi"""
                        // We need to strip off the quotes
                        let file_include = {
                            let file_include_text = &self.text[file.text_range()];
                            if file_include_text.starts_with("\"\"\"") {
                                if file_include_text.ends_with("\"\"\"") {
                                    &file_include_text[3..file_include_text.len() - 3]
                                } else {
                                    // This is some off-nominal case
                                    // Do something reasonable
                                    &file_include_text[3..]
                                }
                            } else {
                                &file_include_text[1..file_include_text.len() - 1]
                            }
                        };

                        let link = DocumentLink {
                            range: text_range_to_range(self.lines, file.text_range()),
                            target: None,
                            tooltip: None,
                            data: Some(
                                serde_json::to_value(DocumentLinkData {
                                    origin_uri: self.uri.clone(),
                                    relative_path: file_include.to_string(),
                                })
                                .unwrap(),
                            ),
                        };

                        state.push(link);
                    }
                }

                VisitorResult::Next
            }

            _ => VisitorResult::Next,
        }
    }

    fn visit_token(&self, _: &mut Self::State, _: &SyntaxToken) {}
}

pub fn handle_document_link_request(
    state: GlobalStateSnapshot,
    request: lsp_types::DocumentLinkParams,
) -> Result<Option<Vec<DocumentLink>>> {
    tracing::info!(uri = %request.text_document.uri.as_str(), "document link request");

    // TODO(tumbar) We probably don't need to run a reparse here
    let (text, parse) = parse_text_document(&state, &request.text_document.uri)?;
    let lines = state.vfs.get_lines(request.text_document.uri.as_str())?;

    let mut links = vec![];
    parse.visit(
        &mut links,
        &DocumentLinksVisitor {
            uri: request.text_document.uri.clone(),
            lines: &lines,
            text: &text,
        },
    );
    if links.is_empty() {
        Ok(None)
    } else {
        Ok(Some(links))
    }
}

pub fn handle_document_link_resolve(
    state: GlobalStateSnapshot,
    request: DocumentLink,
) -> Result<DocumentLink> {
    let data: DocumentLinkData = match request.data {
        None => return Err(anyhow::anyhow!("Document link has no data to resolve")),
        Some(data) => serde_json::from_value(data)?,
    };

    let resolved = state
        .vfs
        .resolve_uri_relative_path(data.origin_uri.as_str(), &data.relative_path)?;
    Ok(DocumentLink {
        range: request.range,
        target: Some(Uri::from_str(&resolved)?),
        tooltip: None,
        data: None,
    })
}
