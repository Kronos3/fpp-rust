use crate::analyzers::analyzer::Analyzer;
use crate::analyzers::basic_use_analyzer::UseAnalysisPass;
use crate::analyzers::use_analyzer::UseAnalyzer;
use crate::errors::SemanticError;
use crate::semantics::{QualifiedName, Symbol, SymbolInterface, Type, TypeConversionResult};
use crate::Analysis;
use fpp_ast::{
    DefAliasType, DefArray, DefConstant, DefEnum, DefEnumConstant, Expr, Node, Visitable, Visitor,
};
use fpp_core::Spanned;
use std::cell::RefCell;
use std::ops::ControlFlow;
use std::rc::Rc;

pub struct CheckExprTypes<'ast> {
    super_: UseAnalyzer<'ast, Self>,
}

impl<'ast> CheckExprTypes<'ast> {
    pub fn new() -> CheckExprTypes<'ast> {
        Self {
            super_: UseAnalyzer::new(),
        }
    }

    fn convert_type(
        &self,
        a: &Analysis<'ast>,
        expr: &'ast Expr,
        to_ty: &Rc<RefCell<Type>>,
    ) -> TypeConversionResult {
        let from_ty = match a.type_map.get(&expr.node_id) {
            None => return Ok(()),
            Some(ty) => ty,
        };

        Type::convert(from_ty, to_ty)
    }

    fn check_type_is_numerical(&self, a: &Analysis<'ast>, expr: &'ast Expr) {
        match self.convert_type(a, expr, &Rc::new(RefCell::new(Type::Integer))) {
            Ok(_) => {}
            Err(err) => {
                SemanticError::TypeConversion {
                    loc: expr.span(),
                    msg: "cannot convert expression to integer".to_string(),
                    err,
                }
                .emit();
            }
        }
    }
}

impl<'ast> Visitor<'ast> for CheckExprTypes<'ast> {
    type Break = ();
    type State = Analysis<'ast>;

    fn super_visit(&self, a: &mut Analysis<'ast>, node: Node<'ast>) -> ControlFlow<Self::Break> {
        self.super_.visit(self, a, node)
    }

    fn visit_def_array(
        &self,
        a: &mut Self::State,
        node: &'ast DefArray,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::DefArray(node))?;
        self.check_type_is_numerical(a, &node.size);

        match (&node.default, a.type_map.get(&node.node_id)) {
            (Some(default), Some(arr_type)) => match self.convert_type(a, default, arr_type) {
                Ok(_) => {}
                Err(err) => SemanticError::TypeConversion {
                    loc: default.span(),
                    msg: format!("default value cannot be converted to {}", arr_type.borrow()),
                    err,
                }
                .emit(),
            },
            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn visit_def_constant(
        &self,
        a: &mut Self::State,
        node: &'ast DefConstant,
    ) -> ControlFlow<Self::Break> {
        if a.type_map.contains_key(&node.node_id) {
            return ControlFlow::Continue(());
        }

        self.super_visit(a, Node::DefConstant(node))?;
        match a.type_map.get(&node.value.node_id) {
            None => {}
            Some(ty) => {
                a.type_map.insert(node.node_id, ty.clone());
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_def_alias_type(
        &self,
        a: &mut Self::State,
        node: &'ast DefAliasType,
    ) -> ControlFlow<Self::Break> {
        if a.type_map.contains_key(&node.node_id) {
            return ControlFlow::Continue(());
        }

        self.super_visit(a, Node::DefAliasType(node))?;
        match a.type_map.get(&node.type_name.node_id) {
            None => {}
            Some(ty) => {
                a.type_map.insert(node.node_id, ty.clone());
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_def_enum(&self, a: &mut Self::State, node: &'ast DefEnum) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::DefEnum(node))?;

        match (&node.default, a.type_map.get(&node.node_id)) {
            (Some(default), Some(enum_type)) => match self.convert_type(a, default, enum_type) {
                Ok(_) => {}
                Err(err) => SemanticError::TypeConversion {
                    loc: default.span(),
                    msg: format!(
                        "default value cannot be converted to {}",
                        enum_type.borrow()
                    ),
                    err,
                }
                .emit(),
            },
            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn visit_def_enum_constant(
        &self,
        a: &mut Self::State,
        node: &'ast DefEnumConstant,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::DefEnumConstant(node))?;

        // Just check that the type of the value expression is convertible to numeric
        // The enum type of the enum constant node is already in the type map
        

        ControlFlow::Continue(())
    }
}

impl<'ast> UseAnalysisPass<'ast> for CheckExprTypes<'ast> {
    fn constant_use(
        &self,
        a: &mut Analysis<'ast>,
        node: &'ast Expr,
        _: QualifiedName,
    ) -> ControlFlow<Self::Break> {
        let symbol = match a.use_def_map.get(&node.node_id) {
            None => return ControlFlow::Continue(()),
            Some(symbol) => symbol.clone(),
        };

        match symbol {
            // Constant symbol: visit the constant definition
            // to ensure it has a type
            Symbol::Constant(def) => {
                def.visit(a, self)?;
            }
            // Enum symbol: if this is in scope, then we are in
            // the enum definition, so it already has a type
            Symbol::EnumConstant(_) => {}
            _ => {
                SemanticError::InvalidSymbol {
                    symbol_name: symbol.name().data.clone(),
                    msg: "not a constant symbol".to_string(),
                    loc: node.span(),
                    def_loc: symbol.name().span(),
                }
                .emit();
                return ControlFlow::Continue(());
            }
        }

        match a.type_map.get(&symbol.node()) {
            None => {}
            Some(ty) => {
                a.type_map.insert(node.node_id, ty.clone());
            }
        }
        ControlFlow::Continue(())
    }
}
