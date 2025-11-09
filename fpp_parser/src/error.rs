use crate::token::TokenKind;
use fpp_core::{Diagnostic, Level, Position, Span};
use std::fmt::Formatter;

#[derive(Debug)]
pub(crate) enum ParseError {
    ExpectedOneOf {
        expected: Vec<TokenKind>,
        got_kind: TokenKind,
        got_span: Span,
        msg: &'static str,
    },

    ExpectedToken {
        expected: TokenKind,
        got: TokenKind,
        last: Span,
        msg: &'static str,
    },

    UnexpectedEof {
        last: Span,
    },

    IncludeCycle {
        span: Span,
        include_cycle: Vec<Position>,
    },
}

struct TokenList(Vec<TokenKind>);
impl std::fmt::Display for TokenList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for tok in self.0.iter() {
            if !first {
                f.write_str(", ")?;
            }

            tok.fmt(f)?;
            first = false
        }

        Ok(())
    }
}

impl Into<Diagnostic> for ParseError {
    fn into(self) -> Diagnostic {
        match self {
            ParseError::ExpectedOneOf {
                got_kind,
                got_span,
                msg,
                expected,
            } => Diagnostic::new(Level::Error, "syntax error")
                .span_annotation(got_span, msg)
                .note(format!("expected one of {}", TokenList(expected)))
                .note(format!("got {}", got_kind)),
            ParseError::ExpectedToken {
                last,
                msg,
                expected,
                got,
            } => Diagnostic::new(Level::Error, "syntax error")
                .span_annotation(last, msg)
                .note(format!("expected {}", expected))
                .note(format!("got {}", got)),
            ParseError::UnexpectedEof { last } => {
                Diagnostic::spanned(last, Level::Error, "unexpected end of input")
            }
            ParseError::IncludeCycle {
                span,
                include_cycle,
            } => {
                let diag = Diagnostic::spanned(span, Level::Error, "include cycle detected");
                include_cycle.into_iter().fold(diag, |diag, pos| {
                    diag.note(format! {"included from {}", pos})
                })
            }
        }
    }
}

pub(crate) type ParseResult<T> = Result<T, ParseError>;
