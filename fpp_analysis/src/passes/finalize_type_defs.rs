use crate::analyzers::analyzer::Analyzer;
use crate::analyzers::nested_analyzer::{NestedAnalyzer, NestedAnalyzerMode};
use crate::errors::SemanticError;
use crate::semantics::{
    AliasType, AnonArrayType, ArrayType, Format, IntegerValue, Symbol, Type, Value,
};
use crate::Analysis;
use fpp_ast::{
    AstNode, DefAliasType, DefArray, Expr, Node, TransUnit, TypeName, TypeNameKind, Visitor,
};
use fpp_core::Spanned;
use std::ops::ControlFlow;
use std::rc::Rc;

pub struct FinalizeTypeDefs<'ast> {
    super_: NestedAnalyzer<'ast, Self>,
}

impl<'ast> FinalizeTypeDefs<'ast> {
    pub fn new() -> FinalizeTypeDefs<'ast> {
        Self {
            super_: NestedAnalyzer::new(NestedAnalyzerMode::DEEP),
        }
    }

    fn expr_as_integer(&self, a: &mut Analysis<'ast>, e: &Expr) -> Option<i128> {
        match a.value_map.get(&e.node_id) {
            None => None,
            Some(v) => {
                if let Some(Value::Integer(IntegerValue(i))) = v.convert(&Rc::new(Type::Integer)) {
                    Some(i)
                } else {
                    None
                }
            }
        }
    }

    fn expr_as_integer_opt(&self, a: &mut Analysis<'ast>, e: &Option<Expr>) -> Option<i128> {
        match e {
            None => None,
            Some(e) => self.expr_as_integer(a, e),
        }
    }

    fn ty(&self, a: &mut Analysis<'ast>, node: &'ast TypeName) -> Option<Rc<Type>> {
        match &node.kind {
            TypeNameKind::QualIdent(q) => match a.use_def_map.get(&q.id()) {
                None => {}
                Some(symbol) => {
                    let _ = match symbol {
                        Symbol::AbsType(ty) => self.visit_def_abs_type(a, ty),
                        Symbol::AliasType(ty) => self.visit_def_alias_type(a, ty),
                        Symbol::Array(ty) => self.visit_def_array(a, ty),
                        Symbol::Enum(ty) => self.visit_def_enum(a, ty),
                        Symbol::Struct(ty) => self.visit_def_struct(a, ty),
                        _ => ControlFlow::Continue(()),
                    };
                }
            },
            TypeNameKind::String(size) => match self.expr_as_integer_opt(a, size) {
                None => {}
                Some(size_v) => {
                    // TODO(tumbar) Should we disallow 0 size strings to be inline with Scala FPP
                    //    See https://github.com/nasa/fpp/issues/878
                    if size_v < 0 {
                        SemanticError::InvalidIntValue {
                            loc: size.as_ref().unwrap().span(),
                            v: Some(size_v),
                            msg: "negative string sizes are not allowed".to_string(),
                        }
                        .emit();
                    } else if size_v >= 1 << 31 {
                        SemanticError::InvalidIntValue {
                            loc: size.as_ref().unwrap().span(),
                            v: Some(size_v),
                            msg: "string size must in range [0, 2^31)".to_string(),
                        }
                        .emit();
                    } else {
                        a.type_map
                            .insert(node.node_id, Rc::new(Type::String(Some(size_v))));
                    }
                }
            },
            _ => {}
        }

        match a.type_map.get(&node.node_id) {
            None => None,
            Some(v) => Some(v.clone()),
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

    fn visit_def_alias_type(
        &self,
        a: &mut Self::State,
        node: &'ast DefAliasType,
    ) -> ControlFlow<Self::Break> {
        let symbol = Symbol::AliasType(node);
        if a.visited_symbol_set.contains(&symbol) {
            return ControlFlow::Continue(());
        }

        a.visited_symbol_set.insert(symbol);
        match self.ty(a, &node.type_name) {
            None => {}
            Some(ty) => {
                a.type_map.insert(
                    node.node_id,
                    Rc::new(Type::AliasType(AliasType {
                        node: node.clone(),
                        alias_type: ty,
                    })),
                );
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_def_array(
        &self,
        a: &mut Self::State,
        node: &'ast DefArray,
    ) -> ControlFlow<Self::Break> {
        let symbol = Symbol::Array(node);
        if a.visited_symbol_set.contains(&symbol) {
            return ControlFlow::Continue(());
        }

        a.visited_symbol_set.insert(symbol);
        let elt_type = match self.ty(a, &node.elt_type) {
            None => return ControlFlow::Continue(()),
            Some(elt_type) => elt_type,
        };

        let size = match self.expr_as_integer(a, &node.size) {
            None => return ControlFlow::Continue(()),
            Some(size) => size,
        };

        if size <= 0 {
            SemanticError::InvalidIntValue {
                loc: node.size.span(),
                v: Some(size),
                msg: "array size must be greater than zero".to_string(),
            }
            .emit();
            return ControlFlow::Continue(());
        }

        let anon_array = AnonArrayType {
            size: Some(size as usize),
            elt_type: elt_type.clone(),
        };

        let anon_array_ty = Type::AnonArray(anon_array.clone());

        // Compute the default value
        let default = match &node.default {
            None => anon_array_ty.default_value(),
            Some(default) => match a.value_map.get(&default.node_id) {
                None => None,
                Some(default) => default.convert(&Rc::new(anon_array_ty)),
            },
        };

        // Compute the format
        let format = match &node.format {
            None => None,
            Some(format) => Some(Format::new(
                format,
                vec![(elt_type.clone(), node.elt_type.span().clone())],
            )),
        };

        let ty = Type::Array(ArrayType {
            node: node.clone(),
            anon_array,
            default,
            format,
        });

        a.type_map.insert(node.node_id, Rc::new(ty));
        ControlFlow::Continue(())
    }
}
