use crate::diagnostics::LspDiagnosticsEmitter;
use crate::global_state::{GlobalState, Task};
use crate::lsp;
use crate::lsp::utils::semantic_token_delta;
use anyhow::Result;
use fpp_analysis::semantics::{Symbol, SymbolInterface};
use fpp_ast::{AstNode, MoveWalkable, Name, Node, Visitor};
use fpp_core::{BytePos, CompilerContext, LineCol, LineIndex, SourceFile};
use fpp_lsp_parser::{SyntaxKind, SyntaxNode, SyntaxToken, TextRange, VisitorResult};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticReportResult, DocumentLink, GotoDefinitionParams, GotoDefinitionResponse,
    Hover, HoverContents, HoverParams, Location, MarkupContent, MarkupKind, Position, Range,
    ReferenceParams, SemanticTokensFullDeltaResult, SemanticTokensRangeResult,
    SemanticTokensResult, Uri,
};
use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;
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
    state: &mut GlobalState,
    request: lsp_types::SemanticTokensParams,
) -> Result<Option<SemanticTokensResult>> {
    tracing::info!(uri = %request.text_document.uri.as_str(), "SemanticTokens");

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
    tracing::info!(uri = %request.text_document.uri.as_str(), "SemanticTokens");

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
    tracing::info!(uri = %request.text_document.uri.as_str(), "SemanticTokens");

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
    tracing::info!(uri = %request.text_document.uri.as_str(), "document diagnostics");

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

struct FindPositionVisitor<'a> {
    source_file: SourceFile,
    looking_for: BytePos,
    context: &'a CompilerContext<LspDiagnosticsEmitter>,
}

impl<'ast> Visitor<'ast> for FindPositionVisitor<'ast> {
    type Break = ();
    type State = Vec<Node<'ast>>;

    /// The default node visiting before.
    /// By default, this will just continue without visiting the children of `node`
    fn super_visit(&self, a: &mut Self::State, node: Node<'ast>) -> ControlFlow<Self::Break> {
        let span = self
            .context
            .span_get(&self.context.node_get_span(&node.id()));

        let src_file: SourceFile = span.file.upgrade().unwrap().as_ref().into();

        if src_file == self.source_file {
            // Check if this node spans the range we are looking for
            if span.start <= self.looking_for && span.start + span.length >= self.looking_for {
                // Depth first
                let out = node.walk(a, self);

                a.push(node);
                out
            } else {
                // This node does not span the range
                // We don't need to walk it since it's children won't span it either
                ControlFlow::Continue(())
            }
        } else {
            // The files don't match
            // We could be looking for something inside an include
            // Keep recursing
            match node {
                Node::DefAction(_) => node.walk(a, self),
                Node::DefComponent(_) => node.walk(a, self),
                Node::DefModule(_) => node.walk(a, self),
                Node::DefState(_) => node.walk(a, self),
                Node::DefStateMachine(_) => node.walk(a, self),
                Node::DefTopology(_) => node.walk(a, self),
                Node::SpecInclude(_) => node.walk(a, self),
                _ => ControlFlow::Continue(()),
            }
        }
    }
}

fn nodes_at_position<'a>(
    state: &'a GlobalState,
    document: &Uri,
    position: &Position,
) -> Option<Vec<Node<'a>>> {
    let files = match state.files.get(document.as_str()) {
        None => return None,
        Some(files) => files,
    };

    Some(
        files
            .into_iter()
            .flat_map(|file| {
                let cache = state.cache.get(&state.parent_file(*file)).unwrap();
                let pos = state.context.file_get(&file).offset_of(LineCol {
                    line: position.line,
                    col: position.character,
                });

                let visitor = FindPositionVisitor {
                    source_file: *file,
                    looking_for: pos,
                    context: &state.context,
                };

                let mut out = vec![];
                let _ = visitor.visit_trans_unit(&mut out, &cache.ast);
                out
            })
            .collect(),
    )
}

fn symbol_at_position<'a>(
    state: &'a GlobalState,
    document: &Uri,
    position: &Position,
) -> Option<(Node<'a>, Symbol)> {
    let nodes = match nodes_at_position(state, document, position) {
        None => return None,
        Some(nodes) => nodes,
    };

    nodes.iter().find_map(|node| {
        if let Some(def) = state.analysis.use_def_map.get(&node.id()) {
            return Some((*node, def.clone()));
        } else {
            None
        }
    })
}

fn node_to_range(state: &GlobalState, node: fpp_core::Node) -> Range {
    let span = state.context.span_get(&state.context.node_get_span(&node));
    let file = span.file.upgrade().unwrap();

    let start = file.position(span.start);
    let end = file.position(span.start + span.length);

    Range {
        start: Position {
            line: start.line(),
            character: start.column(),
        },
        end: Position {
            line: end.line(),
            character: end.column(),
        },
    }
}

fn node_to_location(state: &GlobalState, node: fpp_core::Node) -> Location {
    let span = state.context.span_get(&state.context.node_get_span(&node));
    let file = span.file.upgrade().unwrap();

    let start = file.position(span.start);
    let end = file.position(span.start + span.length);

    Location {
        uri: Uri::from_str(&file.uri).unwrap(),
        range: Range {
            start: Position {
                line: start.line(),
                character: start.column(),
            },
            end: Position {
                line: end.line(),
                character: end.column(),
            },
        },
    }
}

pub fn handle_goto_definition(
    state: &GlobalState,
    request: GotoDefinitionParams,
) -> Result<Option<GotoDefinitionResponse>> {
    if let Some((_, symbol)) = symbol_at_position(
        state,
        &request.text_document_position_params.text_document.uri,
        &request.text_document_position_params.position,
    ) {
        Ok(Some(GotoDefinitionResponse::Scalar(node_to_location(
            state,
            symbol.name().id(),
        ))))
    } else {
        Ok(None)
    }
}

