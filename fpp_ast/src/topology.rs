use crate::*;

/** Topology definition */
#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct DefTopology {
    pub name: Ident,
    pub members: Vec<TopologyMember>,
    pub implements: Vec<QualIdent>,
}

#[ast]
#[derive(AstAnnotated, Clone, DirectWalkable)]
pub enum TopologyMember {
    SpecInstance(SpecInstance),
    SpecConnectionGraph(SpecConnectionGraph),
    SpecInclude(SpecInclude),
    SpecTopPort(SpecTopPort),
    SpecTlmPacketSet(SpecTlmPacketSet),
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecInstance {
    pub instance: QualIdent,
}

#[ast]
#[derive(Debug, Clone, VisitorWalkable)]
pub struct PortInstanceIdentifier {
    pub interface_instance: QualIdent,
    pub port_name: Ident,
}

#[ast]
#[derive(Debug, Clone, VisitorWalkable)]
pub struct Connection {
    #[visitable(ignore)]
    pub is_unmatched: bool,
    pub from_port: PortInstanceIdentifier,
    pub from_index: Option<Expr>,
    pub to_port: PortInstanceIdentifier,
    pub to_index: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum ConnectionPatternKind {
    Command,
    Event,
    Health,
    Param,
    Telemetry,
    TextEvent,
    Time,
}

#[derive(Debug, Clone, DirectWalkable)]
pub enum SpecConnectionGraphKind {
    Direct {
        name: Ident,
        connections: Vec<Connection>,
    },
    Pattern {
        #[visitable(ignore)]
        kind: ConnectionPatternKind,
        source: QualIdent,
        targets: Vec<QualIdent>,
    },
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecConnectionGraph {
    pub kind: SpecConnectionGraphKind,
}

#[ast]
#[derive(Debug, Clone, VisitorWalkable)]
pub struct TlmChannelIdentifier {
    pub component_instance: QualIdent,
    pub channel_name: Ident,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecTopPort {
    pub name: Ident,
    pub underlying_port: PortInstanceIdentifier,
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecTlmPacketSet {
    pub name: Ident,
    pub members: Vec<TlmPacketSetMember>,
    pub omitted: Vec<TlmChannelIdentifier>,
}

#[ast]
#[derive(AstAnnotated, Clone, DirectWalkable)]
pub enum TlmPacketSetMember {
    SpecInclude(SpecInclude),
    SpecTlmPacket(SpecTlmPacket),
}

#[ast]
#[derive(AstAnnotated, Clone, VisitorWalkable)]
pub struct SpecTlmPacket {
    pub name: Ident,
    pub id: Option<Expr>,
    pub group: Expr,
    pub members: Vec<TlmPacketMember>,
}

#[ast]
#[derive(DirectWalkable, Clone)]
pub enum TlmPacketMember {
    SpecInclude(SpecInclude),
    TlmChannelIdentifier(TlmChannelIdentifier),
}
