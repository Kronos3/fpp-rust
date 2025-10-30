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
    pub opcode: Option<Expr>,
    pub priority: Option<Expr>,
    pub queue_full: Option<QueueFull>,
}

pub struct SpecContainer {
    pub name: Ident,
    pub id: Option<Expr>,
    pub default_priority: Option<Expr>,
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

pub struct SpecEvent {
    pub name: Ident,
    pub params: FormalParamList,
    pub severity: EventSeverity,
    pub id: Option<Expr>,
    pub format: AstNode<String>,
    pub throttle: Option<Expr>,
}

/** Internal port specifier */
pub struct SpecInternalPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub priority: Option<Expr>,
    pub queue_full: QueueFull,
}

pub struct SpecParam {
    pub name: Ident,
    pub type_name: AstNode<TypeName>,
    pub default: Option<AstNode<Expr>>,
    pub id: Option<AstNode<Expr>>,
    pub set_opcode: Option<AstNode<Expr>>,
    pub save_opcode: Option<AstNode<Expr>>,
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
    ProductRecv,
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
    pub record_type: TypeName,
    pub is_array: bool,
    pub id: Option<Expr>,
}

pub struct SpecStateMachineInstance {
    pub name: Ident,
    pub state_machine: QualIdent,
    pub priority: Option<Expr>,
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
    pub kind: TlmChannelLimitKind,
    pub value: Expr,
}

pub struct SpecTlmChannel {
    pub name: Ident,
    pub type_name: TypeName,
    pub id: Option<Expr>,
    pub update: Option<TlmChannelUpdate>,
    pub format: Option<AstNode<String>>,
    pub low: Vec<TlmChannelLimit>,
    pub high: Vec<TlmChannelLimit>
}
