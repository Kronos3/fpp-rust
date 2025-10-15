use crate::Node;
use fpp_core::span::Span;
use crate::common::{Annotated, Ident, NodeList, QualIdent, TypeName};

#[derive(fpp_derive::Ast)]
pub struct DefStateMachine {
    name: Ident,
    members: NodeList<StateMachineMember>,
}

#[derive(fpp_derive::Ast)]
pub enum StateMachineMember {
    DefAction(DefAction),
    DefChoice(DefChoice),
    DefGuard(DefGuard),
    DefSignal(DefSignal),
    DefState(DefState),
    SpecInitialTransition(SpecInitialTransition),
}

/** Action definition */
#[derive(fpp_derive::Ast)]
pub struct DefAction {
    name: Ident,
    type_name: Option<TypeName>,
}

/** Choice definition */
#[derive(fpp_derive::Ast)]
pub struct DefChoice {
    name: Ident,
    guard: Ident,
    if_transition: TransitionExpr,
    else_transition: TransitionExpr,
}

/** Guard definition */
#[derive(fpp_derive::Ast)]
pub struct DefGuard {
    name: Ident,
    type_name: Option<TypeName>,
}

/** Transition expression */
#[derive(fpp_derive::Ast)]
pub struct TransitionExpr {
    actions: NodeList<Ident>,
    target: QualIdent,
}

/** Signal definition */
#[derive(fpp_derive::Ast)]
pub struct DefSignal {
    name: Ident,
    type_name: Option<TypeName>,
}

#[derive(fpp_derive::Ast)]
pub struct DefState {
    name: Ident,
    members: NodeList<Annotated<StateMember>>,
}

#[derive(fpp_derive::Ast)]
pub enum StateMember {
    DefChoice(DefChoice),
    DefState(DefState),
    SpecInitialTransition(SpecInitialTransition),

}

/** Initial state specifier */
#[derive(fpp_derive::Ast)]
pub struct SpecInitialTransition {
    transition: TransitionExpr
}

/** State entry specifier */
#[derive(fpp_derive::Ast)]
pub struct SpecStateEntry {
    actions: NodeList<Ident>
}

/** State exit specifier */
#[derive(fpp_derive::Ast)]
pub struct SpecStateExit{
    actions: NodeList<Ident>
}

/** Transition specifier */
#[derive(fpp_derive::Ast)]
pub struct SpecStateTransition{
    signal: Ident,
    guard: Option<Ident>,
    transition_or_do: TransitionOrDo,
}

/** Transition or do within transition specifier */
#[derive(fpp_derive::Ast)]
pub enum TransitionOrDo {
    Transition(TransitionExpr),
    Do(NodeList<Ident>)
}
