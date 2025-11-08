mod analysis;
mod errors;

pub use analysis::*;

pub mod analyzers {
    pub(crate) mod analyzer;
    pub(crate) mod basic_use_analyzer;
    pub(crate) mod nested_analyzer;
    pub(crate) mod use_analyzer;
}

pub mod passes {
    mod check;
    pub use check::*;

    mod enter_symbols;
    pub(crate) use enter_symbols::*;
    mod check_uses;
    pub(crate) use check_uses::*;
}

pub mod semantics {
    mod symbol;
    pub use symbol::*;

    mod name;
    pub use name::*;

    mod implied_use;
    pub use implied_use::*;

    mod scope;
    pub use scope::*;

    mod name_groups;
    pub use name_groups::*;

    mod generic_name_symbol_map;
    mod generic_nested_scope;
    mod generic_scope;
}

#[cfg(test)]
mod test;
