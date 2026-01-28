use crate::errors::SemanticResult;
use crate::semantics::{NameGroup, NestedScope, Scope, Symbol, Type, UseDefMatching, Value};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::sync::Arc;

#[derive(Debug)]
pub struct Analysis {
    /** The mapping from symbols to their parent symbols */
    pub parent_symbol_map: HashMap<Symbol, Symbol>,
    /** The outermost scope */
    pub global_scope: Scope,
    /** The mapping from symbols with scopes to their scopes */
    pub symbol_scope_map: HashMap<Symbol, Scope>,
    /** The mapping from definition node ID to their entered symbol */
    pub symbol_map: HashMap<fpp_core::Node, Symbol>,
    /** The mapping from uses (by node ID) to their definitions */
    pub use_def_map: HashMap<fpp_core::Node, Symbol>,
    /** The list of use-def matchings on the current use-def path.
     *  Used during cycle analysis. */
    pub use_def_matching_list: Vec<UseDefMatching>,
    /** The set of symbols visited so far */
    pub visited_symbol_set: HashSet<Symbol>,
    /** The set of symbols on the current use-def path.
     *  Used during cycle analysis. */
    pub use_def_symbol_set: HashSet<Symbol>,
    /** The current parent symbol */
    pub parent_symbol: Option<Symbol>,
    /** The current nested scope for symbol lookup */
    pub nested_scope: NestedScope,
    /** The mapping from included files `.fppi` to their parent/which context they were included in */
    pub parent_file_map: HashMap<fpp_core::SourceFile, (fpp_core::SourceFile, fpp_parser::IncludeParentKind)>,
    /** The mapping from type and constant symbols, expressions,
     *  and type names to their types */
    pub type_map: HashMap<fpp_core::Node, Arc<Type>>,
    /** The mapping from constant symbols and expressions to their values. */
    pub value_map: HashMap<fpp_core::Node, Value>,
}

impl Analysis {
    pub fn new() -> Analysis {
        // Validate that Analysis is thread safe
        fn is_sync<T: Sync>() {}
        is_sync::<Analysis>();

        Analysis {
            parent_symbol_map: Default::default(),
            global_scope: Scope::new(),
            symbol_scope_map: Default::default(),
            symbol_map: Default::default(),
            use_def_map: Default::default(),
            use_def_matching_list: vec![],
            visited_symbol_set: Default::default(),
            use_def_symbol_set: Default::default(),
            parent_symbol: None,
            nested_scope: NestedScope::new(),
            parent_file_map: Default::default(),
            type_map: Default::default(),
            value_map: Default::default(),
        }
    }

    pub fn get_symbol<N: fpp_ast::AstNode>(&self, node: &N) -> Symbol {
        self.symbol_map.get(&node.id()).unwrap().clone()
    }

    pub fn get_scope(&self, symbol: &Option<Symbol>) -> &Scope {
        match symbol {
            None => &self.global_scope,
            Some(s) => self
                .symbol_scope_map
                .get(s)
                .expect("symbol does not have a scope"),
        }
    }

    pub fn symbol_get(&self, name_group: NameGroup, name: &str) -> Option<Symbol> {
        self.nested_scope
            .search(|s| self.get_scope(s).get(name_group, name))
    }

    pub fn get_scope_mut(&mut self, symbol: &Option<Symbol>) -> &mut Scope {
        match symbol {
            None => &mut self.global_scope,
            Some(s) => self
                .symbol_scope_map
                .get_mut(s)
                .expect("symbol does not have a scope"),
        }
    }

    pub fn symbol_put(&mut self, name_group: NameGroup, symbol: Symbol) -> SemanticResult {
        let scope = self.nested_scope.current().clone();
        self.get_scope_mut(&scope).put(name_group, symbol)
    }
}
