use crate::analyzers::analyzer::Analyzer;
use crate::semantics::Symbol;
use crate::Analysis;
use fpp_ast::{MoveWalkable, Node, Visitor};
use std::marker::PhantomData;
use std::ops::ControlFlow;

pub enum NestedAnalyzerMode {
    SHALLOW,
    DEEP,
}

pub struct NestedAnalyzer<'ast, V: Visitor<'ast, State = Analysis>> {
    phantom_data: PhantomData<&'ast V>,
    mode: NestedAnalyzerMode,
}

impl<'ast, V: Visitor<'ast, State = Analysis>> NestedAnalyzer<'ast, V> {
    pub fn new(mode: NestedAnalyzerMode) -> NestedAnalyzer<'ast, V> {
        NestedAnalyzer {
            phantom_data: Default::default(),
            mode,
        }
    }

    fn walk_symbol(
        &self,
        visitor: &V,
        a: &mut V::State,
        symbol: Symbol,
        node: Node<'ast>,
    ) -> ControlFlow<V::Break> {
        a.nested_scope.push(symbol);
        let out = node.walk(a, visitor);
        a.nested_scope.pop();
        out
    }
}

impl<'ast, V: Visitor<'ast, State = Analysis>> Analyzer<'ast, V> for NestedAnalyzer<'ast, V> {
    fn visit(&self, visitor: &V, a: &mut V::State, node: Node<'ast>) -> ControlFlow<V::Break> {
        match node {
            Node::DefComponent(def) => self.walk_symbol(visitor, a, a.get_symbol(def), node),
            Node::DefEnum(def) => self.walk_symbol(visitor, a, a.get_symbol(def), node),
            Node::DefModule(def) => self.walk_symbol(visitor, a, a.get_symbol(def), node),
            _ => match self.mode {
                NestedAnalyzerMode::SHALLOW => ControlFlow::Continue(()),
                NestedAnalyzerMode::DEEP => node.walk(a, visitor),
            },
        }
    }
}
