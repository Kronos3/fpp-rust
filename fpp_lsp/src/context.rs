use fpp_core::{DiagnosticData, DiagnosticEmitter};
use lsp_types::{Diagnostic, Uri};
use rustc_hash::FxHashMap;
use std::cell::RefCell;
use std::rc::Rc;

pub struct LspContext {
    diagnostics: Rc<RefCell<LspDiagnosticsEmitter>>,
}

impl LspContext {
    pub fn new(diagnostics: Rc<RefCell<LspDiagnosticsEmitter>>) -> LspContext {
        LspContext { diagnostics }
    }

    pub fn update(&mut self, uri: &Uri, content: String) {}
}

pub struct LspDiagnosticsEmitter {
    diagnostics: FxHashMap<Uri, Vec<Diagnostic>>,
}

impl LspDiagnosticsEmitter {
    pub fn new() -> LspDiagnosticsEmitter {
        LspDiagnosticsEmitter {
            diagnostics: Default::default(),
        }
    }
}

impl DiagnosticEmitter for LspDiagnosticsEmitter {
    fn emit(&mut self, diagnostic: DiagnosticData) {}
}
