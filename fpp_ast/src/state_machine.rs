use crate::*;

#[ast]
#[derive(AstAnnotated)]
pub struct DefStateMachine {
    pub name: Ident,
    pub members: Option<Vec<StateMachineMember>>,
}

#[ast]
#[derive(AstAnnotated)]
pub enum StateMachineMember {
    DefAction(DefAction),
    DefChoice(DefChoice),
    DefGuard(DefGuard),
    DefSignal(DefSignal),
    DefState(DefState),
    SpecInitialTransition(SpecInitialTransition),

}

/** Action definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefAction {
    pub name: Ident,
    pub type_name: Option<TypeName>,
}

/** Choice definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefChoice {
    pub name: Ident,
    pub guard: Ident,
    pub if_transition: TransitionExpr,
    pub else_transition: TransitionExpr,
}

/** Guard definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefGuard {
    pub name: Ident,
    pub type_name: Option<TypeName>,
}

/** Transition expression */
#[ast]
#[derive(Debug)]
pub struct TransitionExpr {
    pub actions: Option<DoExpr>,
    pub target: QualIdent,
}

/** Signal definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefSignal {
    pub name: Ident,
    pub type_name: Option<TypeName>,
}

#[ast]
#[derive(AstAnnotated)]
pub struct DefState {
    pub name: Ident,
    pub members: Vec<StateMember>,
}

#[ast]
#[derive(AstAnnotated)]
pub enum StateMember {
    DefChoice(DefChoice),
    DefState(DefState),
    SpecInitialTransition(SpecInitialTransition),
    SpecStateEntry(SpecStateEntry),
    SpecStateExit(SpecStateExit),
    SpecStateTransition(SpecStateTransition),
}

/** Initial state specifier */
#[ast]
#[derive(AstAnnotated)]
pub struct SpecInitialTransition {
    pub transition: TransitionExpr,
}

/** State entry specifier */
#[ast]
#[derive(AstAnnotated)]
pub struct SpecStateEntry {
    pub actions: DoExpr,
}

/** State exit specifier */
#[ast]
#[derive(AstAnnotated)]
pub struct SpecStateExit {
    pub actions: DoExpr,
}

/** Transition specifier */
#[ast]
#[derive(AstAnnotated)]
pub struct SpecStateTransition {
    pub signal: Ident,
    pub guard: Option<Ident>,
    pub transition_or_do: TransitionOrDo,
}

#[ast]
#[derive(Debug)]
pub struct DoExpr {
    pub actions: Vec<Ident>,
}

/** Transition or do within transition specifier */
#[derive(Debug)]
pub enum TransitionOrDo {
    Transition(TransitionExpr),
    Do(DoExpr),
}
