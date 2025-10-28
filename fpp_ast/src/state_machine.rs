use crate::{*};

pub struct DefStateMachine {
    name: Ident,
    members: Vec<StateMachineMember>,
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
    name: Ident,
    type_name: Option<AstNode<TypeName>>,
}

/** Choice definition */
pub struct DefChoice {
    name: Ident,
    guard: AstNode<Ident>,
    if_transition: AstNode<TransitionExpr>,
    else_transition: AstNode<TransitionExpr>,
}

/** Guard definition */
pub struct DefGuard {
    name: Ident,
    type_name: Option<AstNode<TypeName>>,
}

/** Transition expression */
pub struct TransitionExpr {
    actions: Vec<AstNode<Ident>>,
    target: AstNode<QualIdent>,
}

/** Signal definition */
pub struct DefSignal {
    name: Ident,
    type_name: Option<AstNode<TypeName>>,
}

pub struct DefState {
    name: Ident,
    members: Vec<StateMember>,
}

pub enum StateMember {
    DefChoice(AnnotatedNode<DefChoice>),
    DefState(AnnotatedNode<DefState>),
    SpecInitialTransition(AnnotatedNode<SpecInitialTransition>),
}

/** Initial state specifier */
pub struct SpecInitialTransition {
    transition: AstNode<TransitionExpr>
}

/** State entry specifier */
pub struct SpecStateEntry {
    actions: Vec<AstNode<Ident>>
}

/** State exit specifier */
pub struct SpecStateExit{
    actions: Vec<AstNode<Ident>>
}

/** Transition specifier */
pub struct SpecStateTransition{
    signal: AstNode<Ident>,
    guard: Option<AstNode<Ident>>,
    transition_or_do: TransitionOrDo,
}

/** Transition or do within transition specifier */
pub enum TransitionOrDo {
    Transition(AstNode<TransitionExpr>),
    Do(Vec<AstNode<Ident>>)
}
