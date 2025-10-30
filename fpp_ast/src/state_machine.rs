use crate::*;

pub struct DefStateMachine {
    pub name: Ident,
    pub members: Option<Vec<Annotated<StateMachineMember>>>,
}

pub enum StateMachineMember {
    DefAction(AstNode<DefAction>),
    DefChoice(AstNode<DefChoice>),
    DefGuard(AstNode<DefGuard>),
    DefSignal(AstNode<DefSignal>),
    DefState(AstNode<DefState>),
    SpecInitialTransition(AstNode<SpecInitialTransition>),

}

/** Action definition */
pub struct DefAction {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

/** Choice definition */
pub struct DefChoice {
    pub name: Ident,
    pub guard: Ident,
    pub if_transition: AstNode<TransitionExpr>,
    pub else_transition: AstNode<TransitionExpr>,
}

/** Guard definition */
pub struct DefGuard {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

/** Transition expression */
pub struct TransitionExpr {
    pub actions: DoExpr,
    pub target: AstNode<QualIdent>,
}

/** Signal definition */
pub struct DefSignal {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

pub struct DefState {
    pub name: Ident,
    pub members: Vec<Annotated<StateMember>>,
}

pub enum StateMember {
    DefChoice(AstNode<DefChoice>),
    DefState(AstNode<DefState>),
    SpecInitialTransition(AstNode<SpecInitialTransition>),
    SpecStateEntry(AstNode<SpecStateEntry>),
    SpecStateExit(AstNode<SpecStateExit>),
    SpecStateTransition(AstNode<SpecStateTransition>)
}

/** Initial state specifier */
pub struct SpecInitialTransition {
    pub transition: AstNode<TransitionExpr>,
}

/** State entry specifier */
pub struct SpecStateEntry {
    pub actions: DoExpr,
}

/** State exit specifier */
pub struct SpecStateExit {
    pub actions: DoExpr,
}

/** Transition specifier */
pub struct SpecStateTransition {
    pub signal: Ident,
    pub guard: Option<Ident>,
    pub transition_or_do: TransitionOrDo,
}

pub struct DoExpr(pub Vec<Ident>);

/** Transition or do within transition specifier */
pub enum TransitionOrDo {
    Transition(AstNode<TransitionExpr>),
    Do(DoExpr),
}
