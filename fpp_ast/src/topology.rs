use crate::{*};

/** Topology definition */
#[derive(Debug)]
pub struct DefTopology {
    pub name: Ident,
    pub members: Vec<Annotated<TopologyMember>>,
    pub implements: Vec<AstNode<QualIdent>>
}

#[derive(Debug)]
pub enum TopologyMember {
    SpecInstance(AstNode<SpecInstance>),
    SpecConnectionGraph(AstNode<SpecConnectionGraph>),
    SpecInclude(AstNode<SpecInclude>),
    SpecTopPort(AstNode<SpecTopPort>),
    SpecTlmPacketSet(AstNode<SpecTlmPacketSet>),
}

#[derive(Debug)]
pub struct SpecInstance {
    pub instance: AstNode<QualIdent>,
}

#[derive(Debug)]
pub struct PortInstanceIdentifier {
    pub interface_instance: AstNode<QualIdent>,
    pub port_name: Ident,
}

#[derive(Debug)]
pub struct Connection {
    pub is_unmatched: bool,
    pub from_port: AstNode<PortInstanceIdentifier>,
    pub from_index: Option<AstNode<Expr>>,
    pub to_port: AstNode<PortInstanceIdentifier>,
    pub to_index: Option<AstNode<Expr>>,
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
        connections: Vec<AstNode<Connection>>,
    },

    Pattern {
        kind: ConnectionPatternKind,
        source: AstNode<QualIdent>,
        targets: Vec<AstNode<QualIdent>>,
    },
}

#[derive(Debug)]
pub struct TlmChannelIdentifier {
    pub component_instance: AstNode<QualIdent>,
    pub channel_name: Ident,
}

#[derive(Debug)]
pub struct SpecTopPort {
    pub name: Ident,
    pub underlying_port: AstNode<PortInstanceIdentifier>
}

#[derive(Debug)]
pub struct SpecTlmPacketSet {
    pub name: Ident,
    pub members: Vec<Annotated<TlmPacketSetMember>>,
    pub omitted: Vec<AstNode<TlmChannelIdentifier>>,
}

#[derive(Debug)]
pub enum TlmPacketSetMember {
    SpecInclude(AstNode<SpecInclude>),
    SpecTlmPacket(AstNode<SpecTlmPacket>)
}

#[derive(Debug)]
pub struct SpecTlmPacket {
    pub name: Ident,
    pub id: Option<AstNode<Expr>>,
    pub group: AstNode<Expr>,
    pub members: Vec<TlmPacketMember>,
}

#[derive(Debug)]
pub enum TlmPacketMember {
    SpecInclude(AstNode<SpecInclude>),
    TlmChannelIdentifier(AstNode<TlmChannelIdentifier>)
}
