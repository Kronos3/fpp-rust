use fpp_ast::{Visitor, Walkable};
use std::ops::ControlFlow;

struct BasicUseAnalyzer {}

impl<'a> Visitor<'a> for BasicUseAnalyzer {
    type Break = ();

    fn visit<V: Walkable<'a, Self>>(
        &mut self,
        node: &'a V,
        extra: V::Extra,
    ) -> ControlFlow<Self::Break> {
        node.walk_ref(self, extra)
    }
}
