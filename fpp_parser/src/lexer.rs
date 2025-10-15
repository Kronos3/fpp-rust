use fpp_core::file::SourceFile;
use crate::cursor::Cursor;
use crate::token::{Token, TokenKind};

pub struct Lexer<'a> {
    pos: u32,
    file: &'a SourceFile,
    cursor: Cursor<'a>
}

impl<'a> Lexer<'a> {
    pub fn new(file: &'a SourceFile) -> Lexer<'a> {
        Lexer{
            pos: 0,
            file,
            cursor: Cursor::new(file.read())
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        loop {
            self.pos += self.cursor.pos_within_token();
            self.cursor.reset_pos_within_token();
            let start = self.pos;

            match self.cursor.next_token_kind() {
                TokenKind::EOF => return None,
                TokenKind::Whitespace => {},
                TokenKind::Error(err) => todo!(),
                kind => {
                    return Some(Token::new(
                        kind, self.file, start, self.cursor.pos_within_token(),
                    ))
                }
            }
        }
    }
}