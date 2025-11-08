use crate::Node;
use std::ops::{ControlFlow, Deref, DerefMut};

macro_rules! visit_signature {
    ($ty:ident, $visitor:ident) => {
        fn $visitor(&self, a: &mut Self::State, node: &'ast crate::$ty) -> ControlFlow<Self::Break> {
            self.visit(a, Node::$ty(node))
        }
    };
}

macro_rules! visit_signature_mut {
    ($ty:ident, $visitor:ident) => {
        fn $visitor(&mut self, node: &mut crate::$ty) -> ControlFlow<Self::Break> {
            self.visit(node)
        }
    };
}

macro_rules! visit_signatures {
    ($( ( $ty:ident, $visitor:ident ), )*) => {
        $(
            visit_signature!($ty, $visitor);
        )*
    };
}

macro_rules! visit_signatures_mut {
    ($( ( $ty:ident, $visitor:ident ), )*) => {
        $(
            visit_signature_mut!($ty, $visitor);
        )*
    };
}

/// This trait outlines a standard Visitor pattern for FPP.
/// It includes a function signature for each of the 'interesting' nodes in the AST.
/// It also provides generic mechanisms for visiting the AST.
///
/// [Visitor] is the non-mutable variant of the visitor pattern. It takes the ast's
/// lifetime as a parameter which allows implementations to pass out references to nodes.
///
/// If you need to modify the AST, you'll need to use the mutable [MutVisitor] variant instead.
///
/// # Shallow traversal
///
/// Shallow traversal is the most common pattern you'll see compiler passes in FPP
/// implement. They only walk child leaves in the AST when explicitly told to. The default
/// implementation of [Visitor::visit] is a shallow traversal.
///
/// Here is an example of a shallow pass visiting all the members of a component definition:
/// > Note: Member function implementations were omitted.
/// ```
/// use std::ops::ControlFlow;
/// use fpp_ast::{DefComponent, DefModule, Visitor, Walkable};
///
/// struct ComponentPass {}
/// impl<'ast> Visitor<'ast> for ComponentPass {
///     type Break = ();
///
///     fn visit_def_component(&mut self, def: &'ast DefComponent, extra: ()) -> ControlFlow<Self::Break> {
///         def.walk_ref(self, extra)
///     }
///
///     fn visit_def_module(&mut self, def: &'ast DefModule, extra: ()) -> ControlFlow<Self::Break> {
///         def.walk_ref(self, extra)
///     }
/// }
/// ```
///
/// Notice how both [Visitor::visit_def_module] and [Visitor::visit_def_component] were both
/// implemented to recursive through the nodes.
///
/// ## Deep traversal
///
/// Deep traversal is the inverse of shallow traverse. It walks all child nodes in the AST unless
/// explicitly told not to. This is useful if you need to implement a pass that hits the majority
/// of the AST such as [Visitor::visit_expr] or [Visitor::visit_type_name].
///
/// Here is an example of a deep traversal:
/// ```
/// use std::ops::ControlFlow;
/// use fpp_ast::{Expr, Visitor, Walkable};
///
/// struct ExprPass {}
/// impl<'ast> Visitor<'ast> for ExprPass {
///     type Break = ();
///
///     fn visit<V: Walkable<'ast, Self>>(&mut self, node: &'ast V) -> ControlFlow<Self::Break> {
///         node.walk_ref(self, extra)
///     }
///
///     fn visit_expr(&mut self, node: &'ast Expr, extra: ()) -> ControlFlow<Self::Break> {
///         // Run on all the expressions in the entire AST
///         ControlFlow::Continue(())
///     }
/// }
/// ```
pub trait Visitor<'ast>: Sized {
    type Break;
    type State;

    /// The default node visiting before.
    /// By default, this will just continue without visiting the children of `node`
    fn visit(&self, a: &mut Self::State, node: Node<'ast>) -> ControlFlow<Self::Break> {
        let _ = node;
        let _ = a;
        ControlFlow::Continue(())
    }

    visit_signatures!(
        /* Definitions */
        (DefAbsType, visit_def_abs_type),
        (DefAction, visit_def_action),
        (DefAliasType, visit_def_alias_type),
        (DefArray, visit_def_array),
        (DefChoice, visit_def_choice),
        (DefComponent, visit_def_component),
        (DefComponentInstance, visit_def_component_instance),
        (DefConstant, visit_def_constant),
        (DefEnum, visit_def_enum),
        (DefEnumConstant, visit_def_enum_constant),
        (DefGuard, visit_def_guard),
        (DefInterface, visit_def_interface),
        (DefModule, visit_def_module),
        (DefPort, visit_def_port),
        (DefSignal, visit_def_signal),
        (DefState, visit_def_state),
        (DefStateMachine, visit_def_state_machine),
        (DefStruct, visit_def_struct),
        (DefTopology, visit_def_topology),
        /* Specifiers */
        (SpecCommand, visit_spec_command),
        (SpecConnectionGraph, visit_spec_connection_graph),
        (SpecContainer, visit_spec_container),
        (SpecEvent, visit_spec_event),
        (SpecGeneralPortInstance, visit_spec_general_port_instance),
        (SpecImport, visit_spec_import),
        (SpecInclude, visit_spec_include),
        (SpecInit, visit_spec_init),
        (SpecInitialTransition, visit_spec_initial_transition),
        (SpecInstance, visit_spec_instance),
        (SpecInternalPort, visit_spec_internal_port),
        (SpecLoc, visit_spec_loc),
        (SpecParam, visit_spec_param),
        (SpecPortInstance, visit_spec_port_instance),
        (SpecPortMatching, visit_spec_port_matching),
        (SpecRecord, visit_spec_record),
        (SpecSpecialPortInstance, visit_spec_special_port_instance),
        (SpecStateEntry, visit_spec_state_entry),
        (SpecStateExit, visit_spec_state_exit),
        (SpecStateMachineInstance, visit_spec_state_machine_instance),
        (SpecStateTransition, visit_spec_state_transition),
        (SpecTlmChannel, visit_spec_tlm_channel),
        (SpecTlmPacket, visit_spec_tlm_packet),
        (SpecTlmPacketSet, visit_spec_tlm_packet_set),
        (SpecTopPort, visit_spec_top_port),
        /* Other AST nodes */
        (Expr, visit_expr),
        (FormalParam, visit_formal_param),
        (Ident, visit_ident),
        (LitString, visit_lit_string),
        (QualIdent, visit_qual_ident),
        (Qualified, visit_qualified),
        (StructMember, visit_struct_member),
        (TypeName, visit_type_name),
        (TypeNameKind, visit_type_name_kind),
        /* Inner AST nodes */
        (Connection, visit_connection),
        (DoExpr, visit_do_expr),
        (EventThrottle, visit_event_throttle),
        (PortInstanceIdentifier, visit_port_instance_identifier),
        (StructTypeMember, visit_struct_type_member),
        (TlmChannelIdentifier, visit_tlm_channel_identifier),
        (TlmChannelLimit, visit_tlm_channel_limit),
        (TransitionExpr, visit_transition_expr),
    );
}

