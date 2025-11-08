use crate::analysis::Analysis;
use crate::passes::{CheckUses, EnterSymbols};
use fpp_ast::Walkable;
use std::ops::ControlFlow;

pub fn check_semantics<'ast>(
    a: &mut Analysis<'ast>,
    ast: &'ast fpp_ast::TranslationUnit,
) -> ControlFlow<()> {
    ast.walk_ref(a, &EnterSymbols::new())?;
    ast.walk_ref(a, &CheckUses::new())?;

    ControlFlow::Continue(())
}
