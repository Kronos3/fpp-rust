use crate::semantics::generic_name_symbol_map::GenericNameSymbolMap;
use crate::semantics::generic_nested_scope::GenericNestedScope;
use crate::semantics::generic_scope::GenericScope;
use crate::semantics::{NameGroup, NameGroupMap, Symbol};

pub type NestedScope<'a> = GenericNestedScope<
    'a,
    NameGroup,
    Symbol<'a>,
    NameGroupMap<GenericNameSymbolMap<'a, Symbol<'a>>>,
>;

pub type Scope<'a> =
    GenericScope<'a, NameGroup, Symbol<'a>, NameGroupMap<GenericNameSymbolMap<'a, Symbol<'a>>>>;
