use crate::global_state::{GlobalState, Task};
use crate::lsp;
use crate::lsp::utils::semantic_token_delta;
use crate::util::{
    hover_for_node, hover_for_symbol, node_to_location, nodes_at_offset, position_to_offset,
    symbol_at_position, symbol_to_completion_item,
};
use anyhow::Result;
use fpp_analysis::semantics::{NameGroup, SymbolInterface};
use fpp_ast::{AstNode, Node};
use fpp_core::{LineCol, LineIndex};
use fpp_lsp_parser::{
    SyntaxKind, SyntaxNode, SyntaxToken, TextRange, TokenAtOffset, VisitorResult,
};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticReportResult, DocumentLink, GotoDefinitionParams, GotoDefinitionResponse,
    Hover, HoverParams, Location, Position, Range, ReferenceParams, SemanticTokensFullDeltaResult,
    SemanticTokensRangeResult, SemanticTokensResult, Uri,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub fn handle_did_open_text_document(
    state: &mut GlobalState,
    not: DidOpenTextDocumentParams,
) -> Result<()> {
    let uri = not.text_document.uri.clone();
    state.vfs.did_open(not);
    state.task(Task::Update(uri));
    Ok(())
}

pub fn handle_did_change_text_document(
    state: &mut GlobalState,
    not: DidChangeTextDocumentParams,
) -> Result<()> {
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
    let uri = not.text_document.uri.clone();

    state.semantic_tokens.remove(&not.text_document.uri);

    state.vfs.did_close(not);
    state.task(Task::Update(uri));

    Ok(())
}

pub fn handle_exit(state: &mut GlobalState, _: ()) -> Result<()> {
    state.shutdown_requested = true;
    Ok(())
}

fn parse_text_document(state: &GlobalState, uri: &Uri) -> Result<(String, fpp_lsp_parser::Parse)> {
    let text: String = state.vfs.read_sync(uri.as_str())?;

    let parse_kind = match state.files.get(uri.as_str()) {
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
    state: &mut GlobalState,
    request: lsp_types::SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    // TODO(tumbar) We probably don't need to run a reparse here
    let (text, parse) = parse_text_document(&state, &request.text_document.uri)?;
    let semantic_tokens = lsp::semantic_tokens::compute(&text, &parse).finish(None);

    // Unconditionally cache the tokens
    state
        .semantic_tokens
        .insert(request.text_document.uri, semantic_tokens.clone());

    Ok(Some(semantic_tokens.into()))
}

pub fn handle_semantic_tokens_range(
    state: &GlobalState,
    request: lsp_types::SemanticTokensRangeParams,
) -> Result<Option<SemanticTokensRangeResult>> {
    // TODO(tumbar) We probably don't need to run a reparse here
    let (text, parse) = parse_text_document(&state, &request.text_document.uri)?;

    Ok(Some(SemanticTokensRangeResult::Tokens(
        lsp::semantic_tokens::compute(&text, &parse).finish(Some(request.range)),
    )))
}

pub fn handle_semantic_tokens_full_delta(
    state: &mut GlobalState,
    request: lsp_types::SemanticTokensDeltaParams,
) -> Result<Option<SemanticTokensFullDeltaResult>> {
    // TODO(tumbar) We probably don't need to run a reparse here
    let (text, parse) = parse_text_document(&state, &request.text_document.uri)?;

    let semantic_tokens = lsp::semantic_tokens::compute(&text, &parse).finish(None);

    let cached_tokens = state.semantic_tokens.remove(&request.text_document.uri);

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
            .insert(request.text_document.uri, semantic_tokens);
        return Ok(Some(delta.into()));
    }

    // Clone first to keep the lock short
    let semantic_tokens_clone = semantic_tokens.clone();
    state
        .semantic_tokens
        .insert(request.text_document.uri, semantic_tokens_clone);

    Ok(Some(semantic_tokens.into()))
}

