use std::ops::ControlFlow;
use fpp_ast::{Node, Visitor};

pub trait Analyzer<'ast, V: Visitor<'ast>> {
    fn visit(
        &self,
        visitor: &V,
        a: &mut V::State,
        node: Node<'ast>
    ) -> ControlFlow<V::Break>;
}
