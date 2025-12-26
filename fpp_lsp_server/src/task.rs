use crate::global_state::{GlobalState, Task};
use crate::progress::Progress;
use crate::vfs::Vfs;
use fpp_analysis::Analysis;
use rustc_hash::FxHashMap;
use std::ops::Deref;
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

                let progress_c = progress.clone();
                self.asts = fpp_core::run(&mut self.context, || {
                    progress_c.report("Parsing workspace");
                    let asts = on_parse_workspace(&mut analysis, &self.vfs, progress, files);

                    progress_c.report("Analyzing workspace");
                    let _ = fpp_analysis::check_semantics(
                        &mut analysis,
                        self.asts.values().map(|v| v.deref()).collect(),
                    );

                    asts
                });

                self.analysis = Arc::new(analysis);
            }
        }
    }
}
