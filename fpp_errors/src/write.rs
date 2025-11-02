use crate::snippet::diagnostic_to_snippet_group;
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::Renderer;
use fpp_core::{DiagnosticData, DiagnosticEmitter};
use std::io::Write;

pub struct WriteEmitter<W: Write> {
    renderer: Renderer,
    write: W,
}

impl<W: Write> WriteEmitter<W> {
    pub fn new(w: W) -> WriteEmitter<W> {
        WriteEmitter {
            renderer: Renderer::plain().decor_style(DecorStyle::Ascii),
            write: w,
        }
    }
}

impl<W: Write> DiagnosticEmitter for WriteEmitter<W> {
    fn emit<'d>(&'_ mut self, diagnostic: DiagnosticData<'d>) {
        let group = diagnostic_to_snippet_group(&diagnostic);
        let out = self.renderer.render(&[group]);
        self.write
            .write(out.as_bytes())
            .expect("failed to write diagnostic");
    }
}
