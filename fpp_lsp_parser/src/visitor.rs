use crate::{Parse, SyntaxNode};

pub enum VisitorResult {
    /// Visit all children
    Recurse,
    /// Go to next sibling
    Next,
}

pub trait Visitor {
    type State;

    fn visit(&self, state: &mut Self::State, node: &SyntaxNode) -> VisitorResult;
}

fn visit_node<State, V: Visitor<State = State>>(node: &SyntaxNode, state: &mut State, visitor: &V) {
    match visitor.visit(state, node) {
        VisitorResult::Recurse => {
            for child in node.children() {
                visit_node(&child, state, visitor)
            }
        }
        VisitorResult::Next => {}
    }
}

impl Parse {
    pub fn visit<State, V: Visitor<State = State>>(&self, state: &mut State, visitor: &V) {
        visit_node(&self.syntax_node(), state, visitor);
    }
}
