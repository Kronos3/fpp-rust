use crate::semantics::Symbol;
use std::collections::HashMap;

struct Scope(i32);

pub struct Analysis<'ast> {
    /** The current parent symbol */
    parent_symbol: Option<Symbol<'ast>>,
    /** The mapping from symbols to their parent symbols */
    parent_symbol_map: HashMap<Symbol<'ast>, Symbol<'ast>>,
    /** The mapping from symbols with scopes to their scopes */
    symbol_scope_map: HashMap<Symbol<'ast>, Scope>,
}

impl<'a> Analysis<'a> {
    pub fn new() -> Analysis<'a> {
        Analysis {
            parent_symbol: None,
            parent_symbol_map: Default::default(),
            symbol_scope_map: Default::default(),
        }
    }
}
