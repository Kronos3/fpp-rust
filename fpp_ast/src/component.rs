use crate::{*};

use fpp_macros::ast_node;

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
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
    Sync
}

/** Queue full behavior */
#[derive(Debug)]
pub enum QueueFull {
    Assert,
    Block,
    Drop,
    Hook,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecCommand {
    pub kind: InputPortKind,
    pub name: Ident,
    pub params: FormalParamList,
    pub opcode: Option<Expr>,
    pub priority: Option<Expr>,
    pub queue_full: Option<QueueFull>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
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

#[ast_node]
#[derive(Debug)]
pub struct EventThrottle {
    pub count: Expr,
    pub every: Option<Expr>
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecEvent {
    pub name: Ident,
    pub params: FormalParamList,
    pub severity: EventSeverity,
    pub id: Option<Expr>,
    pub format: LitString,
    pub throttle: Option<EventThrottle>,
}

/** Internal port specifier */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecInternalPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub priority: Option<Expr>,
    pub queue_full: Option<QueueFull>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecParam {
    pub name: Ident,
    pub type_name: TypeName,
    pub default: Option<Expr>,
    pub id: Option<Expr>,
    pub set_opcode: Option<Expr>,
    pub save_opcode: Option<Expr>,
    pub is_external: bool
}

#[derive(Debug)]
pub enum GeneralPortInstanceKind {
    Input(InputPortKind),
    Output
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

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecPortMatching {
    pub port1: Ident,
    pub port2: Ident,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecRecord {
    pub name: Ident,
    pub record_type: TypeName,
    pub is_array: bool,
    pub id: Option<Expr>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecStateMachineInstance {
    pub name: Ident,
    pub state_machine: QualIdent,
    pub priority: Option<Expr>,
    pub queue_full: Option<QueueFull>
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

#[ast_node]
#[derive(Debug)]
pub struct TlmChannelLimit {
    pub kind: TlmChannelLimitKind,
    pub value: Expr,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecTlmChannel {
    pub name: Ident,
    pub type_name: TypeName,
    pub id: Option<Expr>,
    pub update: Option<TlmChannelUpdate>,
    pub format: Option<LitString>,
    pub low: Vec<TlmChannelLimit>,
    pub high: Vec<TlmChannelLimit>
}
