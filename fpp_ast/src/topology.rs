use crate::{*};

/** Topology definition */
pub struct DefTopology {
    pub name: Ident,
    pub members: Vec<TopologyMember>,
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
    pub visibility: CompInstanceVisibility,
    pub instance: QualIdent,
}

pub struct PortInstanceIdentifier {
    pub component_instance: QualIdent,
    pub port_name: Ident,
}

pub struct Connection {
    pub is_unmatched: bool,
    pub from_port: PortInstanceIdentifier,
    pub from_index: Option<Expr>,
    pub to_port: PortInstanceIdentifier,
    pub to_index: Option<Expr>,
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
    pub component_instance: QualIdent,
    pub channel_name: Ident,
}

pub struct SpecTlmPacketSet {
    pub name: Ident,
    pub members: Vec<TlmPacketSetMember>,
    pub omitted: Vec<TlmChannelIdentifier>,
}

pub enum TlmPacketSetMember {
    SpecInclude(SpecInclude),
    SpecTlmPacket(SpecTlmPacket)
}

pub struct SpecTlmPacket {
    pub name: Ident,
    pub id: Option<Expr>,
    pub group: Expr,
    pub members: Vec<TlmPacketMember>,
}

pub enum TlmPacketMember {
    SpecInclude(SpecInclude),
    TlmChannelIdentifier(TlmChannelIdentifier)
}
