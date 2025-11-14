use crate::semantics::{NestedScope, Scope, Symbol, Type, UseDefMatching, Value};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct Analysis<'ast> {
    /** The mapping from symbols to their parent symbols */
    pub parent_symbol_map: HashMap<Symbol<'ast>, Symbol<'ast>>,
    /** The mapping from symbols with scopes to their scopes */
    pub symbol_scope_map: HashMap<Symbol<'ast>, Rc<RefCell<Scope<'ast>>>>,
    /** The mapping from uses (by node ID) to their definitions */
    pub use_def_map: HashMap<fpp_core::Node, Symbol<'ast>>,
    /** The list of use-def matchings on the current use-def path.
     *  Used during cycle analysis. */
    pub use_def_matching_list: Vec<UseDefMatching<'ast>>,
    /** The set of symbols visited so far */
    pub visited_symbol_set: HashSet<Symbol<'ast>>,
    /** The set of symbols on the current use-def path.
     *  Used during cycle analysis. */
    pub use_def_symbol_set: HashSet<Symbol<'ast>>,
    /** The current parent symbol */
    pub parent_symbol: Option<Symbol<'ast>>,
    /** The current nested scope for symbol lookup */
    pub nested_scope: NestedScope<'ast>,
    /** The set of files included when parsing input */
    pub included_file_set: HashSet<fpp_core::SourceFile>,
    /** The mapping from type and constant symbols, expressions,
     *  and type names to their types */
    pub type_map: HashMap<fpp_core::Node, Rc<Type>>,
    /** The mapping from constant symbols and expressions to their values. */
    pub value_map: HashMap<fpp_core::Node, Value>,
}

impl<'a> Analysis<'a> {
    pub fn new() -> Analysis<'a> {
        Analysis {
            parent_symbol_map: Default::default(),
            symbol_scope_map: Default::default(),
            use_def_map: Default::default(),
            use_def_matching_list: vec![],
            visited_symbol_set: Default::default(),
            use_def_symbol_set: Default::default(),
            parent_symbol: None,
            nested_scope: NestedScope::new(Scope::new()),
            included_file_set: Default::default(),
            type_map: Default::default(),
            value_map: Default::default(),
        }
    }
}
