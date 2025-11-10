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
    pub use enter_symbols::*;

    mod check_uses;
    pub use check_uses::*;

    mod check_use_def_cycles;
    pub use check_use_def_cycles::*;

    mod check_type_uses;
    pub use check_type_uses::*;
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

    mod use_def_matching;
    pub use use_def_matching::*;

    mod types;
    pub use types::*;

    mod value;
    pub use value::*;

    mod format;
    pub use format::*;

    mod generic_name_symbol_map;
    mod generic_nested_scope;
    mod generic_scope;
}

#[cfg(test)]
mod test {
    mod lib;

    mod defs {
        mod test;
    }

    mod cycles {
        mod test;
    }

    mod invalid_symbols {
        mod test;
    }
}
