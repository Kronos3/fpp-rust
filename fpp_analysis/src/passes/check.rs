use crate::analysis::Analysis;
use crate::passes::{CheckUses, EnterSymbols};
use fpp_ast::{MutVisitor, Visitor};
use std::ops::ControlFlow;

pub fn check_semantics<'ast>(
    a: &mut Analysis<'ast>,
    ast: &'ast mut fpp_ast::TransUnit,
) -> ControlFlow<()> {
    fpp_parser::ResolveSpecInclude {}.visit_trans_unit(&mut a.included_file_set, ast)?;
    EnterSymbols::new().visit_trans_unit(a, ast)?;
    CheckUses::new().visit_trans_unit(a, ast)?;

    ControlFlow::Continue(())
}
