use std::ops::{ControlFlow, Deref, DerefMut};

macro_rules! visit_signature {
    ($(($mut:ident))? $ty:ident, $visitor:ident) => {
        fn $visitor (&mut self, _: & $($mut)? crate::$ty) -> ControlFlow<Self::Break> {
            // ControlFlow::Continue(())
            Self::DEFAULT
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
            visit_signature!((mut) $ty, $visitor);
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

pub trait MutVisitor<'a>: Sized {
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

pub trait Walkable<'a, V: Visitor<'a>> {
    fn walk_ref(&'a self, visitor: &mut V) -> ControlFlow<V::Break>;
}

pub trait MutWalkable<'a, V: MutVisitor<'a>> {
    fn walk_mut(&'a mut self, visitor: &mut V) -> ControlFlow<V::Break>;
}

/// Walkable for Option<T>

impl<'a, V: Visitor<'a>, T: Walkable<'a, V>> Walkable<'a, V> for Option<T> {
    fn walk_ref(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            None => ControlFlow::Continue(()),
            Some(s) => s.walk_ref(visitor),
        }
    }
}

impl<'a, V: MutVisitor<'a>, T: MutWalkable<'a, V>> MutWalkable<'a, V> for Option<T> {
    fn walk_mut(&'a mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        match self {
            None => ControlFlow::Continue(()),
            Some(s) => s.walk_mut(visitor),
        }
    }
}

/// Walkable for Box<T>

impl<'a, V: Visitor<'a>, T: Walkable<'a, V>> Walkable<'a, V> for Box<T> {
    fn walk_ref(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.deref().walk_ref(visitor)
    }
}

impl<'a, V: MutVisitor<'a>, T: MutWalkable<'a, V>> MutWalkable<'a, V> for Box<T> {
    fn walk_mut(&'a mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        self.deref_mut().walk_mut(visitor)
    }
}

/// Walkable for Vec<T>

impl<'a, V: Visitor<'a>, T: Walkable<'a, V>> Walkable<'a, V> for Vec<T> {
    fn walk_ref(&'a self, visitor: &mut V) -> ControlFlow<V::Break> {
        for child in self {
            let _ = child.walk_ref(visitor);
        }

        ControlFlow::Continue(())
    }
}

impl<'a, V: MutVisitor<'a>, T: MutWalkable<'a, V>> MutWalkable<'a, V> for Vec<T> {
    fn walk_mut(&'a mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        for child in self {
            let _ = child.walk_mut(visitor);
        }

        ControlFlow::Continue(())
    }
}
