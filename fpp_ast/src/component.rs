use crate::{*};

pub enum ComponentMember {
    DefAbsType(AnnotatedNode<DefAbsType>),
    DefAliasType(AnnotatedNode<DefAliasType>),
    DefArray(AnnotatedNode<DefArray>),
    DefConstant(AnnotatedNode<DefConstant>),
    DefEnum(AnnotatedNode<DefEnum>),
    DefStateMachine(AnnotatedNode<DefStateMachine>),
    DefStruct(AnnotatedNode<DefStruct>),
    SpecCommand(AnnotatedNode<SpecCommand>),
    SpecContainer(AnnotatedNode<SpecContainer>),
    SpecEvent(AnnotatedNode<SpecEvent>),
    SpecInclude(AnnotatedNode<SpecInclude>),
    SpecInternalPort(AnnotatedNode<SpecInternalPort>),
    SpecParam(AnnotatedNode<SpecParam>),
    SpecPortInstance(AnnotatedNode<SpecPortInstance>),
    SpecPortMatching(AnnotatedNode<SpecPortMatching>),
    SpecRecord(AnnotatedNode<SpecRecord>),
    SpecStateMachineInstance(AnnotatedNode<SpecStateMachineInstance>),
    SpecTlmChannel(AnnotatedNode<SpecTlmChannel>),
    SpecImportInterface(AnnotatedNode<SpecImport>),
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
    kind: InputPortKind,
    name: Ident,
    params: FormalParamList,
    opcode: Option<Expr>,
    priority: Option<Expr>,
    queue_full: Option<QueueFull>,
}

pub struct SpecContainer {
    name: Ident,
    id: Option<Expr>,
    default_priority: Option<Expr>,
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
    name: Ident,
    params: FormalParamList,
    severity: EventSeverity,
    id: Option<Expr>,
    format: AstNode<String>,
    throttle: Option<Expr>,
}

/** Internal port specifier */
pub struct SpecInternalPort {
    name: Ident,
    params: FormalParamList,
    priority: Option<Expr>,
    queue_full: QueueFull,
}

pub struct SpecParam {
    name: Ident,
    type_name: TypeName,
    default: Option<Expr>,
    id: Option<Expr>,
    set_opcode: Option<Expr>,
    save_opcode: Option<Expr>,
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
    port1: Ident,
    port2: Ident,
}

pub struct SpecRecord {
    name: Ident,
    record_type: TypeName,
    is_array: bool,
    id: Option<Expr>,
}

pub struct SpecStateMachineInstance {
    name: Ident,
    state_machine: QualIdent,
    priority: Option<Expr>,
    queue_full: Option<QueueFull>
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
    kind: TlmChannelLimitKind,
    value: Expr,
}

pub struct SpecTlmChannel {
    name: Ident,
    type_name: TypeName,
    id: Option<Expr>,
    update: Option<TlmChannelUpdate>,
    format: Option<AstNode<String>>,
    low: Vec<TlmChannelLimit>,
    high: Vec<TlmChannelLimit>
}