/// This is the mutable variant of [Visitor]. It allows making changes to the AST
/// during traversal.
///
/// Notice that this visitor does not take a lifetime life [Visitor].
/// This is because we cannot pass out mutable references to nodes in the AST that live
/// past the execution of the visitor.
///
/// Generally these visitors should not collect any of the elements AST but rather purely
/// modify the nodes it needs to.
///
/// For more information of implementing a visitor, see [Visitor]
pub trait MutVisitor: Sized {
    type Break;

    /// The default node visiting before.
    /// By default, this will just continue without visiting the children of `node`
    fn visit<V: MutWalkable<Self>>(&mut self, node: &V) -> ControlFlow<Self::Break> {
        let _ = node;
        ControlFlow::Continue(())
    }

    visit_signatures_mut!(
        /* Definitions */
        (DefAbsType, visit_def_abs_type),
        (DefAction, visit_def_action),
        (DefAliasType, visit_def_alias_type),
        (DefArray, visit_def_array),
        (DefChoice, visit_def_choice),
        (DefComponent, visit_def_component),
        (DefComponentInstance, visit_def_component_instance),
        (DefConstant, visit_def_constant),
        (DefEnum, visit_def_enum),
        (DefEnumConstant, visit_def_enum_constant),
        (DefGuard, visit_def_guard),
        (DefInterface, visit_def_interface),
        (DefModule, visit_def_module),
        (DefPort, visit_def_port),
        (DefSignal, visit_def_signal),
        (DefState, visit_def_state),
        (DefStateMachine, visit_def_state_machine),
        (DefStruct, visit_def_struct),
        (DefTopology, visit_def_topology),
        /* Specifiers */
        (SpecCommand, visit_spec_command),
        (SpecConnectionGraph, visit_spec_connection_graph),
        (SpecContainer, visit_spec_container),
        (SpecEvent, visit_spec_event),
        (SpecGeneralPortInstance, visit_spec_general_port_instance),
        (SpecImport, visit_spec_import),
        (SpecInclude, visit_spec_include),
        (SpecInit, visit_spec_init),
        (SpecInitialTransition, visit_spec_initial_transition),
        (SpecInstance, visit_spec_instance),
        (SpecInternalPort, visit_spec_internal_port),
        (SpecLoc, visit_spec_loc),
        (SpecParam, visit_spec_param),
        (SpecPortInstance, visit_spec_port_instance),
        (SpecPortMatching, visit_spec_port_matching),
        (SpecRecord, visit_spec_record),
        (SpecSpecialPortInstance, visit_spec_special_port_instance),
        (SpecStateEntry, visit_spec_state_entry),
        (SpecStateExit, visit_spec_state_exit),
        (SpecStateMachineInstance, visit_spec_state_machine_instance),
        (SpecStateTransition, visit_spec_state_transition),
        (SpecTlmChannel, visit_spec_tlm_channel),
        (SpecTlmPacket, visit_spec_tlm_packet),
        (SpecTlmPacketSet, visit_spec_tlm_packet_set),
        (SpecTopPort, visit_spec_top_port),
        /* Other AST nodes */
        (Expr, visit_expr),
        (FormalParam, visit_formal_param),
        (Ident, visit_ident),
        (LitString, visit_lit_string),
        (QualIdent, visit_qual_ident),
        (Qualified, visit_qualified),
        (StructMember, visit_struct_member),
        (TypeName, visit_type_name),
        (TypeNameKind, visit_type_name_kind),
        /* Inner AST nodes */
        (Connection, visit_connection),
        (DoExpr, visit_do_expr),
        (EventThrottle, visit_event_throttle),
        (PortInstanceIdentifier, visit_port_instance_identifier),
        (StructTypeMember, visit_struct_type_member),
        (TlmChannelIdentifier, visit_tlm_channel_identifier),
        (TlmChannelLimit, visit_tlm_channel_limit),
        (TransitionExpr, visit_transition_expr),
    );
}

