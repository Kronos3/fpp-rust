use crate::analysis::Analysis;
use crate::passes::EnterSymbols;

pub fn check_semantics(a: &Analysis, ast: &fpp_ast::TranslationUnit) {
    EnterSymbols::run(a, ast);
}
