use crate::*;

/** Topology definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefTopology {
    pub name: Ident,
    pub members: Vec<TopologyMember>,
    pub implements: Vec<QualIdent>,
}

#[ast]
#[derive(AstAnnotated)]
pub enum TopologyMember {
    SpecInstance(SpecInstance),
    SpecConnectionGraph(SpecConnectionGraph),
    SpecInclude(SpecInclude),
    SpecTopPort(SpecTopPort),
    SpecTlmPacketSet(SpecTlmPacketSet),
}

#[ast]
#[derive(AstAnnotated)]
pub struct SpecInstance {
    pub instance: QualIdent,
}

#[ast]
#[derive(Debug)]
pub struct PortInstanceIdentifier {
    pub interface_instance: QualIdent,
    pub port_name: Ident,
}

#[ast]
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
pub enum SpecConnectionGraphKind {
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

#[ast]
#[derive(AstAnnotated)]
pub struct SpecConnectionGraph {
    pub kind: SpecConnectionGraphKind,
}

#[ast]
#[derive(Debug)]
pub struct TlmChannelIdentifier {
    pub component_instance: QualIdent,
    pub channel_name: Ident,
}

#[ast]
#[derive(AstAnnotated)]
pub struct SpecTopPort {
    pub name: Ident,
    pub underlying_port: PortInstanceIdentifier,
}

#[ast]
#[derive(AstAnnotated)]
pub struct SpecTlmPacketSet {
    pub name: Ident,
    pub members: Vec<TlmPacketSetMember>,
    pub omitted: Vec<TlmChannelIdentifier>,
}

#[ast]
#[derive(AstAnnotated)]
pub enum TlmPacketSetMember {
    SpecInclude(SpecInclude),
    SpecTlmPacket(SpecTlmPacket),
}

#[ast]
#[derive(AstAnnotated)]
pub struct SpecTlmPacket {
    pub name: Ident,
    pub id: Option<Expr>,
    pub group: Expr,
    pub members: Vec<TlmPacketMember>,
}

#[derive(Debug)]
pub enum TlmPacketMember {
    SpecInclude(SpecInclude),
    TlmChannelIdentifier(TlmChannelIdentifier),
}
