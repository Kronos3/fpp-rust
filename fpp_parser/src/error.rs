use crate::token::TokenKind;
use fpp_core::{Position, SourceFile};

#[derive(Debug)]
pub enum ParseError {
    ExpectedOneOf {
        expected: Vec<TokenKind>,
        source_file: SourceFile,
        pos: Position,
        msg: &'static str,
    },

    ExpectedToken {
        expected: TokenKind,
        got: TokenKind,
        source_file: SourceFile,
        pos: Position,
        msg: &'static str,
    },

    UnexpectedEof {
        source_file: SourceFile,
        pos: Position,
    },
}

pub type ParseResult<T> = Result<T, ParseError>;
