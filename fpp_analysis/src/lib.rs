mod analysis;
mod errors;

pub use analysis::*;

pub mod passes {
    mod check;
    pub use check::*;

    mod enter_symbols;
    pub(crate) use enter_symbols::*;
}

pub mod semantics {
    mod symbol;
    pub use symbol::*;

    mod scope;
    pub use scope::*;

    mod name_groups;
    pub use name_groups::*;

    mod generic_scope;
    mod generic_nested_scope;
    mod generic_name_symbol_map;
}