pub fn handle_document_diagnostics(
    state: &mut GlobalState,
    request: lsp_types::DocumentDiagnosticParams,
) -> Result<DocumentDiagnosticReportResult> {
    Ok(DocumentDiagnosticReportResult::Report(
        lsp_types::DocumentDiagnosticReport::Full(lsp_types::RelatedFullDocumentDiagnosticReport {
            full_document_diagnostic_report: lsp_types::FullDocumentDiagnosticReport {
                items: state.diagnostics.get(&request.text_document.uri.as_str()),
                ..Default::default()
            },
            ..Default::default()
        }),
    ))
}

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
    state: &GlobalState,
    request: lsp_types::DocumentLinkParams,
) -> Result<Option<Vec<DocumentLink>>> {
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
    state: &GlobalState,
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

pub fn handle_goto_definition(
    state: &GlobalState,
    request: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    let offset = position_to_offset(
        state,
        &request.text_document_position_params.text_document.uri,
        &request.text_document_position_params.position,
    );

    if let Some((_, symbol)) = symbol_at_position(
        state,
        &request.text_document_position_params.text_document.uri,
        offset,
    ) {
        Ok(Some(GotoDefinitionResponse::Scalar(node_to_location(
            state,
            symbol.name().id(),
        ))))
    } else {
        Ok(None)
    }
}

pub fn handle_hover(state: &GlobalState, request: HoverParams) -> Result<Option<Hover>> {
    let offset = position_to_offset(
        state,
        &request.text_document_position_params.text_document.uri,
        &request.text_document_position_params.position,
    );

    let nodes = match nodes_at_offset(
        state,
        &request.text_document_position_params.text_document.uri,
        offset,
    ) {
        None => return Ok(None),
        Some(nodes) => nodes,
    };

    // Check if this node is a use/reference to definition
    if let Some((node, symbol)) = nodes.iter().find_map(|node| {
        if let Some(def) = state.analysis.use_def_map.get(&node.id()) {
            Some((*node, def))
        } else {
            None
        }
    }) {
        return Ok(Some(hover_for_symbol(state, node, symbol)));
    }

    // This is not a use/reference to another definition
    // From here on in we should only show hover information for definitions if we are hovering over
    // the definition's name
    if let Some(Node::Name(name)) = nodes.first() {
        // We are hovering over a name
        Ok(nodes
            .iter()
            .find_map(|node| hover_for_node(state, name, *node)))
    } else {
        Ok(None)
    }
}

pub fn handle_references(
    state: &GlobalState,
    request: ReferenceParams,
) -> Result<Option<Vec<Location>>> {
    let offset = position_to_offset(
        state,
        &request.text_document_position.text_document.uri,
        &request.text_document_position.position,
    );

    if let Some(nodes) = nodes_at_offset(
        state,
        &request.text_document_position.text_document.uri,
        offset,
    ) {
        let symbol = {
            // Check if this is a use to a symbol
            if let Some(symbol) = nodes.iter().find_map(|node| {
                if let Some(def) = state.analysis.use_def_map.get(&node.id()) {
                    return Some(def);
                } else {
                    None
                }
            }) {
                Some(symbol)
            // Check if this is a symbol definition
            } else if let Some(symbol) = nodes
                .iter()
                .find_map(|node| state.analysis.symbol_map.get(&node.id()))
                && let Some(Node::Name(_)) = nodes.first()
            {
                Some(symbol)
            } else {
                None
            }
        };

        if let Some(symbol) = symbol {
            // Look for all use-def resolutions that map to this symbol
            Ok(Some(
                state
                    .analysis
                    .use_def_map
                    .iter()
                    .filter_map(|(node, i_symbol)| {
                        if symbol.node() == i_symbol.node() {
                            Some(node_to_location(state, *node))
                        } else {
                            None
                        }
                    })
                    .collect(),
            ))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub fn handle_completion(
    state: &GlobalState,
    request: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let uri = request.text_document_position.text_document.uri;

    let text: String = state.vfs.read_sync(uri.as_str())?;
    let lines = state.vfs.get_lines(uri.as_str())?;

    let cursor_pos = match lines.offset(LineCol {
        line: request.text_document_position.position.line,
        col: request.text_document_position.position.character,
    }) {
        None => return Err(anyhow::anyhow!("position not in file bounds")),
        Some(p) => p,
    };

    let parse_kind = match state.files.get(uri.as_str()) {
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

    let token = parse.syntax_node().token_at_offset(cursor_pos);
    let expected_error_range = match token {
        TokenAtOffset::None => return Ok(None),
        TokenAtOffset::Single(tok) => tok.text_range(),
        TokenAtOffset::Between(l, r) => l.text_range().cover(r.text_range()),
    };

    // Check for parsing errors to extract the next expected token
    Ok(Some(CompletionResponse::Array(
        parse
            .errors()
            .iter()
            .filter_map(|e| {
                if expected_error_range.intersect(e.range()).is_some() {
                    e.expected().map(|k| (k, e.range()))
                } else {
                    None
                }
            })
            .filter_map(|(kind, range)| {
                match kind {
                    keyword if keyword.is_keyword() => {
                        // FIXME(tumbar) This seems brittle but works ok for now
                        let keyword_dbg = format!("{:?}", kind);
                        assert!(keyword_dbg.ends_with("_KW"));
                        let keyword_s = keyword_dbg[..keyword_dbg.len() - 3].to_ascii_lowercase();

                        Some(vec![CompletionItem {
                            label: keyword_s,
                            kind: Some(CompletionItemKind::KEYWORD),
                            ..Default::default()
                        }])
                    }
                    SyntaxKind::IDENT => {
                        let element = parse.syntax_node().covering_element(range);
                        let ancestors: Vec<SyntaxNode> = element.ancestors().collect();

                        if let Some(qual_ident) = ancestors.first()
                            && qual_ident.kind() == SyntaxKind::QUAL_IDENT
                            && let Some(parent_rule) = ancestors.get(1)
                        {
                            let ng = match parent_rule.kind() {
                                SyntaxKind::DEF_COMPONENT_INSTANCE => NameGroup::Component,
                                SyntaxKind::IMPLEMENTS_CLAUSE => NameGroup::PortInterface,
                                SyntaxKind::SPEC_CONNECTION_GRAPH_PATTERN => {
                                    NameGroup::PortInterfaceInstance
                                }
                                SyntaxKind::PATTERN_TARGET_MEMBER_LIST => {
                                    NameGroup::PortInterfaceInstance
                                }
                                SyntaxKind::SPEC_INTERFACE_IMPORT => NameGroup::PortInterface,
                                SyntaxKind::SPEC_INSTANCE => NameGroup::PortInterfaceInstance,
                                SyntaxKind::SPEC_LOC => return None,
                                SyntaxKind::SPEC_PORT_INSTANCE_GENERAL => NameGroup::Port,
                                SyntaxKind::SPEC_STATE_MACHINE_INSTANCE => NameGroup::StateMachine,
                                SyntaxKind::TRANSITION_EXPR => return None,
                                SyntaxKind::TYPE_NAME => NameGroup::Type,

                                _ => return None,
                            };

                            let tokens: Vec<SyntaxToken> = qual_ident
                                .children_with_tokens()
                                .filter_map(|s| s.as_token().map(|ss| ss.clone()))
                                .filter(|t| {
                                    t.kind() == SyntaxKind::IDENT
                                        && t.text_range().end() <= cursor_pos
                                })
                                .collect();

                            // If this is first token, we cannot rely on the AST since it's a completely
                            // invalid qualified identifier. The parent rule will therefore not exist in the AST
                            // The CST has this information of course so we can back out the current scope
                            // from the CST.
                            if tokens.is_empty() {
                                let current_scope: Vec<String> = qual_ident
                                    .ancestors()
                                    .filter_map(|ancestor| match ancestor.kind() {
                                        SyntaxKind::DEF_MODULE
                                        | SyntaxKind::DEF_COMPONENT
                                        | SyntaxKind::DEF_ENUM
                                        | SyntaxKind::DEF_STATE_MACHINE => {
                                            let name =
                                                ancestor.first_child_or_token_by_kind(&|k| {
                                                    k == SyntaxKind::NAME
                                                });
                                            name.map(|n| text[n.text_range()].to_string())
                                        }
                                        _ => None,
                                    })
                                    .collect();

                                eprintln!("scope: {:?}", current_scope);

                                // Merge all symbols going up from each scope
                                let items: Vec<Vec<CompletionItem>> = current_scope
                                    .iter()
                                    .rev()
                                    .fold(
                                        (vec![], Some(&state.analysis.global_scope)),
                                        |(mut out, scope), scope_name| {
                                            if let Some(scope) = scope {
                                                out.push(
                                                    scope
                                                        .get_group(ng)
                                                        .iter()
                                                        .map(|(_, s)| {
                                                            symbol_to_completion_item(state, s)
                                                        })
                                                        .collect(),
                                                );

                                                (
                                                    out,
                                                    scope
                                                        .get(ng, scope_name)
                                                        .map(|symbol| {
                                                            state
                                                                .analysis
                                                                .symbol_scope_map
                                                                .get(&symbol)
                                                        })
                                                        .flatten(),
                                                )
                                            } else {
                                                (out, None)
                                            }
                                        },
                                    )
                                    .0;

                                // The closest symbols should appear first
                                // Flip the completion items and flatten everything
                                Some(items.into_iter().rev().flatten().collect())
                            } else {
                                // Get the final token before the cursor to look it up in the AST
                                let last_token_pos =
                                    tokens.last().unwrap().text_range().start().into();

                                // Look up the symbol before the cursor
                                symbol_at_position(state, &uri, last_token_pos)
                                    .map(|(_, symbol)| state.analysis.symbol_scope_map.get(&symbol))
                                    .flatten()
                                    .map(|scope| {
                                        // Get all symbols under this symbol's scope in the proper
                                        // name group
                                        scope
                                            .get_group(ng)
                                            .iter()
                                            .map(|(_, child_symbol)| {
                                                symbol_to_completion_item(state, child_symbol)
                                            })
                                            .collect()
                                    })
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .flatten()
            .collect(),
    )))
}
