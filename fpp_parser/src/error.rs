use crate::token::TokenKind;
use fpp_core::{Diagnostic, Level, Span};

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
                .span_error(got_span, msg)
                .note(format!("expected one of {:?}", expected))
                .note(format!("got {:?}", got_kind)),
            ParseError::ExpectedToken {
                last,
                msg,
                expected,
                got,
            } => Diagnostic::new(Level::Error, "syntax error")
                .span_error(last, msg)
                .note(format!("expected {:?}", expected))
                .note(format!("got {:?}", got)),
            ParseError::UnexpectedEof { last } => {
                Diagnostic::spanned(last, Level::Error, "unexpected end of input")
            },
        }
    }
}

pub(crate) type ParseResult<T> = Result<T, ParseError>;
