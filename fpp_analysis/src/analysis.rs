use crate::semantics::{NestedScope, Scope, Symbol};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Analysis<'ast> {
    /** The mapping from symbols to their parent symbols */
    pub parent_symbol_map: HashMap<Symbol<'ast>, Symbol<'ast>>,
    /** The mapping from symbols with scopes to their scopes */
    pub symbol_scope_map: HashMap<Symbol<'ast>, Rc<RefCell<Scope<'ast>>>>,
    /** The mapping from uses (by node ID) to their definitions */
    pub use_def_map: HashMap<fpp_core::Node, Symbol<'ast>>,
    /** The current parent symbol */
    pub parent_symbol: Option<Symbol<'ast>>,
    /** The current nested scope for symbol lookup */
    pub nested_scope: NestedScope<'ast>,
}

impl<'a> Analysis<'a> {
    pub fn new() -> Analysis<'a> {
        Analysis {
            parent_symbol_map: Default::default(),
            symbol_scope_map: Default::default(),
            use_def_map: Default::default(),
            parent_symbol: None,
            nested_scope: NestedScope::new(Rc::new(RefCell::new(Scope::new()))),
        }
    }
}
