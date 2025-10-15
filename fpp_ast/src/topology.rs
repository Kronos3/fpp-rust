use crate::common::{Expr, Ident, QualIdent};
use crate::{SpecImport, SpecInclude};

/** Topology definition */
pub struct DefTopology {
    name: Ident,
    members: Vec<TopologyMember>,
}

pub enum TopologyMember {
    SpecCompInstance(SpecCompInstance),
    SpecConnectionGraph(SpecConnectionGraph),
    SpecInclude(SpecInclude),
    SpecTlmPacketSet(SpecTlmPacketSet),
    SpecTopImport(SpecImport),
}

pub enum CompInstanceVisibility {
    Private,
    Public,
}

pub struct SpecCompInstance {
    visibility: CompInstanceVisibility,
    instance: QualIdent,
}

pub struct PortInstanceIdentifier {
    component_instance: QualIdent,
    port_name: Ident,
}

pub struct Connection {
    is_unmatched: bool,
    from_port: PortInstanceIdentifier,
    from_index: Option<Expr>,
    to_port: PortInstanceIdentifier,
    to_index: Option<Expr>,
}

pub enum ConnectionPatternKind {
    Command,
    Event,
    Health,
    Param,
    Telemetry,
    TextEvent,
    Time,
}

pub enum SpecConnectionGraph {
    Direct {
        name: Ident,
        connections: Vec<Connection>,
    },

    Pattern {
        kind: ConnectionPatternKind,
        source: QualIdent,
        targets: Vec<QualIdent>,
    },
}

pub struct TlmChannelIdentifier {
    component_instance: QualIdent,
    channel_name: Ident,
}

pub struct SpecTlmPacketSet {
    name: Ident,
    members: Vec<TlmPacketSetMember>,
    omitted: Vec<TlmChannelIdentifier>,
}

pub enum TlmPacketSetMember {
    SpecInclude(SpecInclude),
    SpecTlmPacket(SpecTlmPacket)
}

pub struct SpecTlmPacket {
    name: Ident,
    id: Option<Expr>,
    group: Expr,
    members: Vec<TlmPacketMember>,
}

pub enum TlmPacketMember {
    SpecInclude(SpecInclude),
    TlmChannelIdentifier(TlmChannelIdentifier)
}
