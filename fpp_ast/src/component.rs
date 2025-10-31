use crate::{*};

pub enum ComponentMember {
    DefAbsType(AstNode<DefAbsType>),
    DefAliasType(AstNode<DefAliasType>),
    DefArray(AstNode<DefArray>),
    DefConstant(AstNode<DefConstant>),
    DefEnum(AstNode<DefEnum>),
    DefStateMachine(AstNode<DefStateMachine>),
    DefStruct(AstNode<DefStruct>),
    SpecCommand(AstNode<SpecCommand>),
    SpecContainer(AstNode<SpecContainer>),
    SpecEvent(AstNode<SpecEvent>),
    SpecInclude(AstNode<SpecInclude>),
    SpecInternalPort(AstNode<SpecInternalPort>),
    SpecParam(AstNode<SpecParam>),
    SpecPortInstance(AstNode<SpecPortInstance>),
    SpecPortMatching(AstNode<SpecPortMatching>),
    SpecRecord(AstNode<SpecRecord>),
    SpecStateMachineInstance(AstNode<SpecStateMachineInstance>),
    SpecTlmChannel(AstNode<SpecTlmChannel>),
    SpecImportInterface(AstNode<SpecImport>),
}

pub enum InputPortKind {
    Async,
    Guarded,
    Sync
}

/** Queue full behavior */
pub enum QueueFull {
    Assert,
    Block,
    Drop,
    Hook,
}

pub struct SpecCommand {
    pub kind: InputPortKind,
    pub name: Ident,
    pub params: FormalParamList,
    pub opcode: Option<AstNode<Expr>>,
    pub priority: Option<AstNode<Expr>>,
    pub queue_full: Option<QueueFull>,
}

pub struct SpecContainer {
    pub name: Ident,
    pub id: Option<AstNode<Expr>>,
    pub default_priority: Option<AstNode<Expr>>,
}

pub enum EventSeverity {
    ActivityHigh,
    ActivityLow,
    Command,
    Diagnostic,
    Fatal,
    WarningHigh,
    WarningLow,
}

pub struct EventThrottle {
    pub count: AstNode<Expr>,
    pub every: Option<AstNode<Expr>>
}

pub struct SpecEvent {
    pub name: Ident,
    pub params: FormalParamList,
    pub severity: EventSeverity,
    pub id: Option<AstNode<Expr>>,
    pub format: AstNode<String>,
    pub throttle: Option<AstNode<EventThrottle>>,
}

/** Internal port specifier */
pub struct SpecInternalPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub priority: Option<AstNode<Expr>>,
    pub queue_full: Option<QueueFull>,
}

pub struct SpecParam {
    pub name: Ident,
    pub type_name: AstNode<TypeName>,
    pub default: Option<AstNode<Expr>>,
    pub id: Option<AstNode<Expr>>,
    pub set_opcode: Option<AstNode<Expr>>,
    pub save_opcode: Option<AstNode<Expr>>,
    pub is_external: bool
}

pub enum GeneralPortInstanceKind {
    Input(InputPortKind),
    Output
}

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

pub struct SpecPortMatching {
    pub port1: Ident,
    pub port2: Ident,
}

pub struct SpecRecord {
    pub name: Ident,
    pub record_type: AstNode<TypeName>,
    pub is_array: bool,
    pub id: Option<AstNode<Expr>>,
}

pub struct SpecStateMachineInstance {
    pub name: Ident,
    pub state_machine: AstNode<QualIdent>,
    pub priority: Option<AstNode<Expr>>,
    pub queue_full: Option<QueueFull>
}

pub enum TlmChannelUpdate {
    Always,
    OnChange,
}

pub enum TlmChannelLimitKind {
    Red,
    Orange,
    Yellow,
}

pub struct TlmChannelLimit {
    pub kind: AstNode<TlmChannelLimitKind>,
    pub value: AstNode<Expr>,
}

pub struct SpecTlmChannel {
    pub name: Ident,
    pub type_name: AstNode<TypeName>,
    pub id: Option<AstNode<Expr>>,
    pub update: Option<TlmChannelUpdate>,
    pub format: Option<AstNode<String>>,
    pub low: Vec<TlmChannelLimit>,
    pub high: Vec<TlmChannelLimit>
}
