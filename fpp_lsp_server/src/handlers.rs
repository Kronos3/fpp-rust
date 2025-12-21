use crate::context::LspDiagnosticsEmitter;
use crate::global_state::GlobalState;
use anyhow::Result;
use fpp_analysis::Analysis;
use fpp_core::CompilerContext;
use notify::{RecursiveMode, Watcher};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

pub fn handle_workspace_reload(state: &mut GlobalState, _: ()) -> Result<()> {
    // Wipe all the accumulated state
    state.diagnostics = Rc::new(RefCell::new(LspDiagnosticsEmitter::new()));
    state.asts = FxHashMap::default();
    state.analysis = Arc::new(Analysis::new());

    let mut ctx = CompilerContext::new(state.diagnostics.clone());

    let mut watcher = notify::recommended_watcher(|res| {
        match res {
            Ok(event) => {
                event;
            }
            Err(_) => {}
        }
    })?;

    watcher.watch(Path::new(&state.workspace_locs), RecursiveMode::NonRecursive)?;

    state.context = ctx;

    Ok(())
}
