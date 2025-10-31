use crate::{*};

/** Topology definition */
#[derive(Debug)]
pub struct DefTopology {
    pub name: Ident,
    pub members: Vec<Annotated<TopologyMember>>,
}

#[derive(Debug)]
pub enum TopologyMember {
    SpecCompInstance(SpecCompInstance),
    SpecConnectionGraph(SpecConnectionGraph),
    SpecInclude(SpecInclude),
    SpecTlmPacketSet(SpecTlmPacketSet),
    SpecTopImport(SpecImport),
}

#[derive(Debug)]
pub enum CompInstanceVisibility {
    Private,
    Public,
}

#[derive(Debug)]
pub struct SpecCompInstance {
    pub visibility: CompInstanceVisibility,
    pub instance: QualIdent,
}

#[derive(Debug)]
pub struct PortInstanceIdentifier {
    pub component_instance: QualIdent,
    pub port_name: Ident,
}

#[derive(Debug)]
pub struct Connection {
    pub is_unmatched: bool,
    pub from_port: PortInstanceIdentifier,
    pub from_index: Option<Expr>,
    pub to_port: PortInstanceIdentifier,
    pub to_index: Option<Expr>,
}

#[derive(Debug)]
pub enum ConnectionPatternKind {
    Command,
    Event,
    Health,
    Param,
    Telemetry,
    TextEvent,
    Time,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct TlmChannelIdentifier {
    pub component_instance: QualIdent,
    pub channel_name: Ident,
}

#[derive(Debug)]
pub struct SpecTlmPacketSet {
    pub name: Ident,
    pub members: Vec<Annotated<TlmPacketSetMember>>,
    pub omitted: Vec<Annotated<TlmChannelIdentifier>>,
}

#[derive(Debug)]
pub enum TlmPacketSetMember {
    SpecInclude(SpecInclude),
    SpecTlmPacket(SpecTlmPacket)
}

#[derive(Debug)]
pub struct SpecTlmPacket {
    pub name: Ident,
    pub id: Option<Expr>,
    pub group: Expr,
    pub members: Vec<TlmPacketMember>,
}

#[derive(Debug)]
pub enum TlmPacketMember {
    SpecInclude(SpecInclude),
    TlmChannelIdentifier(TlmChannelIdentifier)
}
