use crate::common::{Expr, FormalParamList, Ident, StringNode};
use crate::{*};

pub enum ComponentKind {
    Active,
    Passive,
    Queued,
}

/** Component definition */
#[derive(fpp_derive::Ast)]
pub struct DefComponent {
    kind: ComponentKind,
    name: Ident,
    members: Vec<Annotated<ComponentMember>>,
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

#[derive(fpp_derive::Ast)]
pub struct SpecCommand {
    kind: InputPortKind,
    name: Ident,
    params: FormalParamList,
    opcode: Option<Expr>,
    priority: Option<Expr>,
    queue_full: Option<QueueFull>,
}

#[derive(fpp_derive::Ast)]
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

#[derive(fpp_derive::Ast)]
pub struct SpecEvent {
    name: Ident,
    params: FormalParamList,
    severity: EventSeverity,
    id: Option<Expr>,
    format: StringNode,
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

pub enum SpecPortInstance {
    General{
        kind: GeneralPortInstanceKind,
        name: Ident,
        size: Option<Expr>,
        port: Option<QualIdent>,
        priority: Option<Expr>,
        queue_full: Option<QueueFull>,
    },
    Special{
        input_kind: Option<InputPortKind>,
        kind: SpecialPortInstanceKind,
        name: Ident,
        priority: Option<Expr>,
        queue_full: Option<QueueFull>,
    }
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
    format: Option<StringNode>,
    low: Vec<TlmChannelLimit>,
    high: Vec<TlmChannelLimit>
}

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
