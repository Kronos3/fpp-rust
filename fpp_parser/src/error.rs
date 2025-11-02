use crate::token::TokenKind;
use fpp_core::Position;

#[derive(Debug)]
pub enum ParseError {
    FileOpen {
        error: fpp_core::Error,
    },

    ExpectedOneOf {
        expected: Vec<TokenKind>,
        pos: Position,
        msg: &'static str,
    },

    ExpectedToken {
        expected: TokenKind,
        got: TokenKind,
        pos: Position,
        msg: &'static str,
    },

    UnexpectedEof {
        pos: Position,
    },
}

pub type ParseResult<T> = Result<T, ParseError>;
