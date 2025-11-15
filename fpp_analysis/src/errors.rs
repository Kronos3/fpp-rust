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
    EnumConstantShouldBeImplied {
        loc: Span,
    },
    EnumConstantShouldBeExplicit {
        loc: Span,
    },
    DuplicateEnumConstant {
        value: i128,
        loc: Span,
        prev_loc: Span,
    },
    InvalidIntValue {
        loc: Span,
        v: Option<i128>,
        msg: String,
    },
    DivisionByZero {
        loc: Span,
    },
    InvalidTypeForMemberSelection {
        loc: Span,
        member: String,
        type_name: String,
    },
    FormatStringMismatchLength {
        format_locs: Vec<Span>,
        type_locs: Vec<Span>,
    },
    FormatStringInvalidReplacement {
        format_loc: Span,
        type_loc: Span,
        msg: String,
    },
    FormatStringInvalidPrecision {
        loc: Span,
        value: i32,
        max: i32,
    },
    ArrayDefaultMismatchedSize {
        loc: Span,
        size_loc: Span,
        value_size: usize,
        type_size: i128,
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
            SemanticError::EnumConstantShouldBeImplied { loc } => {
                Diagnostic::spanned(loc, Level::Error, "expected constant value to be implied")
                    .note("enum constants must be all explicit or all implied")
            }
            SemanticError::EnumConstantShouldBeExplicit { loc } => {
                Diagnostic::spanned(loc, Level::Error, "expected constant value to be explicit")
                    .note("enum constants must be all explicit or all implied")
            }
            SemanticError::DuplicateEnumConstant {
                value,
                loc,
                prev_loc,
            } => Diagnostic::spanned(
                loc,
                Level::Error,
                format!("duplicate enum constant `{}`", value),
            )
            .span_note(prev_loc, "previously defined here"),
            SemanticError::InvalidIntValue { loc, v, msg } => {
                let diag = Diagnostic::spanned(loc, Level::Error, msg);
                match v {
                    None => diag,
                    Some(v) => diag.note(format!("expression evaluated to `{}`", v)),
                }
            }
            SemanticError::DivisionByZero { loc } => {
                Diagnostic::spanned(loc, Level::Error, "division by zero")
            }
            SemanticError::InvalidTypeForMemberSelection {
                loc,
                member,
                type_name,
            } => Diagnostic::spanned(
                loc,
                Level::Error,
                format!("{} has no member `{}`", type_name, member),
            ),
            SemanticError::FormatStringMismatchLength {
                format_locs,
                type_locs,
            } => {
                if format_locs.len() < type_locs.len() {
                    let diag =
                        Diagnostic::new(Level::Error, "format string missing replacement fields");
                    type_locs[format_locs.len()..]
                        .iter()
                        .fold(diag, |diag, loc| {
                            diag.span_note(loc.clone(), "missing format replacement field")
                        })
                } else {
                    let diag = Diagnostic::new(Level::Error, "too many format specifiers");
                    format_locs[type_locs.len()..]
                        .iter()
                        .fold(diag, |diag, loc| {
                            diag.span_annotation(loc.clone(), "no field to format")
                        })
                }
            }
            SemanticError::FormatStringInvalidReplacement {
                format_loc,
                type_loc,
                msg,
            } => Diagnostic::spanned(format_loc, Level::Error, msg)
                .span_note(type_loc, "type defined here"),
            SemanticError::FormatStringInvalidPrecision { loc, value, max } => Diagnostic::spanned(
                loc,
                Level::Error,
                format!(
                    "precision value `{}` is larger than the maximum ({})",
                    value, max
                ),
            ),
            SemanticError::ArrayDefaultMismatchedSize {
                loc,
                size_loc,
                value_size,
                type_size,
            } => Diagnostic::spanned(
                loc,
                Level::Error,
                "cannot convert value to array type due to mismatched sizes",
            )
            .note(format!("value size `{}`", value_size))
            .span_note(size_loc, format!("array size `{}`", type_size)),
        }
    }
}
