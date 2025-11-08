use crate::analyzers::analyzer::Analyzer;
use crate::semantics::Symbol;
use crate::Analysis;
use fpp_ast::{MoveWalkable, Node, Visitor};
use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::ops::ControlFlow::Continue;

pub struct NestedAnalyzer<'ast, V: Visitor<'ast, State = Analysis<'ast>>> {
    phantom_data: PhantomData<&'ast V>,
}

impl<'ast, V: Visitor<'ast, State = Analysis<'ast>>> NestedAnalyzer<'ast, V> {
    pub fn new() -> NestedAnalyzer<'ast, V> {
        NestedAnalyzer {
            phantom_data: Default::default(),
        }
    }

    fn walk_symbol(
        &self,
        visitor: &V,
        a: &mut V::State,
        symbol: Symbol<'ast>,
        node: Node<'ast>,
    ) -> ControlFlow<V::Break> {
        let sym_scope = a.symbol_scope_map.get(&symbol).unwrap().clone();
        a.nested_scope.push(sym_scope);
        let out = node.walk(a, visitor);
        a.nested_scope.pop();
        out
    }
}

impl<'ast, V: Visitor<'ast, State = Analysis<'ast>>> Analyzer<'ast, V> for NestedAnalyzer<'ast, V> {
    fn visit(&self, visitor: &V, a: &mut V::State, node: Node<'ast>) -> ControlFlow<V::Break> {
        match &node {
            Node::DefComponent(def) => self.walk_symbol(visitor, a, Symbol::Component(def), node),
            Node::DefEnum(def) => self.walk_symbol(visitor, a, Symbol::Enum(def), node),
            Node::DefModule(def) => self.walk_symbol(visitor, a, Symbol::Module(def), node),
            _ => Continue(()),
        }
    }
}
