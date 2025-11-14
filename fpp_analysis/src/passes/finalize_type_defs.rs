use crate::analyzers::analyzer::Analyzer;
use crate::analyzers::nested_analyzer::{NestedAnalyzer, NestedAnalyzerMode};
use crate::Analysis;
use fpp_ast::{Expr, Node, TransUnit, Visitor};
use std::ops::ControlFlow;

pub struct FinalizeTypeDefs<'ast> {
    super_: NestedAnalyzer<'ast, Self>,
}

impl<'ast> FinalizeTypeDefs<'ast> {
    pub fn new() -> FinalizeTypeDefs<'ast> {
        Self {
            super_: NestedAnalyzer::new(NestedAnalyzerMode::DEEP),
        }
    }
}

impl<'ast> Visitor<'ast> for FinalizeTypeDefs<'ast> {
    type Break = ();
    type State = Analysis<'ast>;

    fn super_visit(&self, a: &mut Analysis<'ast>, node: Node<'ast>) -> ControlFlow<Self::Break> {
        self.super_.visit(self, a, node)
    }

    // No more need to evaluate sub-expressions
    fn visit_expr(&self, _: &mut Self::State, _: &'ast Expr) -> ControlFlow<Self::Break> {
        ControlFlow::Continue(())
    }

    fn visit_trans_unit(
        &self,
        a: &mut Self::State,
        node: &'ast TransUnit,
    ) -> ControlFlow<Self::Break> {
        a.visited_symbol_set.clear();
        self.super_visit(a, Node::TransUnit(node))
    }
}
