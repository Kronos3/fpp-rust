use crate::semantics::TypeConversionError;
use fpp_core::{Diagnostic, Level, Span};

#[derive(Debug)]
pub struct SymbolUse {
    pub def_loc: Span,
    pub use_loc: Span,
}

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
        ng: String,
        name: String,
        loc: Span,
    },
    UseDefCycle {
        loc: Span,
        cycle: Vec<SymbolUse>,
    },
    InvalidSymbol {
        symbol_name: String,
        msg: String,
        loc: Span,
        def_loc: Span,
    },
    InvalidType {
        loc: Span,
        msg: String,
    },
    DuplicateStructMember {
        name: String,
        loc: Span,
        prev_loc: Span,
    },
    TypeConversion {
        loc: Span,
        msg: String,
        err: TypeConversionError,
    },
    EmptyArray {
        loc: Span,
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
            SemanticError::UndefinedSymbol { ng, name, loc } => {
                Diagnostic::new(Level::Error, "undefined symbol")
                    .span_annotation(loc, format!("cannot find {} `{}` in scope", ng, name))
            }
            SemanticError::InvalidSymbol {
                symbol_name,
                msg,
                loc,
                def_loc,
            } => Diagnostic::spanned(loc, Level::Error, msg)
                .span_note(def_loc, format!("{} defined here", symbol_name)),
            SemanticError::UseDefCycle { loc, cycle } => cycle.iter().enumerate().fold(
                Diagnostic::spanned(loc, Level::Error, "encountered symbol use-definition cycle"),
                |out, (i, suse)| match i {
                    0 => out.span_note(suse.def_loc, "defined here"),
                    _ if i == cycle.len() - 1 => out.span_note(suse.use_loc, "used here"),
                    _ => out
                        .span_note(suse.use_loc, "used here")
                        .span_note(suse.def_loc, "defined here"),
                },
            ),
            SemanticError::InvalidType { loc, msg } => Diagnostic::spanned(loc, Level::Error, msg),
            SemanticError::DuplicateStructMember {
                name,
                loc,
                prev_loc,
            } => Diagnostic::spanned(
                loc,
                Level::Error,
                format!("duplicate struct member `{}`", name),
            )
            .span_note(prev_loc, "previously defined here"),
            SemanticError::TypeConversion { loc, msg, err } => {
                err.annotate(Diagnostic::spanned(loc, Level::Error, msg))
            }
            SemanticError::EmptyArray { loc } => {
                Diagnostic::spanned(loc, Level::Error, "array expression may not be empty")
            }
        }
    }
}
