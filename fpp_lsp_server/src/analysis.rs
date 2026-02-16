use crate::diagnostics::DiagnosticType;
use crate::global_state::{GlobalState, TranslationUnitCache, Workspace};
use fpp_analysis::Analysis;
use fpp_ast::MutVisitor;
use fpp_core::{CompilerContext, Diagnostic, GarbageCollectionSet, Level, SourceFile};
use lsp_types::Uri;
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};

use fpp_parser::ResolveIncludes;
use ignore::WalkBuilder;
use std::ffi::OsStr;
use std::str::FromStr;
use url::Url;

#[derive(Debug)]
pub enum Task {
    Response(lsp_server::Response),
    Notification(lsp_server::Notification),
    // Retry(lsp_server::Request),
    ReloadWorkspace,
    LoadLocsFile(Uri),
    LoadFullWorkspace(Uri),
    /// The VFS indicated a file changed, we need to reprocess it in the analysis
    /// This may trigger 0+ 'Reprocess' tasks
    Update(Uri),
    /// This source file has changed and must be re-analyzed
    /// Read the contents from the VFS, parse it, resolve includes, and schedule an analysis
    Reprocess(SourceFile),
    /// One or more translation units have changed and semantic analysis is now out of date
    /// Rerun semantic analysis incorporating all the currently cached translation units
    Analysis,
}

impl GlobalState {
    fn new_translation_unit_cache(&self, uri: &str) -> anyhow::Result<TranslationUnitCache> {
        tracing::info_span!("parsing file {}", uri = uri);

        GarbageCollectionSet::start();

        // Clear all diagnostics for this file
        self.diagnostics.lock().unwrap().clear_for(uri);
        self.diagnostics
            .lock()
            .unwrap()
            .set_mode(DiagnosticType::Syntax);

        // Read the file from VFS and produce the initial AST
        let content = self.vfs.read(uri)?;
        let file = SourceFile::new(uri, content);

        let mut ast = fpp_ast::TransUnit(fpp_parser::parse(
            file,
            |p: &mut fpp_parser::Parser| p.module_members(),
            None,
        ));

        let mut include_context_map = Default::default();
        let _ =
            ResolveIncludes::new(&self.vfs).visit_trans_unit(&mut include_context_map, &mut ast);

        Ok(TranslationUnitCache {
            uri: uri.to_string(),
            file,
            ast,
            include_context_map,
            gc: GarbageCollectionSet::finish(),
        })
    }

