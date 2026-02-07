use crate::diagnostics::LspDiagnosticsEmitter;
use crate::progress::{CancellationToken, Progress};
use crate::{lsp, vfs};
use crossbeam_channel::{Receiver, Sender};
use fpp_analysis::Analysis;
use fpp_core::CompilerContext;
use lsp_types::{SemanticTokens, Uri};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use threadpool::ThreadPool;

pub(crate) type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

#[derive(Debug)]
pub enum Task {
    Response(lsp_server::Response),
    Notification(lsp_server::Notification),
    // Retry(lsp_server::Request),
    IndexWorkspace((Progress, Vec<(String, String)>)),
    Parse(Uri),
    Analysis(()),
}

pub struct GlobalState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,

    task_rx: Receiver<Task>,
    task_tx: Sender<Task>,

    pub(crate) cancellable: FxHashMap<lsp_types::ProgressToken, CancellationToken>,

    pub(crate) vfs: vfs::Vfs,

    pub(crate) task_pool: Arc<ThreadPool>,
    pub(crate) shutdown_requested: bool,

    pub(crate) workspace_locs: String,

    pub(crate) diagnostics: Arc<Mutex<LspDiagnosticsEmitter>>,
    pub(crate) context: Arc<Mutex<CompilerContext<LspDiagnosticsEmitter>>>,
    pub(crate) asts: FxHashMap<String, Arc<fpp_ast::TransUnit>>,
    pub(crate) analysis: Arc<Analysis>,

    pub(crate) capabilities: Arc<lsp::capabilities::ClientCapabilities>,

    pub(crate) semantic_tokens: Arc<Mutex<FxHashMap<Uri, SemanticTokens>>>,
}

impl GlobalState {
    pub fn new(
        sender: Sender<lsp_server::Message>,
        capabilities: lsp::capabilities::ClientCapabilities,
    ) -> GlobalState {
        let (task_tx, task_rx) = crossbeam_channel::unbounded();
        let task_pool = Arc::new(ThreadPool::new(10));

        let diagnostics: Arc<Mutex<LspDiagnosticsEmitter>> = Default::default();

        GlobalState {
            sender,
            req_queue: Default::default(),
            cancellable: Default::default(),
            task_rx,
            task_tx,
            vfs: vfs::Vfs::new(),
            task_pool: task_pool.clone(),
            shutdown_requested: false,
            workspace_locs: "".to_string(),
            diagnostics: diagnostics.clone(),
            context: Arc::new(Mutex::new(CompilerContext::new(diagnostics))),
            asts: Default::default(),
            analysis: Arc::new(Analysis::new()),
            capabilities: Arc::new(capabilities),
            semantic_tokens: Default::default(),
        }
    }

    pub(crate) fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap();
    }

    pub(crate) fn get_sender(&self) -> GlobalComm {
        GlobalComm(self.task_tx.clone())
    }

    pub(crate) fn register_request(
        &mut self,
        request: &lsp_server::Request,
        request_received: Instant,
    ) {
        self.req_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        );
    }

    pub(crate) fn send_request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) {
        let request = self
            .req_queue
            .outgoing
            .register(R::METHOD.to_owned(), params, handler);
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
            tracing::info!(name: "message response", method, %response.id, duration = format_args!("{:0.2?}", duration));
            self.send(response.into());
        } else {
            tracing::warn!(%response.id, "invalid response id")
        }
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            analysis: self.analysis.clone(),
            asts: self.asts.clone(),
            vfs: self.vfs.clone(),
            task_tx: self.task_tx.clone(),
            capabilities: self.capabilities.clone(),
            semantic_tokens: self.semantic_tokens.clone(),
            diagnostics: self.diagnostics.clone(),
            context: self.context.clone(),
        }
    }

    /// Registers and handles a request. This should only be called once per incoming request.
    fn on_new_request(&mut self, request_received: Instant, req: lsp_server::Request) {
        let _p =
            tracing::span!(tracing::Level::INFO, "GlobalState::on_new_request", req.method = ?req.method).entered();
        self.register_request(&req, request_received);
        self.on_request(req);
    }

    fn on_message(&mut self, start: Instant, msg: lsp_server::Message) {
        match msg {
            lsp_server::Message::Request(req) => {
                self.on_new_request(start, req);
            }
            lsp_server::Message::Response(res) => {
                match self.req_queue.outgoing.complete(res.id.clone()) {
                    None => {}
                    Some(handler) => handler(self, res),
                }
            }
            lsp_server::Message::Notification(not) => {
                self.on_notification(not);
            }
        }
    }

    pub(crate) fn task(&self, task: Task) {
        match self.task_tx.send(task) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!(err = %e, "failed to queue task")
            }
        }
    }

    fn main_loop(&mut self, receiver: Receiver<lsp_server::Message>) {
        while !self.shutdown_requested {
            crossbeam_channel::select_biased! {
                recv(self.task_rx) -> msg => {
                    if let Ok(msg) = msg { self.on_task(msg) }
                }
                recv(receiver) -> msg => {
                    if let Ok(msg) = msg {
                        self.on_message(Instant::now(), msg)
                    }
                }
            }
        }
    }

    pub fn run(
        connection: lsp_server::Connection,
        capabilities: lsp::capabilities::ClientCapabilities,
    ) {
        let mut state = GlobalState::new(connection.sender, capabilities);
        state.main_loop(connection.receiver);
    }
}

#[derive(Clone)]
pub struct GlobalComm(Sender<Task>);

impl GlobalComm {
    pub(crate) fn send(&self, task: Task) {
        match self.0.send(task) {
            Ok(_) => {}
            Err(err) => {
                tracing::error!(err = %err, "failed to queue task")
            }
        }
    }

    pub(crate) fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_owned(), params);
        self.send(Task::Notification(not.into()));
    }
}

pub struct GlobalStateSnapshot {
    pub context: Arc<Mutex<CompilerContext<LspDiagnosticsEmitter>>>,
    pub analysis: Arc<Analysis>,
    pub diagnostics: Arc<Mutex<LspDiagnosticsEmitter>>,
    pub asts: FxHashMap<String, Arc<fpp_ast::TransUnit>>,
    pub semantic_tokens: Arc<Mutex<FxHashMap<Uri, SemanticTokens>>>,
    pub vfs: vfs::Vfs,
    pub capabilities: Arc<lsp::capabilities::ClientCapabilities>,
    pub task_tx: Sender<Task>,
}

impl GlobalStateSnapshot {
    pub(crate) fn task(&self, task: Task) {
        match self.task_tx.send(task) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!(err = %e, "failed to queue task")
            }
        }
    }
}
