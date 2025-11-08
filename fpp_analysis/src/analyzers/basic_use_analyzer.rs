use crate::analyzers::analyzer::Analyzer;
use crate::analyzers::nested_analyzer::NestedAnalyzer;
use crate::semantics::QualifiedName;
use crate::Analysis;
use fpp_ast::*;
use std::ops::{ControlFlow, Deref};

/// An extension of the standard [Visitor] trait that allows analyzing uses of symbols
/// [BasicUseAnalyzer] or [UseAnalyzer] should be used in your pass for this to work properly
pub trait UseAnalysisPass<'ast>: Visitor<'ast, State = Analysis<'ast>> {
    /** A use of a component definition */
    fn component_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break>;

    /** A use of an interface instance (topology def or component instance def) */
    fn interface_instance_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break>;

    /** A use of a constant definition or enumerated constant definition */
    fn constant_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast Expr,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break>;

    /** A use of a port definition */
    fn port_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break>;

    /** A use of an interface definition */
    fn interface_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break>;

    /** A use of a type definition */
    fn type_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break>;

    /** A use of a state machine definition*/
    fn state_machine_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break>;
}

pub struct BasicUseAnalyzer<'ast, V: UseAnalysisPass<'ast>> {
    super_: NestedAnalyzer<'ast, V>,
}

impl<'ast, V: UseAnalysisPass<'ast>> BasicUseAnalyzer<'ast, V> {
    pub fn new() -> BasicUseAnalyzer<'ast, V> {
        BasicUseAnalyzer {
            super_: NestedAnalyzer::new(),
        }
    }
}

impl<'ast, V: UseAnalysisPass<'ast>> Analyzer<'ast, V> for BasicUseAnalyzer<'ast, V> {
    fn visit(&self, visitor: &V, a: &mut V::State, node: Node<'ast>) -> ControlFlow<V::Break> {
        match node {
            Node::DefComponentInstance(ci) => {
                visitor.component_use(a, &ci.component, (&ci.component).into())?;
                ci.walk_ref(a, visitor)
            }
            Node::DefTopology(t) => {
                for i in &t.implements {
                    visitor.interface_instance_use(a, i, i.into())?;
                }

                t.walk_ref(a, visitor)
            }
            Node::Expr(e) => match &e.kind {
                ExprKind::Dot { e: e1, .. } => {
                    fn name_opt(e: &Expr, mut qualifier: Vec<String>) -> Option<QualifiedName> {
                        match &e.kind {
                            ExprKind::Ident(id) => {
                                qualifier.push(id.clone());
                                Some(qualifier.into())
                            }
                            ExprKind::Dot { e: e1, id } => {
                                qualifier.push(id.data.clone());
                                name_opt(e1.deref(), qualifier)
                            }
                            _ => None,
                        }
                    }

                    match name_opt(&e, vec![]) {
                        // Assume the entire qualified identifier is a use
                        Some(use_) => visitor.constant_use(a, e, use_),
                        // This is some other type of dot expression (not a qual ident)
                        // Analyze the left side, which may contain constant uses
                        None => self.visit(visitor, a, Node::Expr(&e1)),
                    }
                }
                ExprKind::Ident(id) => visitor.constant_use(a, e, id.clone().into()),
                _ => ControlFlow::Continue(()),
            },
            Node::SpecInstance(si) => {
                visitor.interface_instance_use(a, &si.instance, (&si.instance).into())
            }
            Node::SpecStateMachineInstance(si) => {
                visitor.state_machine_use(a, &si.state_machine, (&si.state_machine).into())?;
                si.walk_ref(a, visitor)
            }
            Node::TlmChannelIdentifier(ci) => visitor.interface_instance_use(
                a,
                &ci.component_instance,
                (&ci.component_instance).into(),
            ),
            _ => self.super_.visit(visitor, a, node),
        }
    }
}
