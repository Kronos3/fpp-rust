use crate::context::with;
use crate::{Span};

pub trait Positioned {
    fn span(&self) -> Span;
}

pub struct NodeId {
    handle: u32,
}

impl NodeId {
    pub fn new(span: Span) -> NodeId {
        with(|w| w.add_node(&span))
    }
}

impl Positioned for NodeId {
    fn span(&self) -> Span {
        with(|w| w.node_span(self))
    }
}
