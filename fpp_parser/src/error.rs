use fpp_core::{SourceFile, Position};
use crate::token::{TokenKind};

#[derive(Debug)]
pub(crate) enum ParseError {
    ExpectedOneOf {
        expected: Vec<TokenKind>,
        source_file: SourceFile,
        pos: Position,
        msg: &'static str
    },

    ExpectedToken {
        expected: TokenKind,
        got: TokenKind,
        source_file: SourceFile,
        pos: Position,
        msg: &'static str
    },

    UnexpectedEof {
        source_file: SourceFile,
        pos: Position,
    },

    NotImplemented
}

pub type ParseResult<T> = Result<T, ParseError>;
