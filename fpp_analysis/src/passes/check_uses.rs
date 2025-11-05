use crate::Analysis;
use fpp_ast::{TranslationUnit, Visitor, Walkable};
use std::ops::ControlFlow;

pub(crate) struct CheckUses<'a> {
    a: &'a mut Analysis<'a>,
}

impl<'a> CheckUses<'a> {
    pub fn run<'analysis>(a: &'analysis mut Analysis<'a>, ast: &'a TranslationUnit)
    where
        'analysis: 'a,
    {
        let mut pass = Self { a };
        let _ = ast.walk_ref(&mut pass, ());
    }
}

impl<'ast> Visitor<'ast> for CheckUses<'ast> {
    type Break = ();

    // Walk all nodes deeply
    fn visit<V: Walkable<'ast, Self>>(
        &mut self,
        node: &'ast V,
        extra: V::Extra,
    ) -> ControlFlow<Self::Break> {
        node.walk_ref(self, extra)
    }
}
