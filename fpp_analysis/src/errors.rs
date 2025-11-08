use fpp_core::{Diagnostic, Level, Span};

#[derive(Debug)]
pub enum SemanticError {
    RedefinedSymbol {
        /// Name of the symbol being redefined
        name: String,
        /// Location of the duplicate symbol
        loc: Span,
        /// Location of the previous symbol that is clashing
        prev_loc: Span,
    },
    UndefinedSymbol {
        name: String,
        loc: Span,
    },
    InvalidSymbol {
        symbol_name: String,
        msg: String,
        loc: Span,
        def_loc: Span,
    },
}

pub type SemanticResult<T = ()> = Result<T, SemanticError>;

impl SemanticError {
    pub fn emit(self) {
        Into::<Diagnostic>::into(self).emit();
    }
}

impl Into<Diagnostic> for SemanticError {
    fn into(self) -> Diagnostic {
        match self {
            SemanticError::RedefinedSymbol {
                name,
                loc,
                prev_loc,
            } => Diagnostic::new(Level::Error, "duplicate symbol definition")
                .span_annotation(loc, format!("redefinition of symbol {}", name))
                .span_note(prev_loc, "previous definition is here"),
            SemanticError::UndefinedSymbol { name, loc } => {
                Diagnostic::new(Level::Error, "undefined symbol").span_annotation(loc, name)
            }
            SemanticError::InvalidSymbol {
                symbol_name,
                msg,
                loc,
                def_loc,
            } => Diagnostic::spanned(loc, Level::Error, msg)
                .span_note(def_loc, format!("{} defined here", symbol_name)),
        }
    }
}
