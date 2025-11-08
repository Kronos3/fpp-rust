use crate::analyzers::analyzer::Analyzer;
use crate::analyzers::basic_use_analyzer::{BasicUseAnalyzer, UseAnalysisPass};
use crate::errors::{SemanticError, SemanticResult};
use crate::semantics::{NameGroup, QualifiedName, Symbol, SymbolInterface};
use crate::Analysis;
use fpp_ast::{AstNode, Expr, Ident, Node, QualIdent, Visitor};
use fpp_core::Spanned;
use std::ops::{ControlFlow, Deref};

pub(crate) struct CheckUses<'ast> {
    super_: BasicUseAnalyzer<'ast, Self>,
}

impl<'ast> CheckUses<'ast> {
    pub fn new() -> CheckUses<'ast> {
        Self {
            super_: BasicUseAnalyzer::new(),
        }
    }

    fn visit_qualified(
        &self,
        a: &mut Analysis<'ast>,
        ng: NameGroup,
        qualifier: &QualIdent,
        name: &Ident,
    ) -> SemanticResult<Symbol<'ast>> {
        self.visit_qual_ident_impl(a, ng, qualifier)?;
        let scope = {
            let symbol = a.use_def_map.get(&qualifier.id()).unwrap();
            match a.symbol_scope_map.get(symbol) {
                None => {
                    return Err(SemanticError::InvalidSymbol {
                        symbol_name: symbol.name().data.clone(),
                        msg: "not a qualifier".to_string(),
                        loc: qualifier.span(),
                        def_loc: symbol.node().span(),
                    });
                }
                Some(scope) => scope.borrow(),
            }
        };

        match scope.get(ng, &name.data) {
            None => Err(SemanticError::UndefinedSymbol {
                name: name.data.clone(),
                loc: name.span(),
            }),
            Some(sym) => {
                a.use_def_map.insert(name.id(), sym);
                Ok(sym)
            }
        }
    }

    fn visit_qual_ident_impl(
        &self,
        a: &mut Analysis<'ast>,
        ng: NameGroup,
        node: &QualIdent,
    ) -> SemanticResult<Symbol<'ast>> {
        match &node {
            QualIdent::Unqualified(name) => match a.nested_scope.get(ng, &name.data) {
                None => Err(SemanticError::UndefinedSymbol {
                    name: name.data.clone(),
                    loc: name.span(),
                }),
                Some(sym) => {
                    a.use_def_map.insert(name.id(), sym);
                    Ok(sym)
                }
            },
            QualIdent::Qualified(qualified) => {
                self.visit_qualified(a, ng, qualified.qualifier.deref(), &qualified.name)
            }
        }
    }

    fn visit_qual_ident(&self, a: &mut Analysis<'ast>, ng: NameGroup, node: &QualIdent) {
        match self.visit_qual_ident_impl(a, ng, node) {
            Ok(sym) => {
                a.use_def_map.insert(node.id(), sym);
            }
            Err(err) => {
                err.emit();
            }
        }
    }
}

impl<'ast> Visitor<'ast> for CheckUses<'ast> {
    type Break = ();
    type State = Analysis<'ast>;

    // Walk all nodes deeply and collect up scope where relevant
    fn visit(&self, a: &mut Analysis<'ast>, node: Node<'ast>) -> ControlFlow<Self::Break> {
        self.super_.visit(self, a, node)
    }
}

impl<'ast> UseAnalysisPass<'ast> for CheckUses<'ast> {
    fn component_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let _ = name;
        self.visit_qual_ident(a, NameGroup::Component, node);
        ControlFlow::Continue(())
    }

    fn interface_instance_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let _ = name;
        self.visit_qual_ident(a, NameGroup::PortInterfaceInstance, node);
        ControlFlow::Continue(())
    }

    fn constant_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &Expr,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let _ = name;
        let _ = a;
        let _ = node;
        // self.visit_qual_ident(a, NameGroup::Value, node);
        ControlFlow::Continue(())
    }

    fn port_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let _ = name;
        self.visit_qual_ident(a, NameGroup::Port, node);
        ControlFlow::Continue(())
    }

    fn interface_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let _ = name;
        self.visit_qual_ident(a, NameGroup::PortInterface, node);
        ControlFlow::Continue(())
    }

    fn type_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let _ = name;
        self.visit_qual_ident(a, NameGroup::Type, node);
        ControlFlow::Continue(())
    }

    fn state_machine_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &QualIdent,
        name: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let _ = name;
        self.visit_qual_ident(a, NameGroup::StateMachine, node);
        ControlFlow::Continue(())
    }
}
