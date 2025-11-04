use crate::analysis::Analysis;
use crate::semantics::Symbol;
use fpp_ast::{DefAbsType, DefComponent, DefModule, Visitor, Walkable};
use std::ops::ControlFlow;

pub(crate) struct EnterSymbols<'a> {
    a: &'a Analysis<'a>,
    parent: Option<Symbol<'a>>,
}

impl<'a> EnterSymbols<'a> {
    pub fn run(a: &Analysis, ast: &fpp_ast::TranslationUnit) {
        let mut pass = EnterSymbols { a, parent: None };
        let _ = ast.walk_ref(&mut pass);
    }

    fn enter_symbol(&mut self, _: Symbol<'a>) {}
}

impl<'ast> Visitor<'ast> for EnterSymbols<'ast> {
    type Break = ();

    // Don't deeply traverse the AST
    // We only care about definitions that can create symbols
    const DEFAULT: ControlFlow<Self::Break> = ControlFlow::Break(());

    fn visit_def_abs_type(&mut self, def: &'ast DefAbsType) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::AbsType(def));
        ControlFlow::Break(())
    }

    fn visit_def_component(&mut self, def: &'ast DefComponent) -> ControlFlow<Self::Break> {
        self.enter_symbol(Symbol::Component(def));
        ControlFlow::Break(())
    }

    fn visit_def_module(&mut self, def: &DefModule) -> ControlFlow<Self::Break> {
        let save_paren = self.parent;
        ControlFlow::Continue(())
    }
}
