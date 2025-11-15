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

    mod check_expr_types;
    pub use check_expr_types::*;

    mod eval_implied_enum_consts;
    pub use eval_implied_enum_consts::*;

    mod eval_constant_exprs;
    pub use eval_constant_exprs::*;

    mod finalize_type_defs;
    pub use finalize_type_defs::*;
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

    mod cycles {
        mod test;
    }

    mod defs {
        mod test;
    }

    // mod interface {
    //     mod test;
    // }

    mod types {
        mod test;
    }

    // mod port_matching {
    //     mod test;
    // }

    // mod record {
    //     mod test;
    // }

    // mod port_numbering {
    //     mod test;
    // }

    mod array {
        mod test;
    }

    // mod tlm_packets {
    //     mod test;
    // }

    mod enums {
        mod test;
    }

    // mod tlm_channel {
    //     mod test;
    // }

    // mod framework_defs {
    //     mod test;
    // }

    mod expr {
        mod test;
    }

    // mod component {
    //     mod test;
    // }

    // mod param {
    //     mod test;
    // }

    // mod container {
    //     mod test;
    // }

    // mod component_instance_def {
    //     mod test;
    // }

    // mod port_instance {
    //     mod test;
    // }

    mod constant {
        mod test;
    }

    mod structs {
        mod test;
    }

    mod invalid_symbols {
        mod test;
    }

    mod redef {
        mod test;
    }

    // mod unconnected {
    //     mod test;
    // }

    // mod command {
    //     mod test;
    // }

    // mod port {
    //     mod test;
    // }

    // mod instance_spec {
    //     mod test;
    // }

    // mod connection_direct {
    //     mod test;
    // }

    // mod spec_init {
    //     mod test;
    // }

    // mod event {
    //     mod test;
    // }

    // mod spec_loc {
    //     mod test;
    // }

    // mod top_import {
    //     mod test;
    // }

    // mod state_machine_instance {
    //     mod test;
    // }

    // mod connection_pattern {
    //     mod test;
    // }

    // mod internal_port {
    //     mod test;
    // }

    // mod top_ports {
    //     mod test;
    // }

    // mod state_machine {
    //     mod types {
    //         mod test;
    //     }

    //     mod initial_transitions {
    //         mod test;
    //     }

    //     mod transition_graph {
    //         mod test;
    //     }

    //     mod signal_uses {
    //         mod test;
    //     }

    //     mod redef {
    //         mod test;
    //     }

    //     mod typed_elements {
    //         mod test;
    //     }

    //     mod undef {
    //         mod test;
    //     }
    // }
}
