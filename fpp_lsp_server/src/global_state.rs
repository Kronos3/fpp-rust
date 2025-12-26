use crate::context::LspDiagnosticsEmitter;
use crate::progress::{CancellationToken, Progress};
use crate::vfs;
use fpp_analysis::Analysis;
use fpp_core::CompilerContext;
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Instant;
use threadpool::ThreadPool;

pub(crate) type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

#[derive(Debug)]
pub struct ProcessFile {
    pub key: String,
    pub content: String,
    pub progress: Option<Progress>,
}

#[derive(Debug)]
pub enum Task {
    // Response(lsp_server::Response),
    // Retry(lsp_server::Request),
    IndexWorkspace((Progress, Vec<(String, String)>)),
}

pub enum Event {
    Lsp(lsp_server::Message),
    Task(Task),
    Vfs(vfs::Message),
}

pub struct GlobalState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,

    inbox: Receiver<Event>,

    pub(crate) cancellable: FxHashMap<lsp_types::ProgressToken, CancellationToken>,
    pub(crate) inbox_tx: Sender<Event>,

    pub(crate) vfs: vfs::Vfs,

    pub(crate) task_pool: Arc<ThreadPool>,
    pub(crate) shutdown_requested: bool,
    pub(crate) refresh_semantics: bool,

    pub(crate) workspace: String,
    pub(crate) workspace_locs: String,

    pub(crate) diagnostics: Rc<RefCell<LspDiagnosticsEmitter>>,
    pub(crate) context: CompilerContext<LspDiagnosticsEmitter>,
    pub(crate) asts: FxHashMap<String, Arc<fpp_ast::TransUnit>>,
    pub(crate) analysis: Arc<Analysis>,
}

impl GlobalState {
    pub fn new(sender: Sender<lsp_server::Message>) -> GlobalState {
        let (tx, rx) = std::sync::mpsc::channel();
        let task_pool = Arc::new(ThreadPool::new(10));

        GlobalState {
            sender,
            req_queue: Default::default(),
            cancellable: Default::default(),
            inbox: rx,
            inbox_tx: tx.clone(),
            vfs: vfs::Vfs::new(),
            task_pool: task_pool.clone(),
            shutdown_requested: false,
            refresh_semantics: false,
            workspace: "".to_string(),
            workspace_locs: "".to_string(),
            diagnostics: Rc::new(RefCell::new(LspDiagnosticsEmitter {
                diagnostics: Default::default(),
            })),
            context: (),
            asts: Default::default(),
            analysis: Arc::new(Analysis::new()),
        }
    }

    fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap();
    }

    pub(crate) fn get_sender(&self) -> GlobalComm {
        GlobalComm {
            tx: self.inbox_tx.clone(),
            sender: self.sender.clone(),
        }
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
            tracing::debug!(name: "message response", method, %response.id, duration = format_args!("{:0.2?}", duration));
            self.send(response.into());
        }
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            analysis: self.analysis.clone(),
            asts: self.asts.clone(),
            inbox_tx: self.inbox_tx.clone(),
        }
    }

    fn main_loop(&mut self, inbox: Receiver<Event>) {
        for event in inbox {
            match event {
                Event::Lsp(lsp_server::Message::Request(req)) => {
                    self.on_request(req);
                }
                Event::Lsp(lsp_server::Message::Response(res)) => {
                    match self.req_queue.outgoing.complete(res.id.clone()) {
                        None => {}
                        Some(handler) => handler(self, res),
                    }
                }
                Event::Lsp(lsp_server::Message::Notification(not)) => {
                    self.on_notification(not);
                }
                Event::Task(task) => self.on_task(task),
                Event::Vfs(msg) => self.vfs.on_message(msg),
            }

            if self.shutdown_requested {
                tracing::info!("shutdown requested, exiting main loop");
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct GlobalComm {
    // Message channel to main event loop
    tx: Sender<Event>,
    // Message channel to IDE client
    sender: Sender<lsp_server::Message>,
}

impl GlobalComm {
    pub(crate) fn send(&self, message: lsp_server::Message) {
        match self.sender.send(message) {
            Ok(_) => {}
            Err(err) => {
                tracing::error!(err = %err, "failed to message")
            }
        }
    }

    pub(crate) fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_owned(), params);
        self.send(not.into());
    }

    pub(crate) fn send_inbox(&self, ev: Event) {
        self.tx.send(ev).unwrap();
    }
}

pub struct GlobalStateSnapshot {
    pub analysis: Arc<Analysis>,
    pub asts: FxHashMap<String, Arc<fpp_ast::TransUnit>>,
    inbox_tx: Sender<Event>,
}

impl GlobalStateSnapshot {
    pub(crate) fn task(&self, task: Task) {
        match self.inbox_tx.send(Event::Task(task)) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!(err = %e, "failed to queue task")
            }
        }
    }
}
