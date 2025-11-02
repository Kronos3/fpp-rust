use crate::token::TokenKind;
use fpp_core::{Diagnostic, Level, Span};

#[derive(Debug)]
pub enum ParseError {
    ExpectedOneOf {
        expected: Vec<TokenKind>,
        last: Span,
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
}

impl Into<Diagnostic> for ParseError {
    fn into(self) -> Diagnostic {
        match self {
            ParseError::ExpectedOneOf {
                last,
                msg,
                expected,
            } => Diagnostic::spanned(last, Level::Error, msg)
                .note(format!("expected one of {:?}", expected)),
            ParseError::ExpectedToken { last, msg, .. } => {
                Diagnostic::spanned(last, Level::Error, msg)
            }
            ParseError::UnexpectedEof { last } => {
                Diagnostic::spanned(last, Level::Error, "unexpected end of input")
            }
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
