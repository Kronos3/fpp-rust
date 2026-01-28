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
                let key: String = uri.path().to_string();
                let content = self.vfs.read_sync(&key.clone().into()).unwrap();

                if let Some(ast) = fpp_core::run(&mut self.context.lock().unwrap(), || {
                    let file: fpp_core::SourceFile =
                        fpp_core::SourceFile::new(uri.as_str(), content);

                    if let Some((parent, _)) = self.analysis.parent_file_map.get(&file) {
                        self.task(Task::Parse(Uri::from_str(&parent.uri()).unwrap()));
                        None
                    } else {
                        Some(fpp_parser::parse(
                            file,
                            |p: &mut fpp_parser::Parser| p.module_members(),
                            None,
                        ))
                    }
                }) {
                    self.asts.insert(key, Arc::new(fpp_ast::TransUnit(ast)));
                    self.task(Task::Analysis(()))
                }
            }
            Task::Analysis(_) => {
                let mut analysis = Analysis::new();
                analysis.parent_file_map = self.analysis.parent_file_map.clone();

                self.diagnostics.lock().unwrap().clear();

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