/// Trait to wrap all the nodes that have a structure to them
/// The walking should recursively call out to its child nodes
/// [Visitable::visit] delegator which in turn calls out to its own
/// [Visitor::visit_*].
///
/// Walkable is a trait that is automatically implemented by the [Walkable]
/// derive macro. This macro also implements the [Visitable] trait for that type
/// so that it may itself be visited.
pub trait Walkable<'a, V: Visitor<'a>> {
    /// Walk all the child nodes of this node
    ///
    /// # Arguments
    ///
    /// * `visitor`: the visitor that should be called into when visiting nodes
    /// * `scope`: the scope built up while visiting nodes recursively
    ///
    /// returns: ControlFlow<<V as Visitor>::Break, ()>
    fn walk_ref(&'a self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break>;
}

pub trait MoveWalkable<'a, V: Visitor<'a>> {
    fn walk(self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break>;
}

pub trait MutWalkable<V: MutVisitor> {
    fn walk_mut(&mut self, visitor: &mut V) -> ControlFlow<V::Break>;
}

pub trait Visitable<'a, V: Visitor<'a>> {
    fn visit(&'a self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break>;
}

pub trait MutVisitable<V: MutVisitor> {
    fn visit(&mut self, visitor: &mut V) -> ControlFlow<V::Break>;
}

/// Visitable for Option<T>

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Visitable<'a, V> for Option<T> {
    fn visit(&'a self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break> {
        match self {
            None => ControlFlow::Continue(()),
            Some(s) => s.visit(a, visitor),
        }
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutVisitable<V> for Option<T> {
    fn visit(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            None => ControlFlow::Continue(()),
            Some(s) => s.visit(visitor),
        }
    }
}

/// Walkable for Box<T> by visiting the inner object

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Walkable<'a, V> for Box<T> {
    fn walk_ref(&'a self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break> {
        self.visit(a, visitor)
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutWalkable<V> for Box<T> {
    fn walk_mut(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.visit(visitor)
    }
}

/// Visitable for Box<T>

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Visitable<'a, V> for Box<T> {
    fn visit(&'a self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break> {
        self.deref().visit(a, visitor)
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutVisitable<V> for Box<T> {
    fn visit(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.deref_mut().visit(visitor)
    }
}

/// Walkable for Vec<T> by visiting all the children

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Walkable<'a, V> for Vec<T> {
    fn walk_ref(&'a self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break> {
        for child in self {
            child.visit(a, visitor)?;
        }

        ControlFlow::Continue(())
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutWalkable<V> for Vec<T> {
    fn walk_mut(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        for child in self {
            child.visit(visitor)?;
        }

        ControlFlow::Continue(())
    }
}

/// Visitable for Vec<T> by walking all the elements in the vec

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Visitable<'a, V> for Vec<T> {
    fn visit(&'a self, a: &mut V::State, visitor: &V) -> ControlFlow<V::Break> {
        self.walk_ref(a, visitor)
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutVisitable<V> for Vec<T> {
    fn visit(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.walk_mut(visitor)
    }
}
