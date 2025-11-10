use crate::*;

#[ast]
#[derive(AstAnnotated, Clone, DirectWalkable)]
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
    SpecInterfaceImport(SpecInterfaceImport),
}

#[derive(Debug, Clone)]
pub enum InputPortKind {
    Async,
    Guarded,
    Sync,
}

/** Queue full behavior */
#[derive(Debug, Clone)]
pub enum QueueFull {
    Assert,
    Block,
    Drop,
    Hook,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
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
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecContainer {
    pub name: Ident,
    pub id: Option<Expr>,
    pub default_priority: Option<Expr>,
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, VisitorWalkable)]
pub struct EventThrottle {
    pub count: Expr,
    pub every: Option<Expr>,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
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
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecInternalPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub priority: Option<Expr>,
    #[visitable(ignore)]
    pub queue_full: Option<QueueFull>,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
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

#[derive(Debug, Clone)]
pub enum GeneralPortInstanceKind {
    Input(InputPortKind),
    Output,
}

#[derive(Debug, Clone)]
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
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecPortMatching {
    pub port1: Ident,
    pub port2: Ident,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecRecord {
    pub name: Ident,
    pub record_type: TypeName,
    #[visitable(ignore)]
    pub is_array: bool,
    pub id: Option<Expr>,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecStateMachineInstance {
    pub name: Ident,
    pub state_machine: QualIdent,
    pub priority: Option<Expr>,
    #[visitable(ignore)]
    pub queue_full: Option<QueueFull>,
}

#[derive(Debug, Clone)]
pub enum TlmChannelUpdate {
    Always,
    OnChange,
}

#[derive(Debug, Clone)]
pub enum TlmChannelLimitKind {
    Red,
    Orange,
    Yellow,
}

#[ast]
#[derive(Debug, Clone, VisitorWalkable)]
pub struct TlmChannelLimit {
    #[visitable(ignore)]
    pub kind: TlmChannelLimitKind,
    pub value: Expr,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
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