    pub(crate) fn on_task(&mut self, task: Task) {
        match task {
            Task::ReloadWorkspace => {
                tracing::info!(context = "task", "reloading workspace");

                match self.workspace.clone() {
                    Workspace::None => {}
                    Workspace::LocsFile(uri) => self.task(Task::LoadLocsFile(uri)),
                    Workspace::FullWorkspace(uri) => self.task(Task::LoadFullWorkspace(uri)),
                }
            }
            Task::LoadLocsFile(locs_uri) => {
                tracing::info!(context = "load locs file", uri = %locs_uri.as_str(), "reloading locs file");

                // Read the locs file to build a list of files to add to the analysis
                let locs_content = match self.vfs.read(locs_uri.as_str()) {
                    Ok(locs_content) => locs_content,
                    Err(err) => {
                        tracing::warn!(context = "task", uri = %locs_uri.as_str(), err = ?err, "failed to read locs file during workspace reload");
                        return;
                    }
                };

                // Refresh the context and all caches
                self.context = Arc::new(Mutex::new(CompilerContext::new(self.diagnostics.clone())));
                self.diagnostics.lock().unwrap().clear();
                self.cache = Default::default();
                self.files = Default::default();
                self.analysis = Arc::new(Analysis::new());
                self.workspace = Workspace::LocsFile(locs_uri.clone());

                let vfs = self.vfs.clone();

                let cache = fpp_core::run(&mut self.context.lock().unwrap(), || {
                    let mut locs: Vec<fpp_ast::SpecLoc> = fpp_parser::parse(
                        SourceFile::new(&locs_uri.as_str(), locs_content),
                        |p| p.module_members(),
                        None,
                    )
                    .into_iter()
                    .filter_map(|loc| match loc {
                        fpp_ast::ModuleMember::SpecLoc(loc) => Some(loc),
                        _ => None,
                    })
                    .collect();

                    locs.dedup();

                    let mut cache = FxHashMap::default();
                    for loc in locs {
                        match vfs.resolve_uri_relative_path(locs_uri.as_str(), &loc.file.data) {
                            Ok(file_uri) => match self.new_translation_unit_cache(&file_uri) {
                                Ok(tu_cache) => {
                                    cache.insert(tu_cache.file, Arc::new(tu_cache));
                                }
                                Err(err) => Diagnostic::new(
                                    loc,
                                    Level::Error,
                                    "failed to process location specifier",
                                )
                                .annotation(err.to_string())
                                .emit(),
                            },
                            Err(err) => Diagnostic::new(
                                loc,
                                Level::Error,
                                "failed to resolve location specifier",
                            )
                            .annotation(err.to_string())
                            .emit(),
                        }
                    }

                    cache
                });

                tracing::info!(context = "task", "finished reparsing workspace");
                self.cache = cache;
                self.task(Task::Analysis);
            }
            Task::LoadFullWorkspace(workspace_uri) => {
                tracing::info!(context = "load full workspace", uri = %workspace_uri.as_str(), "scanning workspace for FPP files");

                // WalkBuilder automatically respects .gitignore rules by default
                // Scan for all the .fpp files in the workspace
                let mut files = vec![];
                for result in WalkBuilder::new(workspace_uri.path().as_str()).build() {
                    match result {
                        Ok(entry) => {
                            let path = entry.path();

                            // Check if the entry is a file and matches the extension filter
                            if path.is_file() {
                                if path
                                    .extension()
                                    .map_or(false, |ext| OsStr::new("fpp") == ext)
                                {
                                    match Url::from_file_path(path) {
                                        Ok(url) => match Uri::from_str(url.as_str()) {
                                            Ok(uri) => {
                                                files.push(uri);
                                            }
                                            Err(err) => {
                                                tracing::warn!(context = "load full workspace", err = ?err, "failed to convert Url to Uri");
                                            }
                                        },
                                        Err(err) => {
                                            tracing::warn!(context = "load full workspace", err = ?err, "convert file path into url");
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            tracing::warn!(context = "load full workspace", err = ?err, "failed to walk directory");
                        }
                    }
                }

                tracing::info!(context = "load full workspace", uri = %workspace_uri.as_str(), "found {} FPP files", files.len());

                // Refresh the context and all caches
                self.context = Arc::new(Mutex::new(CompilerContext::new(self.diagnostics.clone())));
                self.diagnostics.lock().unwrap().clear();
                self.cache = Default::default();
                self.files = Default::default();
                self.analysis = Arc::new(Analysis::new());
                self.workspace = Workspace::FullWorkspace(workspace_uri.clone());

                let cache = fpp_core::run(&mut self.context.lock().unwrap(), || {
                    let mut cache = FxHashMap::default();
                    for file in files {
                        match self.vfs.read(file.as_str()) {
                            Ok(file_uri) => match self.new_translation_unit_cache(&file_uri) {
                                Ok(tu_cache) => {
                                    cache.insert(tu_cache.file, Arc::new(tu_cache));
                                }
                                Err(err) => {
                                    tracing::error!(context = "load full workspace", uri = %file.as_str(), err = ?err, "failed to process file in workspace");
                                }
                            },
                            Err(err) => {
                                tracing::error!(context = "load full workspace", uri = %file.as_str(), err = ?err, "failed to read file in workspace");
                            }
                        }
                    }

                    cache
                });

                tracing::info!(context = "task", "finished reparsing workspace");
                self.cache = cache;
                self.task(Task::Analysis);
            }
            Task::Response(response) => self.respond(response),
            Task::Notification(notification) => {
                self.send(lsp_server::Message::Notification(notification))
            }
            Task::Update(uri) => {
                tracing::info!(uri = %uri.as_str(), context = "task", "parsing");

                // Check if this file is the locs file
                if self.workspace == Workspace::LocsFile(uri.clone()) {
                    tracing::info!(uri = %uri.as_str(), context = "task", "workspace locs has updated, refreshing workspace");
                    self.task(Task::ReloadWorkspace);
                    return;
                }

                // Check if this file is currently part of the compiler context
                match self.files.get(uri.as_str()) {
                    None => {
                        tracing::debug!(uri = %uri.as_str(), context = "task", "not part of the compiler context, not adding to analysis")
                    }
                    Some(files) => {
                        // This file is added in one or more ways to the compiler analysis
                        for file in files {
                            self.task(Task::Reprocess(*file))
                        }
                    }
                }
            }
            Task::Reprocess(source_file) => {
                let old_cache = self.cache.remove(&source_file).unwrap();
                let new_cache = match fpp_core::run(&mut self.context.lock().unwrap(), || {
                    assert!(source_file.parent().is_none());
                    let uri = source_file.uri();

                    // Clean up the old file cache in the compiler context
                    old_cache.gc.cleanup();

                    (
                        self.new_translation_unit_cache(uri.as_str()),
                        uri.as_str().to_string(),
                    )
                }) {
                    (Ok(tu_cache), _) => tu_cache,
                    (Err(err), uri) => {
                        tracing::error!(context = "reprocess file", uri = %uri, err = ?err, "failed to reprocess file");
                        return;
                    }
                };

                self.cache.insert(new_cache.file, Arc::new(new_cache));
                self.task(Task::Analysis);
            }
            Task::Analysis => {
                tracing::info!(context = "task", "analysis");

                // Clear all analysis diagnostics
                self.diagnostics
                    .lock()
                    .unwrap()
                    .set_mode(DiagnosticType::Analysis);
                self.diagnostics.lock().unwrap().clear_all_analysis();

                let analysis = fpp_core::run(&mut self.context.lock().unwrap(), || {
                    let mut files = FxHashMap::default();
                    let mut analysis = Analysis::new();

                    for (file, cache) in &self.cache {

                        for (included, include_context) in &cache.include_context_map {
                            analysis.include_context_map.insert(*included, *include_context);
                            let included_uri = included.uri();

                            match files.get_mut(&included_uri) {
                                None => {
                                    files.insert(included_uri, vec![*included]);
                                }
                                Some(v) => {
                                    v.push(*included);
                                }
                            }
                        }

                        let file_uri = file.uri();
                        match files.get_mut(&file_uri) {
                            None => {
                                files.insert(file_uri, vec![*file]);
                            }
                            Some(v) => {
                                v.push(*file);
                            }
                        }
                    }

                    self.files = Arc::new(files);

                    let _ = fpp_analysis::check_semantics(
                        &mut analysis,
                        self.cache.values().map(|v| &v.ast).collect(),
                    );

                    analysis
                });

                self.analysis = Arc::new(analysis)
            }
        }
    }
}
