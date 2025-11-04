use crate::*;

use fpp_macros::ast;

#[ast]
#[derive(AstAnnotated, MatchWalkable)]
pub enum ComponentMember {
    DefAbsType(DefAbsType),
    DefAliasType(DefAliasType),
    DefArray(DefArray),
    DefConstant(DefConstant),
    DefEnum(DefEnum),
    DefStateMachine(DefStateMachine),
    DefStruct(DefStruct),
    SpecCommand(SpecCommand),
    SpecContainer(SpecContainer),
    SpecEvent(SpecEvent),
    SpecInclude(SpecInclude),
    SpecInternalPort(SpecInternalPort),
    SpecParam(SpecParam),
    SpecPortInstance(SpecPortInstance),
    SpecPortMatching(SpecPortMatching),
    SpecRecord(SpecRecord),
    SpecStateMachineInstance(SpecStateMachineInstance),
    SpecTlmChannel(SpecTlmChannel),
    SpecImportInterface(SpecImport),
}

#[derive(Debug)]
pub enum InputPortKind {
    Async,
    Guarded,
    Sync,
}

/** Queue full behavior */
#[derive(Debug)]
pub enum QueueFull {
    Assert,
    Block,
    Drop,
    Hook,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecCommand {
    #[visitable(ignore)]
    pub kind: InputPortKind,
    pub name: Ident,
    pub params: FormalParamList,
    pub opcode: Option<Expr>,
    pub priority: Option<Expr>,
    #[visitable(ignore)]
    pub queue_full: Option<QueueFull>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecContainer {
    pub name: Ident,
    pub id: Option<Expr>,
    pub default_priority: Option<Expr>,
}

#[derive(Debug)]
pub enum EventSeverity {
    ActivityHigh,
    ActivityLow,
    Command,
    Diagnostic,
    Fatal,
    WarningHigh,
    WarningLow,
}

#[ast]
#[derive(Debug, Walkable)]
pub struct EventThrottle {
    pub count: Expr,
    pub every: Option<Expr>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecEvent {
    pub name: Ident,
    pub params: FormalParamList,
    #[visitable(ignore)]
    pub severity: EventSeverity,
    pub id: Option<Expr>,
    #[visitable(ignore)]
    pub format: LitString,
    pub throttle: Option<EventThrottle>,
}

/** Internal port specifier */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecInternalPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub priority: Option<Expr>,
    #[visitable(ignore)]
    pub queue_full: Option<QueueFull>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecParam {
    pub name: Ident,
    pub type_name: TypeName,
    pub default: Option<Expr>,
    pub id: Option<Expr>,
    pub set_opcode: Option<Expr>,
    pub save_opcode: Option<Expr>,
    #[visitable(ignore)]
    pub is_external: bool,
}

#[derive(Debug)]
pub enum GeneralPortInstanceKind {
    Input(InputPortKind),
    Output,
}

#[derive(Debug)]
pub enum SpecialPortInstanceKind {
    CommandRecv,
    CommandReg,
    CommandResp,
    Event,
    ParamGet,
    ParamSet,
    ProductGet,
    ProductRecv,
    ProductRequest,
    ProductSend,
    Telemetry,
    TextEvent,
    TimeGet,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecPortMatching {
    pub port1: Ident,
    pub port2: Ident,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecRecord {
    pub name: Ident,
    pub record_type: TypeName,
    #[visitable(ignore)]
    pub is_array: bool,
    pub id: Option<Expr>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecStateMachineInstance {
    pub name: Ident,
    pub state_machine: QualIdent,
    pub priority: Option<Expr>,
    #[visitable(ignore)]
    pub queue_full: Option<QueueFull>,
}

#[derive(Debug)]
pub enum TlmChannelUpdate {
    Always,
    OnChange,
}

#[derive(Debug)]
pub enum TlmChannelLimitKind {
    Red,
    Orange,
    Yellow,
}

#[ast]
#[derive(Debug, Walkable)]
pub struct TlmChannelLimit {
    #[visitable(ignore)]
    pub kind: TlmChannelLimitKind,
    pub value: Expr,
}

// const _: () = {
//     impl<'__ast, __V> crate::visit::Walkable<'__ast, __V> for TlmChannelLimit
//     where
//         __V: crate::visit::Visitor<'__ast>,
//     {
//         fn walk_ref(&'__ast self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
//             __visitor.visit_tlm_channel_limit(self)?;
//             match *self {
//                 TlmChannelLimit {
//                     value: ref __binding_1, ..
//                 } => { { crate::visit::Walkable::walk_ref(__binding_1, __visitor )? } }
//             }
//             std::ops::ControlFlow::Continue(())
//         }
//     }
//     impl<__V> crate::visit::MutWalkable<__V> for TlmChannelLimit
//     where
//         __V: crate::visit::MutVisitor,
//     {
//         fn walk_mut(&mut self, __visitor: &mut __V) -> std::ops::ControlFlow<__V::Break> {
//             let r = ::std::cell::RefCell::new(self);
//             __visitor.visit_tlm_channel_limit(r.borrow_mut())?;
//             match r.borrow_mut().deref_mut() {
//                 TlmChannelLimit {
//                     value: __binding_1, ..
//                 } => { { crate::visit::MutWalkable::walk_mut(__binding_1, __visitor )? } }
//             }
//             std::ops::ControlFlow::Continue(())
//         }
//     }
// };

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecTlmChannel {
    pub name: Ident,
    pub type_name: TypeName,
    pub id: Option<Expr>,
    #[visitable(ignore)]
    pub update: Option<TlmChannelUpdate>,
    #[visitable(ignore)]
    pub format: Option<LitString>,
    pub low: Vec<TlmChannelLimit>,
    pub high: Vec<TlmChannelLimit>,
}
