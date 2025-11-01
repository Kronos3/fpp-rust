use crate::*;

/** Topology definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefTopology {
    pub name: Ident,
    pub members: Vec<TopologyMember>,
    pub implements: Vec<QualIdent>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub enum TopologyMember {
    SpecInstance(SpecInstance),
    SpecConnectionGraph(SpecConnectionGraph),
    SpecInclude(SpecInclude),
    SpecTopPort(SpecTopPort),
    SpecTlmPacketSet(SpecTlmPacketSet),
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecInstance {
    pub instance: QualIdent,
}

#[ast_node]
#[derive(Debug)]
pub struct PortInstanceIdentifier {
    pub interface_instance: QualIdent,
    pub port_name: Ident,
}

#[ast_node]
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

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecConnectionGraph {
    pub kind: SpecConnectionGraphKind,
}

#[ast_node]
#[derive(Debug)]
pub struct TlmChannelIdentifier {
    pub component_instance: QualIdent,
    pub channel_name: Ident,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecTopPort {
    pub name: Ident,
    pub underlying_port: PortInstanceIdentifier,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecTlmPacketSet {
    pub name: Ident,
    pub members: Vec<TlmPacketSetMember>,
    pub omitted: Vec<TlmChannelIdentifier>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub enum TlmPacketSetMember {
    SpecInclude(SpecInclude),
    SpecTlmPacket(SpecTlmPacket),
}

#[ast_node]
#[ast_annotated]
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
    TlmChannelIdentifier(TlmChannelIdentifier),
}
