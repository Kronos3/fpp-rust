use std::ops::{ControlFlow, Deref, DerefMut};

macro_rules! visit_signature {
    ($ty:ident, $visitor:ident) => {
        fn $visitor (&mut self, node: &'a crate::$ty) -> ControlFlow<Self::Break> {
            crate::Walkable::walk_ref(node, self)
        }
    };
}

macro_rules! visit_signature_mut {
    ($ty:ident, $visitor:ident) => {
        fn $visitor (&mut self, node: &mut crate::$ty) -> ControlFlow<Self::Break> {
            crate::MutWalkable::walk_mut(node, self)
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

/// Each method of the `Visitor` trait is a hook to be potentially
/// overridden. Each method's default implementation recursively visits
/// the substructure of the input via the corresponding `walk` method;
/// e.g., the `visit_item` method by default calls `visit::walk_item`.
///
/// If you want to ensure that your code handles every variant
/// explicitly, you need to override each method. (And you also need
/// to monitor future changes to `Visitor` in case a new method with a
/// new default implementation gets introduced.)
pub trait Visitor<'a>: Sized {
    type Break;
    const DEFAULT: ControlFlow<Self::Break> = ControlFlow::Continue(());

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

pub trait MutVisitor: Sized {
    type Break;
    const DEFAULT: ControlFlow<Self::Break> = ControlFlow::Continue(());

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
    ///
    /// returns: ControlFlow<<V as Visitor>::Break, ()>
    fn walk_ref(&'a self, visitor: &mut V) -> ControlFlow<V::Break>;
}

pub trait MutWalkable<V: MutVisitor> {
    fn walk_mut(&mut self, visitor: &mut V) -> ControlFlow<V::Break>;
}

pub trait Visitable<'a, V: Visitor<'a>> {
    fn visit(&'a self, visitor: &mut V) -> ControlFlow<V::Break>;
}

pub trait MutVisitable<V: MutVisitor> {
    fn visit(&mut self, visitor: &mut V) -> ControlFlow<V::Break>;
}

/// Visitable for Option<T>

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Visitable<'a, V> for Option<T> {
    fn visit(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            None => ControlFlow::Continue(()),
            Some(s) => s.visit(visitor),
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
    fn walk_ref(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.visit(visitor)
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutWalkable<V> for Box<T> {
    fn walk_mut(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.visit(visitor)
    }
}

/// Visitable for Box<T>

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Visitable<'a, V> for Box<T> {
    fn visit(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.deref().visit(visitor)
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutVisitable<V> for Box<T> {
    fn visit(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.deref_mut().visit(visitor)
    }
}

/// Walkable for Vec<T> by visiting all the children

impl<'a, V: Visitor<'a>, T: Visitable<'a, V>> Walkable<'a, V> for Vec<T> {
    fn walk_ref(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        for child in self {
            child.visit(visitor)?;
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
    fn visit(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.walk_ref(visitor)
    }
}

impl<V: MutVisitor, T: MutVisitable<V>> MutVisitable<V> for Vec<T> {
    fn visit(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.walk_mut(visitor)
    }
}
