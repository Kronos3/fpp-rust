use crate::snippet::diagnostic_to_snippet_group;
use annotate_snippets::renderer::DecorStyle;
use annotate_snippets::Renderer;
use fpp_core::DiagnosticData;

pub struct ConsoleEmitter {
    renderer: Renderer,
}

impl ConsoleEmitter {
    pub fn color() -> ConsoleEmitter {
        ConsoleEmitter {
            renderer: Renderer::styled().decor_style(DecorStyle::Ascii),
        }
    }

    pub fn plain() -> ConsoleEmitter {
        ConsoleEmitter {
            renderer: Renderer::plain().decor_style(DecorStyle::Ascii),
        }
    }
}

impl fpp_core::DiagnosticEmitter for ConsoleEmitter {
    fn emit<'d>(&'_ mut self, diagnostic: DiagnosticData<'d>) {
        let group = diagnostic_to_snippet_group(&diagnostic);
        anstream::println!("{}\n", self.renderer.render(&[group]));
    }
}
