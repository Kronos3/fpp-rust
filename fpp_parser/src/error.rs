use fpp_core::file::SourceFile;
use fpp_core::span::{Position};
use crate::token::{Token, TokenKind};

// pub(crate) struct ParseError {
//     pub expected: TokenKind,
//     pub got: Option<Token>,
//     pub source_file: SourceFile,
//     pub pos: Position,
//     pub msg: &'static str
// }

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
}

pub type ParseResult<T> = Result<T, ParseError>;
