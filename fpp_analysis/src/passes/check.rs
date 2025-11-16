use crate::analysis::Analysis;
use crate::passes::{
    CheckExprTypes, CheckTypeUses, CheckUseDefCycles, CheckUses, EnterSymbols, EvalConstantExprs,
    EvalImpliedEnumConsts, FinalizeTypeDefs,
};
use fpp_ast::{MutVisitor, Visitor};
use fpp_core::FileReader;
use std::ops::ControlFlow;

pub fn check_semantics<'ast>(
    a: &mut Analysis<'ast>,
    reader: Box<dyn FileReader>,
    ast: &'ast mut fpp_ast::TransUnit,
) -> ControlFlow<()> {
    fpp_parser::ResolveIncludes::new(reader).visit_trans_unit(&mut a.included_file_set, ast)?;
    EnterSymbols::new().visit_trans_unit(a, ast)?;
    CheckUses::new().visit_trans_unit(a, ast)?;
    CheckUseDefCycles::new().visit_trans_unit(a, ast)?;
    CheckTypeUses::new().visit_trans_unit(a, ast)?;
    CheckExprTypes::new().visit_trans_unit(a, ast)?;
    EvalImpliedEnumConsts::new().visit_trans_unit(a, ast)?;
    EvalConstantExprs::new().visit_trans_unit(a, ast)?;
    FinalizeTypeDefs::new().visit_trans_unit(a, ast)?;

    ControlFlow::Continue(())
}
