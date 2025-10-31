use crate::*;

#[derive(Debug)]
pub struct DefStateMachine {
    pub name: Ident,
    pub members: Option<Vec<Annotated<StateMachineMember>>>,
}

#[derive(Debug)]
pub enum StateMachineMember {
    DefAction(AstNode<DefAction>),
    DefChoice(AstNode<DefChoice>),
    DefGuard(AstNode<DefGuard>),
    DefSignal(AstNode<DefSignal>),
    DefState(AstNode<DefState>),
    SpecInitialTransition(AstNode<SpecInitialTransition>),

}

/** Action definition */
#[derive(Debug)]
pub struct DefAction {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

/** Choice definition */
#[derive(Debug)]
pub struct DefChoice {
    pub name: Ident,
    pub guard: Ident,
    pub if_transition: AstNode<TransitionExpr>,
    pub else_transition: AstNode<TransitionExpr>,
}

/** Guard definition */
#[derive(Debug)]
pub struct DefGuard {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

/** Transition expression */
#[derive(Debug)]
pub struct TransitionExpr {
    pub actions: DoExpr,
    pub target: AstNode<QualIdent>,
}

/** Signal definition */
#[derive(Debug)]
pub struct DefSignal {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

#[derive(Debug)]
pub struct DefState {
    pub name: Ident,
    pub members: Vec<Annotated<StateMember>>,
}

#[derive(Debug)]
pub enum StateMember {
    DefChoice(AstNode<DefChoice>),
    DefState(AstNode<DefState>),
    SpecInitialTransition(AstNode<SpecInitialTransition>),
    SpecStateEntry(AstNode<SpecStateEntry>),
    SpecStateExit(AstNode<SpecStateExit>),
    SpecStateTransition(AstNode<SpecStateTransition>)
}

/** Initial state specifier */
#[derive(Debug)]
pub struct SpecInitialTransition {
    pub transition: AstNode<TransitionExpr>,
}

/** State entry specifier */
#[derive(Debug)]
pub struct SpecStateEntry {
    pub actions: DoExpr,
}

/** State exit specifier */
#[derive(Debug)]
pub struct SpecStateExit {
    pub actions: DoExpr,
}

/** Transition specifier */
#[derive(Debug)]
pub struct SpecStateTransition {
    pub signal: Ident,
    pub guard: Option<Ident>,
    pub transition_or_do: TransitionOrDo,
}

#[derive(Debug)]
pub struct DoExpr(pub Vec<Ident>);

/** Transition or do within transition specifier */
#[derive(Debug)]
pub enum TransitionOrDo {
    Transition(AstNode<TransitionExpr>),
    Do(DoExpr),
}
