use crate::semantics::generic_name_symbol_map::GenericNameSymbolMap;
use crate::semantics::generic_nested_scope::GenericNestedScope;
use crate::semantics::generic_scope::GenericScope;
use crate::semantics::{NameGroups, NameGroupsMap, Symbol};

pub type NestedScope<'a> = GenericNestedScope<
    'a,
    NameGroups,
    Symbol<'a>,
    NameGroupsMap<GenericNameSymbolMap<'a, Symbol<'a>>>,
>;

pub type Scope<'a> =
    GenericScope<'a, NameGroups, Symbol<'a>, NameGroupsMap<GenericNameSymbolMap<'a, Symbol<'a>>>>;
