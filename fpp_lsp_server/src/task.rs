use crate::diagnostics::DiagnosticType;
use crate::global_state::{GlobalState, Task};
use crate::progress::Progress;
use crate::vfs::Vfs;
use fpp_analysis::Analysis;
use lsp_types::Uri;
use rustc_hash::FxHashMap;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

fn on_parse_workspace(
    a: &mut Analysis,
    vfs: &Vfs,
    progress: Progress,
    files: Vec<(String, String)>,
) -> FxHashMap<String, Arc<fpp_ast::TransUnit>> {
    files
        .into_iter()
        .map(|(filename, file)| {
            progress.report(&format!("Parsing {}", filename));
            let mut ast = fpp_ast::TransUnit(fpp_parser::parse(
                fpp_core::SourceFile::new(&filename, file),
                |p| p.module_members(),
                None,
            ));

            let _ = fpp_analysis::resolve_includes(a, vfs, &mut ast);

            (filename.clone(), Arc::new(ast))
        })
        .collect()
}

impl GlobalState {
    pub(crate) fn on_task(&mut self, task: Task) {
        match task {
            Task::IndexWorkspace((progress, files)) => {
                let mut analysis = Analysis::new();
                self.diagnostics.lock().unwrap().clear();

                let progress_c = progress.clone();
                self.asts = fpp_core::run(&mut self.context.lock().unwrap(), || {
                    progress_c.report("Parsing workspace");
                    let asts = on_parse_workspace(&mut analysis, &self.vfs, progress, files);

                    asts
                });

                self.analysis = Arc::new(analysis);
                self.task(Task::Analysis(()));
            }
            Task::Response(response) => self.respond(response),
            Task::Notification(notification) => {
                self.send(lsp_server::Message::Notification(notification))
            }
            Task::Parse(uri) => {
                // Clear all diagnostics for this file
                self.diagnostics.lock().unwrap().clear_for(&uri);
                self.diagnostics
                    .lock()
                    .unwrap()
                    .set_mode(DiagnosticType::Syntax);

                if let Some((ast, a)) = fpp_core::run(&mut self.context.lock().unwrap(), || {
                    if let Some(parent) = fpp_core::SourceFile::get(uri.as_str())
                        .map(|file| file.get_parent())
                        .flatten()
                    {
                        // This file was included from another file
                        // We should reprocess the above file since thats the true translation unit that changed
                        tracing::info!(uri = %uri.as_str(), parent_uri = %parent.uri(), "parent file found");
                        self.task(Task::Parse(Uri::from_str(&parent.uri()).unwrap()));
                        None
                    } else {
                        // Read the file from VFS and produce the initial AST
                        let content = self.vfs.read_sync(uri.as_str()).unwrap();
                        let mut ast = fpp_ast::TransUnit(fpp_parser::parse(
                            fpp_core::SourceFile::new(uri.as_str(), content),
                            |p: &mut fpp_parser::Parser| p.module_members(),
                            None,
                        ));

                        let mut a = Analysis::new();
                        a.include_context_map = self.analysis.include_context_map.clone();
                        let _ = fpp_analysis::resolve_includes(&mut a, &self.vfs, &mut ast);

                        Some((ast, a))
                    }
                }) {
                    self.asts.insert(uri.as_str().to_string(), Arc::new(ast));
                    self.analysis = Arc::new(a);
                    self.task(Task::Analysis(()))
                }
            }
            Task::Analysis(_) => {
                let mut analysis = Analysis::new();
                analysis.include_context_map = self.analysis.include_context_map.clone();

                // Clear all analysis diagnostics
                self.diagnostics
                    .lock()
                    .unwrap()
                    .set_mode(DiagnosticType::Analysis);
                self.diagnostics.lock().unwrap().clear_all_analysis();

                fpp_core::run(&mut self.context.lock().unwrap(), || {
                    let _ = fpp_analysis::check_semantics(
                        &mut analysis,
                        self.asts.values().map(|v| v.deref()).collect(),
                    );
                });

                self.analysis = Arc::new(analysis)
            }
        }
    }
}
