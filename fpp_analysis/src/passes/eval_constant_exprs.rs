use crate::analyzers::analyzer::Analyzer;
use crate::analyzers::basic_use_analyzer::UseAnalysisPass;
use crate::analyzers::use_analyzer::UseAnalyzer;
use crate::errors::SemanticError;
use crate::semantics::{
    EnumConstantValue, IntegerValue, QualifiedName, Symbol, SymbolInterface, Type, Value,
};
use crate::Analysis;
use fpp_ast::{DefConstant, DefEnum, DefEnumConstant, Expr, Node, Visitable, Visitor};
use fpp_core::Spanned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::ControlFlow;
use std::rc::Rc;

pub struct EvalConstantExprs<'ast> {
    super_: UseAnalyzer<'ast, Self>,
}

impl<'ast> EvalConstantExprs<'ast> {
    pub fn new() -> EvalConstantExprs<'ast> {
        Self {
            super_: UseAnalyzer::new(),
        }
    }
}

impl<'ast> Visitor<'ast> for EvalConstantExprs<'ast> {
    type Break = ();
    type State = Analysis<'ast>;

    fn super_visit(&self, a: &mut Analysis<'ast>, node: Node<'ast>) -> ControlFlow<Self::Break> {
        self.super_.visit(self, a, node)
    }

    fn visit_def_constant(
        &self,
        a: &mut Self::State,
        node: &'ast DefConstant,
    ) -> ControlFlow<Self::Break> {
        if a.value_map.contains_key(&node.node_id) {
            return ControlFlow::Continue(());
        }

        self.super_visit(a, Node::DefConstant(node))?;
        match a.value_map.get(&node.value.node_id) {
            None => {}
            Some(value) => {
                a.value_map.insert(node.node_id, value.clone());
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_def_enum(&self, a: &mut Self::State, node: &'ast DefEnum) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::DefEnum(node))?;

        // Check for duplicate values
        let mut values: HashMap<i128, fpp_core::Span> = HashMap::default();
        for constant in &node.constants {
            match a.value_map.get(&constant.node_id) {
                Some(Value::EnumConstant(EnumConstantValue { value, .. })) => {
                    if let Some(old) = values.insert(value.1, constant.span()) {
                        SemanticError::DuplicateEnumConstant {
                            value: value.1,
                            loc: constant.span(),
                            prev_loc: old,
                        }
                        .emit()
                    }
                }
                _ => {}
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_def_enum_constant(
        &self,
        a: &mut Self::State,
        node: &'ast DefEnumConstant,
    ) -> ControlFlow<Self::Break> {
        if a.value_map.contains_key(&node.node_id) {
            return ControlFlow::Continue(());
        }

        self.super_visit(a, Node::DefEnumConstant(node))?;

        fn apply_value<'ast>(a: &mut Analysis<'ast>, node: &'ast DefEnumConstant) -> Option<()> {
            let value_expr = match &node.value {
                None => return None,
                Some(v) => v,
            };

            let value = match a
                .value_map
                .get(&value_expr.node_id)?
                .convert(&Rc::new(RefCell::new(Type::Integer)))?
            {
                Value::Integer(IntegerValue(value)) => value,
                _ => panic!("expected integer value"),
            };

            let ty = a.type_map.get(&node.node_id)?;
            a.value_map.insert(
                node.node_id,
                Value::EnumConstant(EnumConstantValue::new(
                    node.name.data.clone(),
                    value,
                    ty.clone(),
                )),
            );

            Some(())
        }

        let _ = apply_value(a, node);
        ControlFlow::Continue(())
    }
}

impl<'ast> UseAnalysisPass<'ast> for EvalConstantExprs<'ast> {
    fn constant_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast Expr,
        _: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let symbol = match a.use_def_map.get(&node.node_id) {
            Some(sym @ Symbol::Constant(def)) => {
                let sym = sym.clone();
                def.visit(a, self)?;
                sym
            }
            Some(sym @ Symbol::EnumConstant(def)) => {
                let sym = sym.clone();
                def.visit(a, self)?;
                sym
            }
            _ => return ControlFlow::Continue(()),
        };

        match a.value_map.get(&symbol.node()) {
            Some(value) => {
                a.value_map.insert(node.node_id, value.clone());
            }
            None => {}
        }

        ControlFlow::Continue(())
    }
}
