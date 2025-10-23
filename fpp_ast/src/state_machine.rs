use crate::common::{Annotated, Ident, NodeList, QualIdent, TypeName};

pub struct DefStateMachine {
    name: Ident,
    members: NodeList<StateMachineMember>,
}

pub enum StateMachineMember {
    DefAction(DefAction),
    DefChoice(DefChoice),
    DefGuard(DefGuard),
    DefSignal(DefSignal),
    DefState(DefState),
    SpecInitialTransition(SpecInitialTransition),
}

/** Action definition */
pub struct DefAction {
    name: Ident,
    type_name: Option<TypeName>,
}

/** Choice definition */
pub struct DefChoice {
    name: Ident,
    guard: Ident,
    if_transition: TransitionExpr,
    else_transition: TransitionExpr,
}

/** Guard definition */
pub struct DefGuard {
    name: Ident,
    type_name: Option<TypeName>,
}

/** Transition expression */
pub struct TransitionExpr {
    actions: NodeList<Ident>,
    target: QualIdent,
}

/** Signal definition */
pub struct DefSignal {
    name: Ident,
    type_name: Option<TypeName>,
}

pub struct DefState {
    name: Ident,
    members: NodeList<Annotated<StateMember>>,
}

pub enum StateMember {
    DefChoice(DefChoice),
    DefState(DefState),
    SpecInitialTransition(SpecInitialTransition),

}

/** Initial state specifier */
pub struct SpecInitialTransition {
    transition: TransitionExpr
}

/** State entry specifier */
pub struct SpecStateEntry {
    actions: NodeList<Ident>
}

/** State exit specifier */
pub struct SpecStateExit{
    actions: NodeList<Ident>
}

/** Transition specifier */
pub struct SpecStateTransition{
    signal: Ident,
    guard: Option<Ident>,
    transition_or_do: TransitionOrDo,
}

/** Transition or do within transition specifier */
pub enum TransitionOrDo {
    Transition(TransitionExpr),
    Do(NodeList<Ident>)
}
