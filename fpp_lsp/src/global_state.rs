use std::sync::mpsc::Sender;
use rustc_hash::FxHashMap;
use fpp_analysis::Analysis;

pub struct GlobalState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,

    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,

    asts: FxHashMap<String, fpp_ast::TransUnit>,

    analysis: Analysis,
}

impl GlobalState {
    pub(crate) fn respond(&self, response: lsp_server::Response) {
        
    }
}

pub struct GlobalStateSnapshot {
    
}
