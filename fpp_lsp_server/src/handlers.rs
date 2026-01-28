use crate::global_state::Task::IndexWorkspace;
use crate::global_state::{GlobalState, GlobalStateSnapshot, Task};
use crate::lsp::utils::semantic_token_delta;
use crate::{lsp, vfs};
use anyhow::Result;
use fpp_analysis::Analysis;
use fpp_ast::ModuleMember;
use fpp_core::{CompilerContext, SourceFile};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DocumentDiagnosticReportResult, SemanticTokensFullDeltaResult, SemanticTokensRangeResult,
    SemanticTokensResult,
};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

async fn read_workspace(workspace_locs: String, vfs: vfs::Vfs) -> Result<Vec<(String, String)>> {
    let mut vfs_c_1 = vfs.clone();
    let workspace_locs_1 = workspace_locs.clone();

    let mut locs: Vec<String> = fpp_parser::parse(
        SourceFile::new(
            &workspace_locs,
            tokio::task::spawn_blocking(move || vfs_c_1.read(&workspace_locs_1.clone().into()))
                .await??,
        ),
        |p| p.module_members(),
        None,
    )
    .into_iter()
    .filter_map(|loc| match loc {
        ModuleMember::SpecLoc(loc) => Some(loc.file.data),
        _ => None,
    })
    .collect();

    locs.dedup();

    // Scan the locs file synchronously
    let locs_path = PathBuf::from_str(&workspace_locs)?;
    let locs_dir = locs_path.parent().unwrap();

    let mut loc_files_futures = tokio::task::JoinSet::new();
    for file in locs {
        let mut path: PathBuf = locs_dir.into();
        path.push(file);
        let mut vfs_c = vfs.clone();
        loc_files_futures.spawn(async move {
            (
                path.to_string_lossy().to_string(),
                tokio::task::spawn_blocking(move || vfs_c.read(&path)).await,
            )
        });
    }

    Ok(loc_files_futures
        .join_all()
        .await
        .into_iter()
        .filter_map(|(path, text)| match text {
            Ok(Ok(text)) => Some((path, text)),
            Ok(Err(err)) => {
                tracing::error!(err = %err, "failed to read file {}", path);
                None
            }
            Err(err) => {
                tracing::error!(err = %err, "failed to read file {}", path);
                None
            }
        })
        .collect())
}

pub fn handle_workspace_reload(state: &mut GlobalState, _: ()) -> Result<()> {
    // Wipe all the accumulated state
    state.diagnostics.lock().unwrap().clear();
    state.asts = FxHashMap::default();
    state.analysis = Arc::new(Analysis::new());
    state.context = Arc::new(Mutex::new(CompilerContext::new(state.diagnostics.clone())));
    state.vfs.clear();

    let watchers = vec![lsp_types::FileSystemWatcher {
        glob_pattern: lsp_types::GlobPattern::String("**/*.rs".to_string()),
        kind: None,
    }];

    let registration_options = lsp_types::DidChangeWatchedFilesRegistrationOptions { watchers };

    let registration = lsp_types::Registration {
        id: "workspace/didChangeWatchedFiles".to_owned(),
        method: "workspace/didChangeWatchedFiles".to_owned(),
        register_options: Some(serde_json::to_value(registration_options)?),
    };
    state.send_request::<lsp_types::request::RegisterCapability>(
        lsp_types::RegistrationParams {
            registrations: vec![registration],
        },
        |_, _| (),
    );

    // TODO(tumbar) How do I hook the cancellation token into the event loop?
    //      One way might be to use a Go-style context
    let (progress, _) = state.new_progress("Indexing FPP Project", 0);

    progress.report("Reading workspace from filesystem");

    let read_future = read_workspace(state.workspace_locs.clone(), state.vfs.clone());
    let sender = state.get_sender();

    // Perform read/indexing asynchronously
    tokio::spawn(async move {
        match read_future.await {
            // Successfully read from the filesystem
            // Send the results back to the main_loop for processing
            Ok(files) => sender.send(IndexWorkspace((progress.with_total(files.len()), files))),
            Err(e) => {
                progress.finish(None);
                sender.send_notification::<lsp_types::notification::ShowMessage>(
                    lsp_types::ShowMessageParams {
                        typ: lsp_types::MessageType::ERROR,
                        message: format!("failed to read workspace: {:#?}", e),
                    },
                );
            }
        }
    });

    Ok(())
}

pub fn handle_did_open_text_document(
    state: &mut GlobalState,
    not: DidOpenTextDocumentParams,
) -> Result<()> {
    tracing::info!(uri = %not.text_document.uri.as_str(), "DidOpenTextDocument");
    let uri = not.text_document.uri.clone();
    state.vfs.did_open(not);
    state.task(Task::Parse(uri));
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
    state.task(Task::Parse(uri));
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
    state.task(Task::Parse(uri));

    Ok(())
}

pub fn handle_exit(state: &mut GlobalState, _: ()) -> Result<()> {
    state.shutdown_requested = true;
    Ok(())
}

fn parse_text_document(
    state: &GlobalStateSnapshot,
    uri: &lsp_types::Uri,
) -> Result<(String, fpp_lsp_parser::Parse)> {
    let path_s = &uri.path().to_string();
    let text = state.vfs.read_sync(&path_s.into())?;

    let parse_kind = fpp_core::run(&mut state.context.lock().unwrap(), || {
        if let Some(source_file) = SourceFile::get(&path_s) {
            match state.analysis.parent_file_map.get(&source_file) {
                Some((_, kind)) => kind.clone(),
                None => fpp_parser::IncludeParentKind::Module,
            }
        } else {
            fpp_parser::IncludeParentKind::Module
        }
    });

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
                    .diagnostics
                    .get(&request.text_document.uri)
                    .map_or(vec![], |d| d.clone()),
                ..Default::default()
            },
            ..Default::default()
        }),
    ))
}
