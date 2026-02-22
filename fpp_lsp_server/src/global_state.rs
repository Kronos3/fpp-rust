pub use crate::analysis::Task;
use crate::diagnostics::LspDiagnosticsEmitter;
use crate::{lsp, vfs};
use crossbeam_channel::{Receiver, Sender};
use fpp_analysis::Analysis;
use fpp_core::{CompilerContext, SourceFile};
use lsp_server::RequestId;
use lsp_types::{SemanticTokens, Uri};
use rustc_hash::FxHashMap;
use std::sync::{Arc};
use std::time::Instant;

pub(crate) type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

#[derive(Debug)]
pub struct TranslationUnitCache {
    pub uri: String,
    pub file: SourceFile,
    pub ast: fpp_ast::TransUnit,
    pub include_context_map: FxHashMap<SourceFile, fpp_parser::IncludeParentKind>,
    pub gc: fpp_core::GarbageCollectionSet,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Workspace {
    #[default]
    None,
    LocsFile(Uri),
    FullWorkspace(Uri),
}

pub struct TaskWithReply {
    task: Task,
    reply_to: Option<RequestId>,
}

pub struct GlobalState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,

    task_rx: Receiver<TaskWithReply>,
    task_tx: Sender<TaskWithReply>,

    pub(crate) vfs: vfs::Vfs,

    pub(crate) shutdown_requested: bool,

    pub(crate) workspace: Workspace,

    pub(crate) diagnostics: LspDiagnosticsEmitter,
    pub(crate) context: CompilerContext<LspDiagnosticsEmitter>,
    /// Top level files in project pointing to their translation unit
    pub(crate) cache: FxHashMap<SourceFile, Arc<TranslationUnitCache>>,
    pub(crate) files: FxHashMap<String, Vec<SourceFile>>,
    /// Computed compiler analysis
    pub(crate) analysis: Arc<Analysis>,

    pub(crate) capabilities: Arc<lsp::capabilities::ClientCapabilities>,

    /// Semantic tokens cache for computing deltas of semantic tokens
    pub(crate) semantic_tokens: FxHashMap<Uri, SemanticTokens>,
}

impl GlobalState {
    pub fn new(
        sender: Sender<lsp_server::Message>,
        capabilities: lsp::capabilities::ClientCapabilities,
    ) -> GlobalState {
        let (task_tx, task_rx) = crossbeam_channel::unbounded();
        let diagnostics: LspDiagnosticsEmitter = Default::default();

        GlobalState {
            sender,
            req_queue: Default::default(),
            task_rx,
            task_tx,
            vfs: vfs::Vfs::new(),
            shutdown_requested: false,
            workspace: Workspace::None,
            diagnostics: diagnostics.clone(),
            context: CompilerContext::new(diagnostics),
            cache: Default::default(),
            files: Default::default(),
            analysis: Arc::new(Analysis::new()),
            capabilities: Arc::new(capabilities),
            semantic_tokens: Default::default(),
        }
    }

    pub(crate) fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap();
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
        match self.task_tx.send(TaskWithReply {
            task,
            reply_to: None,
        }) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!(err = %e, "failed to queue task")
            }
        }
    }

    pub(crate) fn task_reply_to(&self, task: Task, reply_to: RequestId) {
        match self.task_tx.send(TaskWithReply {
            task,
            reply_to: Some(reply_to),
        }) {
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
                    if let Ok(msg) = msg {
                        self.on_task(msg.task);
                        if let Some(reply_id) = msg.reply_to {
                            // The previous task might have put some more tasks on the queue
                            // We don't want to synchronously respond here since it'll finish early
                            // Instead we just place another 'response' task on the queue
                            self.task(Task::Response(lsp_server::Response::new_ok(reply_id, &())))
                        }
                    }
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
