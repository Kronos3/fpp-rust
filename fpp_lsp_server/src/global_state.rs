use crate::context::LspDiagnosticsEmitter;
use crate::{lsp_ext};
use fpp_analysis::Analysis;
use fpp_core::{CompilerContext, FileReader};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc};
use std::time::Instant;
use threadpool::ThreadPool;

pub(crate) type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

pub struct GlobalState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,

    pub(crate) task_pool: ThreadPool,
    pub(crate) shutdown_requested: bool,

    pub(crate) reader: Box<dyn FileReader>,
    pub(crate) workspace_locs: String,

    pub(crate) diagnostics: Rc<RefCell<LspDiagnosticsEmitter>>,
    pub(crate) context: CompilerContext<LspDiagnosticsEmitter>,
    pub(crate) asts: FxHashMap<String, Arc<fpp_ast::TransUnit>>,
    pub(crate) analysis: Arc<Analysis>,
}

impl GlobalState {
    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            analysis: self.analysis.clone(),
            asts: self.asts.clone(),
        }
    }

    fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap();
    }

    pub(crate) fn get_sender(&self) -> Sender<lsp_server::Message> {
        self.sender.clone()
    }

    pub(crate) fn send_request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) {
        let request = self.req_queue.outgoing.register(R::METHOD.to_owned(), params, handler);
        self.send(request.into());
    }

    pub(crate) fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_owned(), params);
        self.send(not.into());
    }

    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(&response.id) {
            let duration = start.elapsed();
            tracing::debug!(name: "message response", method, %response.id, duration = format_args!("{:0.2?}", duration));
            self.send(response.into());
        }
    }
}

pub struct GlobalStateSnapshot {
    analysis: Arc<Analysis>,
    asts: FxHashMap<String, Arc<fpp_ast::TransUnit>>,
}
