use crate::analyzers::analyzer::Analyzer;
use crate::analyzers::basic_use_analyzer::UseAnalysisPass;
use crate::analyzers::use_analyzer::UseAnalyzer;
use crate::errors::SemanticError;
use crate::semantics::{
    AnonArrayType, AnonStructType, ArrayType, QualifiedName, StructType, Symbol, SymbolInterface,
    Type, TypeConversionResult,
};
use crate::Analysis;
use fpp_ast::{
    DefAliasType, DefArray, DefConstant, DefEnum, DefEnumConstant, DefStruct, Expr, ExprKind,
    FloatKind, IntegerKind, Node, SpecCommand, SpecContainer, SpecEvent, SpecGeneralPortInstance,
    SpecInit, SpecInternalPort, SpecParam, SpecRecord, SpecSpecialPortInstance,
    SpecStateMachineInstance, SpecTlmChannel, SpecTlmPacket, StructTypeMember, TypeName,
    TypeNameKind, Visitable, Visitor,
};
use fpp_core::Spanned;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{ControlFlow, Deref};
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

    fn check_type_is_numerical_opt(&self, a: &Analysis<'ast>, expr: &'ast Option<Expr>) {
        match expr {
            None => {}
            Some(expr) => self.check_type_is_numerical(a, expr),
        }
    }

    fn check_expr_matches_node(&self, a: &Analysis<'ast>, expr: &'ast Expr, node: &fpp_core::Node) {
        match a.type_map.get(node) {
            Some(ty) => match self.convert_type(a, expr, ty) {
                Ok(_) => {}
                Err(err) => SemanticError::TypeConversion {
                    loc: expr.span(),
                    msg: format!("default value cannot be converted to {}", ty.borrow()),
                    err,
                }
                .emit(),
            },
            None => {}
        }
    }

    fn check_expr_opt_matches_node(
        &self,
        a: &Analysis<'ast>,
        expr: &'ast Option<Expr>,
        node: &fpp_core::Node,
    ) {
        match &expr {
            Some(expr) => self.check_expr_matches_node(a, expr, node),
            _ => {}
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
        self.check_expr_opt_matches_node(a, &node.default, &node.node_id);

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
        self.check_expr_opt_matches_node(a, &node.default, &node.node_id);

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
        match &node.value {
            None => {}
            Some(value) => self.check_type_is_numerical(a, value),
        }

        ControlFlow::Continue(())
    }

    fn visit_def_struct(
        &self,
        a: &mut Self::State,
        node: &'ast DefStruct,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::DefStruct(node))?;
        self.check_expr_opt_matches_node(a, &node.default, &node.node_id);

        ControlFlow::Continue(())
    }

    fn visit_expr(&self, a: &mut Self::State, node: &'ast Expr) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::Expr(node))?;

        match &node.kind {
            ExprKind::Array(array) => {
                let first_node = match array.first() {
                    None => {
                        SemanticError::EmptyArray { loc: node.span() }.emit();
                        return ControlFlow::Continue(());
                    }
                    Some(first) => first,
                };

                let first_ty = match a.type_map.get(&first_node.node_id) {
                    Some(first_ty) => first_ty.clone(),
                    // Node doesn't have a type
                    None => return ControlFlow::Continue(()),
                };

                let common = array.iter().skip(1).fold(Ok(first_ty), |common, next| {
                    match (common, a.type_map.get(&next.node_id)) {
                        (Ok(common), Some(next_type)) => {
                            match Type::common_type(&common, next_type) {
                                None => Err(SemanticError::InvalidType {
                                    loc: next.span(),
                                    msg: format!(
                                        "cannot find common type between {} and {}",
                                        next_type.borrow(),
                                        common.borrow()
                                    ),
                                }),
                                Some(new_ty) => Ok(new_ty),
                            }
                        }
                        (Ok(common), None) => {
                            // Node does not have a type
                            Ok(common)
                        }
                        (Err(err), _) => Err(err),
                    }
                });

                match common {
                    Ok(common) => {
                        // A common type was found across all the elements
                        // Assign the full expression to an array of the proper size
                        a.type_map.insert(
                            node.node_id,
                            Rc::new(RefCell::new(Type::AnonArray(AnonArrayType {
                                elt_type: common,
                                size: Some(array.len()),
                            }))),
                        );
                    }
                    Err(err) => err.emit(),
                }
            }
            ExprKind::ArraySubscript { e1, e2 } => {
                self.check_type_is_numerical(a, e2);

                let arr_ty = match a.type_map.get(&e1.node_id) {
                    None => return ControlFlow::Continue(()),
                    Some(arr_ty) => Type::underlying_type(arr_ty),
                };

                let elt_ty = match arr_ty.borrow().deref() {
                    Type::Array(ArrayType { anon_array, .. }) | Type::AnonArray(anon_array) => {
                        anon_array.elt_type.clone()
                    }
                    ty => {
                        SemanticError::InvalidType {
                            loc: e1.span(),
                            msg: format!("{} is not an array type", ty),
                        }
                        .emit();
                        return ControlFlow::Continue(());
                    }
                };

                a.type_map.insert(node.node_id, elt_ty);
            }
            ExprKind::Binop { left, right, .. } => {
                let ty = match (
                    a.type_map.get(&left.node_id),
                    a.type_map.get(&right.node_id),
                ) {
                    (Some(lty), Some(rty)) => match Type::common_type(lty, rty) {
                        None => {
                            SemanticError::InvalidType {
                                loc: node.span(),
                                msg: format!(
                                    "invalid binary operation between {} and {}",
                                    lty.borrow(),
                                    rty.borrow()
                                ),
                            }
                            .emit();
                            return ControlFlow::Continue(());
                        }
                        Some(ty) => ty,
                    },
                    _ => {
                        // One of the sides of the binary operation does not have a type
                        return ControlFlow::Continue(());
                    }
                };

                // TODO(tumbar) Do I need to check if l/r are numeric?
                a.type_map.insert(node.node_id, ty);
            }
            ExprKind::Dot { e, id } => {
                if a.type_map.get(&node.node_id).is_some() {
                    // Type for this entire dot expression is already resolved
                    // This means it's actually a _use_ of some sort of symbol (constant/enum constant)
                    // We can just use the type that was already resolved
                    return ControlFlow::Continue(());
                }

                // This is some member selection of a struct/anon struct
                let e_ty = match a.type_map.get(&e.node_id) {
                    Some(e_ty) => Type::underlying_type(e_ty),
                    None => return ControlFlow::Continue(()),
                };

                match e_ty.borrow().deref() {
                    Type::AnonStruct(anon_struct)
                    | Type::Struct(StructType { anon_struct, .. }) => {
                        match anon_struct.members.get(&id.data) {
                            None => SemanticError::InvalidType {
                                loc: id.span(),
                                msg: format!("{} has no member `{}`", e_ty.borrow(), id.data),
                            }
                            .emit(),
                            Some(member_ty) => {
                                a.type_map.insert(node.node_id, member_ty.clone());
                            }
                        }
                    }
                    ty => SemanticError::InvalidType {
                        loc: e.span(),
                        msg: format!("{} does not have members", ty),
                    }
                    .emit(),
                }
            }
            ExprKind::Ident(_) => {} // already handled by constantUse
            ExprKind::LiteralBool(_) => {
                a.type_map
                    .insert(node.node_id, Rc::new(RefCell::new(Type::Boolean)));
            }
            ExprKind::LiteralInt(_) => {
                a.type_map
                    .insert(node.node_id, Rc::new(RefCell::new(Type::Integer)));
            }
            ExprKind::LiteralFloat(_) => {
                a.type_map.insert(
                    node.node_id,
                    Rc::new(RefCell::new(Type::Float(FloatKind::F64))),
                );
            }
            ExprKind::LiteralString(_) => {
                a.type_map
                    .insert(node.node_id, Rc::new(RefCell::new(Type::String(None))));
            }
            ExprKind::Paren(e) => match a.type_map.get(&e.node_id) {
                Some(ty) => {
                    a.type_map.insert(node.node_id, ty.clone());
                }
                _ => {}
            },
            ExprKind::Struct(struct_expr) => {
                let mut members_out = HashMap::new();
                for member in struct_expr {
                    match a.type_map.get(&member.value.node_id) {
                        Some(member_ty) => {
                            members_out.insert(member.name.data.clone(), member_ty.clone());
                        }
                        _ => return ControlFlow::Continue(()),
                    }
                }

                a.type_map.insert(
                    node.node_id,
                    Rc::new(RefCell::new(Type::AnonStruct(AnonStructType {
                        members: members_out,
                    }))),
                );
            }
            ExprKind::Unop { e, .. } => {
                // TODO(tumbar) Do I need to check if e is numeric?
                match a.type_map.get(&e.node_id) {
                    Some(ty) => {
                        a.type_map.insert(node.node_id, ty.clone());
                    }
                    _ => {}
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_spec_command(
        &self,
        a: &mut Self::State,
        node: &'ast SpecCommand,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecCommand(node))?;
        self.check_type_is_numerical_opt(a, &node.opcode);
        self.check_type_is_numerical_opt(a, &node.priority);

        ControlFlow::Continue(())
    }

    fn visit_spec_container(
        &self,
        a: &mut Self::State,
        node: &'ast SpecContainer,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecContainer(node))?;
        self.check_type_is_numerical_opt(a, &node.id);
        self.check_type_is_numerical_opt(a, &node.default_priority);

        ControlFlow::Continue(())
    }

    fn visit_spec_event(
        &self,
        a: &mut Self::State,
        node: &'ast SpecEvent,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecEvent(node))?;
        self.check_type_is_numerical_opt(a, &node.id);

        // Check that the throttle count/every are valid
        match &node.throttle {
            None => {}
            Some(throttle) => {
                self.check_type_is_numerical(a, &throttle.count);
                match &throttle.every {
                    Some(every) => {
                        match self.convert_type(
                            a,
                            every,
                            &Rc::new(RefCell::new(Type::AnonStruct(AnonStructType {
                                members: HashMap::from([
                                    (
                                        "seconds".to_string(),
                                        Rc::new(RefCell::new(Type::PrimitiveInt(IntegerKind::U32))),
                                    ),
                                    (
                                        "useconds".to_string(),
                                        Rc::new(RefCell::new(Type::PrimitiveInt(IntegerKind::U32))),
                                    ),
                                ]),
                            }))),
                        ) {
                            Err(err) => SemanticError::TypeConversion {
                                loc: every.span(),
                                msg: "event throttle every must be convertable to a time interval"
                                    .to_string(),
                                err,
                            }
                            .emit(),
                            Ok(()) => {}
                        }
                    }
                    None => {}
                }
            }
        }

        ControlFlow::Continue(())
    }

    fn visit_spec_init(
        &self,
        a: &mut Self::State,
        node: &'ast SpecInit,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecInit(node))?;
        self.check_type_is_numerical(a, &node.phase);
        ControlFlow::Continue(())
    }

    fn visit_spec_internal_port(
        &self,
        a: &mut Self::State,
        node: &'ast SpecInternalPort,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecInternalPort(node))?;
        self.check_type_is_numerical_opt(a, &node.priority);
        ControlFlow::Continue(())
    }

    fn visit_spec_param(
        &self,
        a: &mut Self::State,
        node: &'ast SpecParam,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecParam(node))?;

        self.check_expr_opt_matches_node(a, &node.default, &node.type_name.node_id);
        self.check_type_is_numerical_opt(a, &node.id);
        self.check_type_is_numerical_opt(a, &node.set_opcode);
        self.check_type_is_numerical_opt(a, &node.save_opcode);

        ControlFlow::Continue(())
    }

    fn visit_spec_general_port_instance(
        &self,
        a: &mut Self::State,
        node: &'ast SpecGeneralPortInstance,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecGeneralPortInstance(node))?;

        self.check_type_is_numerical_opt(a, &node.size);
        self.check_type_is_numerical_opt(a, &node.priority);

        ControlFlow::Continue(())
    }

    fn visit_spec_special_port_instance(
        &self,
        a: &mut Self::State,
        node: &'ast SpecSpecialPortInstance,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecSpecialPortInstance(node))?;
        self.check_type_is_numerical_opt(a, &node.priority);
        ControlFlow::Continue(())
    }

    fn visit_spec_record(
        &self,
        a: &mut Self::State,
        node: &'ast SpecRecord,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecRecord(node))?;
        self.check_type_is_numerical_opt(a, &node.id);
        ControlFlow::Continue(())
    }

    fn visit_spec_state_machine_instance(
        &self,
        a: &mut Self::State,
        node: &'ast SpecStateMachineInstance,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecStateMachineInstance(node))?;
        self.check_type_is_numerical_opt(a, &node.priority);
        ControlFlow::Continue(())
    }

    fn visit_spec_tlm_channel(
        &self,
        a: &mut Self::State,
        node: &'ast SpecTlmChannel,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecTlmChannel(node))?;
        self.check_type_is_numerical_opt(a, &node.id);
        for limit in &node.low {
            self.check_type_is_numerical(a, &limit.value);
            self.check_expr_matches_node(a, &limit.value, &node.type_name.node_id)
        }

        for limit in &node.high {
            self.check_type_is_numerical(a, &limit.value);
            self.check_expr_matches_node(a, &limit.value, &node.type_name.node_id)
        }

        ControlFlow::Continue(())
    }

    fn visit_spec_tlm_packet(
        &self,
        a: &mut Self::State,
        node: &'ast SpecTlmPacket,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::SpecTlmPacket(node))?;
        self.check_type_is_numerical_opt(a, &node.id);
        self.check_type_is_numerical(a, &node.group);
        ControlFlow::Continue(())
    }

    fn visit_struct_type_member(
        &self,
        a: &mut Self::State,
        node: &'ast StructTypeMember,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::StructTypeMember(node))?;
        self.check_type_is_numerical_opt(a, &node.size);
        ControlFlow::Continue(())
    }

    fn visit_type_name(
        &self,
        a: &mut Self::State,
        node: &'ast TypeName,
    ) -> ControlFlow<Self::Break> {
        self.super_visit(a, Node::TypeName(node))?;
        match &node.kind {
            TypeNameKind::String(size) => {
                self.check_type_is_numerical_opt(a, size);
            }
            _ => {}
        }

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
