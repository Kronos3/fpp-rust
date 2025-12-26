use crate::global_state::{GlobalComm, GlobalState};
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CancellationToken(Arc<Mutex<bool>>);

impl CancellationToken {
    fn new() -> CancellationToken {
        CancellationToken(Arc::new(Mutex::new(false)))
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.lock().unwrap().clone()
    }
}

#[derive(Clone)]
pub(crate) struct Progress {
    /// LSP message sender
    sender: GlobalComm,
    /// Progress token
    token: lsp_types::ProgressToken,
    /// Number of work items completed
    done: Arc<Mutex<usize>>,
    /// Total number of work items
    total: usize,
}

impl Debug for Progress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Progress")
            .field("token", &self.token)
            .field("done", &*self.done.lock().unwrap())
            .field("total", &self.total)
            .finish()
    }
}

impl Progress {
    fn send_notification(&self, params: lsp_types::WorkDoneProgress) {
        self.sender
            .send_notification::<lsp_types::notification::Progress>(lsp_types::ProgressParams {
                token: self.token.clone(),
                value: lsp_types::ProgressParamsValue::WorkDone(params),
            });
    }

    pub(crate) fn with_total(self, total: usize) -> Progress {
        Progress {
            sender: self.sender,
            token: self.token,
            done: self.done,
            total,
        }
    }

    /// Report a work item has finished
    pub(crate) fn report(&self, message: &str) {
        let (percentage, done): (u32, usize) = {
            if self.total == 0 {
                // Unknown total, assume 0%
                (0, 0)
            } else {
                let mut done_guard = self.done.lock().unwrap();
                *done_guard += 1;
                let done = *done_guard;
                let f = done as f64 / self.total.max(1) as f64;
                ((f * 100.0) as u32, done)
            }
        };

        let msg = if self.total == 0 {
            message.to_string()
        } else {
            format!("{}/{} {}", done, self.total, message)
        };

        self.send_notification(lsp_types::WorkDoneProgress::Report(
            lsp_types::WorkDoneProgressReport {
                cancellable: Some(true),
                message: Some(msg),
                percentage: Some(percentage.min(100)),
            },
        ));
    }

    pub(crate) fn finish(&self, message: Option<String>) {
        self.send_notification(lsp_types::WorkDoneProgress::End(
            lsp_types::WorkDoneProgressEnd { message },
        ));
    }
}

#[derive(Debug, Eq, PartialEq)]
enum ProgressState {
    Begin,
    Report,
    End,
}

impl ProgressState {
    fn fraction(done: usize, total: usize) -> f64 {
        assert!(done <= total);
        done as f64 / total.max(1) as f64
    }
}

impl GlobalState {
    fn create_cancellation_token(
        &mut self,
        name: &str,
    ) -> (CancellationToken, lsp_types::ProgressToken) {
        let cancel_token = format!("fpp/{name}");
        let token_key = lsp_types::ProgressToken::String(cancel_token);
        let token = CancellationToken::new();

        self.cancellable.insert(token_key.clone(), token.clone());
        (token, token_key)
    }

    pub(crate) fn new_progress(
        &mut self,
        title: &str,
        total: usize,
    ) -> (Progress, CancellationToken) {
        let (cancellation_token, token) = self.create_cancellation_token(title);
        assert_ne!(total, 0);

        // Tell the client that work is being done
        self.send_request::<lsp_types::request::WorkDoneProgressCreate>(
            lsp_types::WorkDoneProgressCreateParams {
                token: token.clone(),
            },
            |_, _| (),
        );

        let progress = Progress {
            sender: self.get_sender(),
            token,
            done: Arc::new(Mutex::new(0)),
            total,
        };

        // Send the initial progress notification
        progress.send_notification(lsp_types::WorkDoneProgress::Begin(
            lsp_types::WorkDoneProgressBegin {
                title: title.into(),
                cancellable: Some(true),
                message: None,
                percentage: Some(0),
            },
        ));

        (progress, cancellation_token)
    }
}
