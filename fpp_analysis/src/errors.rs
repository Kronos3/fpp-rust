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
}

pub type SemanticResult = Result<(), SemanticError>;

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
        }
    }
}
