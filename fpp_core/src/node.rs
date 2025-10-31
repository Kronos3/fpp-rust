use crate::Span;
use crate::interface::with;
use std::fmt::{Debug, Formatter};

pub trait Positioned {
    fn span(&self) -> Span;
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct NodeId {
    pub(crate) handle: u32,
}

impl NodeId {
    pub fn new(span: Span) -> NodeId {
        with(|w| w.node_add(&span))
    }

    pub(crate) fn next(&self) -> NodeId {
        NodeId {
            handle: self.handle + 1,
        }
    }
}

impl Positioned for NodeId {
    fn span(&self) -> Span {
        with(|w| w.node_span(self))
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.span().fmt(f)
    }
}
