mod analysis;
pub use analysis::{*};

pub mod passes {
    mod check;
    pub use check::{*};

    mod enter_symbols;
    pub(crate) use enter_symbols::{*};
}

pub mod semantics {
    mod symbol;
    pub use symbol::{*};

    mod name_groups;
    pub use name_groups::{*};
}
