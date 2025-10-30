use crate::error::{ParseError, ParseResult};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};
use fpp_core::span::Position;
use std::collections::VecDeque;

pub struct Cursor<'a> {
    lexer: Lexer<'a>,
    token_queue: VecDeque<Token>,
    last_consumed_span: Option<fpp_core::span::Span>,
}

impl Cursor {
    /// Look ahead 'n' tokens and get the token kind
    /// This will pull in tokens from the lexer when needed
    pub fn peek(&mut self, n: usize) -> TokenKind {
        if self.token_queue.len() > n {
            self.token_queue.get(n).unwrap().kind()
        } else {
            // Queue up as many tokens as we need
            for _ in 0..n {
                match self.lexer.next_token() {
                    None => return TokenKind::EOF,
                    Some(tok) => self.token_queue.push_back(tok),
                }
            }

            self.token_queue.get(n).unwrap().kind()
        }
    }

    /// Generate a new error while expecting a certain type of token
    /// Messages here are meant to only be simple literals, the full error message
    /// will be formatted given other context information.
    pub fn err_expected_token(
        &self,
        msg: &'static str,
        expected: TokenKind,
        got: TokenKind,
    ) -> ParseError {
        ParseError::ExpectedToken {
            expected,
            got,
            source_file: self.lexer.file(),
            pos: match self.last_consumed_span {
                None => Position::start(self.lexer.file()),
                Some(span) => span.end(),
            },
            msg,
        }
    }

    /// Insert a single token into the front of the queue to be pulled next
    // pub fn insert(&mut self, token: Token) {
    //     self.token_queue.push_front(token)
    // }

    pub fn err_expected_one_of(
        &self,
        msg: &'static str,
        expected_one_of: Vec<TokenKind>,
    ) -> ParseError {
        ParseError::ExpectedOneOf {
            expected: expected_one_of,
            source_file: self.lexer.file(),
            pos: match self.last_consumed_span {
                None => Position::start(self.lexer.file()),
                Some(span) => span.end(),
            },
            msg,
        }
    }

    /// Consume the next token in the stream
    /// Returns None if EOF has been reached
    pub fn next(&mut self) -> Option<Token> {
        // Try to pull token off the queue
        let tok = match self.token_queue.pop_front() {
            // No more tokens in our queue, go to the lexer
            None => self.lexer.next_token(),
            Some(tok) => Some(tok),
        };

        match tok {
            Some(tok) => {
                self.last_consumed_span = Some(tok.span());
                Some(tok)
            }
            None => None,
        }
    }

    #[inline]
    pub fn consume(&mut self, kind: TokenKind) -> ParseResult<Token> {
        match self.next() {
            None => Err(self.err_expected_token("unexpected end of file", kind, TokenKind::EOF)),
            Some(tok) => {
                if tok.kind() != kind {
                    Err(self.err_expected_token("unexpected token", kind, tok.kind()))
                } else {
                    Ok(tok)
                }
            }
        }
    }
}