fn hover_for_symbol(state: &GlobalState, hover_node: Node, symbol: &Symbol) -> Hover {
    let node_data = state.context.node_get(&symbol.node());

    let symbol_kind = match symbol {
        Symbol::AbsType(_) => "Abstract Type",
        Symbol::AliasType(_) => "Type Alias",
        Symbol::Array(_) => "Array",
        Symbol::Component(_) => "Component",
        Symbol::ComponentInstance(_) => "Component Instance",
        Symbol::Constant(_) => "Constant",
        Symbol::Enum(_) => "Enum",
        Symbol::EnumConstant(_) => "Enum Constant",
        Symbol::Interface(_) => "Interface",
        Symbol::Module(_) => "Module",
        Symbol::Port(_) => "Port",
        Symbol::StateMachine(_) => "State Machine",
        Symbol::Struct(_) => "Struct",
        Symbol::Topology(_) => "Topology",
    };

    // Convert the name into a fully qualified name by following the parent symbols
    let mut qualified_name = vec![symbol];
    let mut current = symbol;
    loop {
        match state.analysis.parent_symbol_map.get(current) {
            None => break,
            Some(parent) => {
                qualified_name.push(parent);
                current = parent;
            }
        }
    }

    qualified_name.reverse();
    let qualified_idents: Vec<&str> = qualified_name
        .into_iter()
        .map(|n| n.name().data.as_str())
        .collect();
    let qual_ident = qualified_idents.join(".");

    let symbol_kind_line = format!("({symbol_kind}) {qual_ident}");

    let markdown_lines: Vec<String> = node_data
        .pre_annotation
        .clone()
        .into_iter()
        .chain(vec![
            "```typescript".to_string(),
            symbol_kind_line,
            "```".to_string(),
        ])
        .chain(node_data.post_annotation.clone().into_iter())
        .collect();

    Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown_lines.join("\n"),
        }),
        range: Some(node_to_range(state, hover_node.id())),
    }
}

fn hover_for_node(state: &GlobalState, hover_node: &Name, def_node: Node) -> Option<Hover> {
    let symbol_kind = match def_node {
        Node::DefAbsType(_) => "Abstract Type",
        Node::DefAliasType(_) => "Type Alias",
        Node::DefArray(_) => "Array",
        Node::DefComponent(_) => "Component",
        Node::DefComponentInstance(_) => "Component Instance",
        Node::DefConstant(_) => "Constant",
        Node::DefEnum(_) => "Enum",
        Node::DefEnumConstant(_) => "Enum Constant",
        Node::DefInterface(_) => "Interface",
        Node::DefModule(_) => "Module",
        Node::DefPort(_) => "Port",
        Node::DefStateMachine(_) => "State Machine",
        Node::DefStruct(_) => "Struct",
        Node::DefTopology(_) => "Topology",
        Node::DefChoice(_) => "Choice",
        Node::DefGuard(_) => "Guard",
        Node::DefSignal(_) => "Signal",
        Node::DefState(_) => "State",
        Node::SpecCommand(_) => "Command",
        Node::SpecConnectionGraph(_) => "Connection Graph",
        Node::SpecContainer(_) => "Container",
        Node::SpecEvent(_) => "Event",
        Node::SpecGeneralPortInstance(_) => "Port Instance",
        Node::SpecParam(_) => "Parameter",
        Node::SpecRecord(_) => "Record",
        Node::SpecSpecialPortInstance(_) => "Special Port Instance",
        Node::SpecStateMachineInstance(_) => "State Machine Instance",
        Node::SpecTlmChannel(_) => "Telemetry Channel",
        Node::SpecTlmPacket(_) => "Telemetry Packet",
        Node::SpecTlmPacketSet(_) => "Telemetry Packet Set",
        Node::SpecTopPort(_) => "Topology Port",
        _ => return None,
    };

    let node_data = state.context.node_get(&def_node.id());

    // Convert the name into a fully qualified name by following the parent symbols
    let qual_ident = {
        if let Some(symbol) = state.analysis.symbol_map.get(&def_node.id()) {
            let mut qualified_name = vec![symbol];
            let mut current = symbol;
            loop {
                match state.analysis.parent_symbol_map.get(current) {
                    None => break,
                    Some(parent) => {
                        qualified_name.push(parent);
                        current = parent;
                    }
                }
            }

            qualified_name.reverse();
            let qualified_idents: Vec<&str> = qualified_name
                .into_iter()
                .map(|n| n.name().data.as_str())
                .collect();

            qualified_idents.join(".")
        } else {
            hover_node.data.clone()
        }
    };

    let symbol_kind_line = format!("({symbol_kind}) {qual_ident}");

    let markdown_lines: Vec<String> = node_data
        .pre_annotation
        .clone()
        .into_iter()
        .chain(vec![
            "```typescript".to_string(),
            symbol_kind_line,
            "```".to_string(),
        ])
        .chain(node_data.post_annotation.clone().into_iter())
        .collect();

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown_lines.join("\n"),
        }),
        range: Some(node_to_range(state, hover_node.id())),
    })
}

pub fn handle_hover(state: &GlobalState, request: HoverParams) -> Result<Option<Hover>> {
    let nodes = match nodes_at_position(
        state,
        &request.text_document_position_params.text_document.uri,
        &request.text_document_position_params.position,
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
    if let Some(nodes) = nodes_at_position(
        state,
        &request.text_document_position.text_document.uri,
        &request.text_document_position.position,
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
