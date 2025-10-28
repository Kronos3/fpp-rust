use crate::*;

pub struct DefStateMachine {
    pub name: Ident,
    pub members: Vec<StateMachineMember>,
}

pub enum StateMachineMember {
    DefAction(AnnotatedNode<DefAction>),
    DefChoice(AnnotatedNode<DefChoice>),
    DefGuard(AnnotatedNode<DefGuard>),
    DefSignal(AnnotatedNode<DefSignal>),
    DefState(AnnotatedNode<DefState>),
    SpecInitialTransition(AnnotatedNode<SpecInitialTransition>),
}

/** Action definition */
pub struct DefAction {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

/** Choice definition */
pub struct DefChoice {
    pub name: Ident,
    pub guard: AstNode<Ident>,
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
    pub actions: Vec<AstNode<Ident>>,
    pub target: AstNode<QualIdent>,
}

/** Signal definition */
pub struct DefSignal {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
}

pub struct DefState {
    pub name: Ident,
    pub members: Vec<StateMember>,
}

pub enum StateMember {
    DefChoice(AnnotatedNode<DefChoice>),
    DefState(AnnotatedNode<DefState>),
    SpecInitialTransition(AnnotatedNode<SpecInitialTransition>),
}

/** Initial state specifier */
pub struct SpecInitialTransition {
    pub transition: AstNode<TransitionExpr>,
}

/** State entry specifier */
pub struct SpecStateEntry {
    pub actions: Vec<AstNode<Ident>>,
}

/** State exit specifier */
pub struct SpecStateExit {
    pub actions: Vec<AstNode<Ident>>,
}

/** Transition specifier */
pub struct SpecStateTransition {
    pub signal: AstNode<Ident>,
    pub guard: Option<AstNode<Ident>>,
    pub transition_or_do: TransitionOrDo,
}

/** Transition or do within transition specifier */
pub enum TransitionOrDo {
    Transition(AstNode<TransitionExpr>),
    Do(Vec<AstNode<Ident>>),
}
