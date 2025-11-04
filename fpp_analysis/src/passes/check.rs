use crate::analysis::Analysis;
use crate::passes::EnterSymbols;

pub fn check_semantics<'ast, 'analysis>(
    a: &'analysis mut Analysis<'ast>,
    ast: &'ast fpp_ast::TranslationUnit,
) where
    'analysis: 'ast,
{
    EnterSymbols::run(a, ast);
}
