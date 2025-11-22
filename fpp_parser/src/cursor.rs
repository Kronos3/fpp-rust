use crate::error::{ParseError, ParseResult};
use crate::token::Token;
use fpp_core::{Diagnostic, Level, SourceFile, Span, Spanned};
use fpp_lexer::{Lexer, TokenKind};
use std::collections::VecDeque;

pub struct Cursor<'a> {
    lexer: Lexer<'a>,

    /// The lexer only tells us the length of the next tokens
    /// We need to track the current position in the file
    pos: usize,
    content: &'a str,
    file: SourceFile,
    include_span: Option<Span>,
    lookahead: Option<fpp_lexer::Token>,

    token_queue: VecDeque<Token>,
    last_consumed_span: Span,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(
        source_file: SourceFile,
        content: &'a str,
        include_span: Option<Span>,
    ) -> Cursor<'a> {
        Cursor {
            lexer: Lexer::new(content),
            pos: 0,
            content,
            file: source_file,
            token_queue: Default::default(),
            last_consumed_span: Span::new(source_file, 1, 0, None),
            include_span,
            lookahead: None,
        }
    }

    pub fn emit_errors(&self) {
        self.lexer.errors().for_each(|err| {
            Diagnostic::new(
                Span::new(self.file, err.pos, err.len, self.include_span),
                Level::Error,
                "syntax error: invalid token",
            )
            .annotation(err.msg.clone())
            .emit();
        })
    }

    /// Keep eating whitespace/comments/newlines until we reach another token
    /// Return the token we reached (or None if EOF)
    fn eat_newlines(&mut self) -> Option<fpp_lexer::Token> {
        loop {
            let token = self.lexer.next()?;
            match token.kind {
                TokenKind::Eol | TokenKind::Comment | TokenKind::Whitespace => {
                    self.pos += token.len;
                }
                _ => return Some(token),
            }
        }
    }

    fn next_internal(&mut self) -> Option<Token> {
        loop {
            let prev = self.pos;
            let tok = match self.lookahead.take() {
                None => self.lexer.next()?,
                Some(lookahead) => lookahead,
            };

            self.pos += tok.len;
            match tok.kind {
                TokenKind::EOF => unreachable!(),
                TokenKind::Unknown => {
                    Diagnostic::new(
                        Span::new(self.file, prev, tok.len, self.include_span),
                        Level::Error,
                        format!(
                            "syntax error: invalid character {:#?}",
                            self.content.as_bytes()[prev] as char
                        ),
                    )
                    .emit();
                }
                TokenKind::Whitespace => {}
                TokenKind::Eol | TokenKind::Comment => {
                    // Check what comes after the EOL to see if this actually an EOL or
                    // just whitespace
                    return match self.eat_newlines() {
                        None => Some(Token::new(
                            TokenKind::Eol,
                            None,
                            self.file,
                            prev,
                            self.pos - prev,
                            self.include_span,
                        )),
                        Some(lookahead) => {
                            match lookahead.kind {
                                TokenKind::RightParen
                                | TokenKind::RightCurly
                                | TokenKind::RightSquare => {
                                    // Absorb all the previous whitespace before these closing tokens
                                    let lookahead_prev = self.pos;
                                    self.pos += lookahead.len;
                                    Some(Token::new(
                                        lookahead.kind,
                                        None,
                                        self.file,
                                        lookahead_prev,
                                        lookahead.len,
                                        self.include_span,
                                    ))
                                }
                                _ => {
                                    // Save this other type of token for later
                                    self.lookahead = Some(lookahead);

                                    // This is actually a newline delimiter
                                    Some(Token::new(
                                        TokenKind::Eol,
                                        None,
                                        self.file,
                                        prev,
                                        self.pos - prev,
                                        self.include_span,
                                    ))
                                }
                            }
                        }
                    };
                }
                TokenKind::Identifier => {
                    // Check if this identifier is an escaped keyword
                    let start = if self.content.as_bytes()[prev] == ('$' as u8) {
                        prev + 1
                    } else {
                        prev
                    };

                    return Some(Token::new(
                        TokenKind::Identifier,
                        Some(self.content[start..self.pos].to_string()),
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }
                TokenKind::PostAnnotation => {
                    self.lookahead = self.eat_newlines();
                    return Some(Token::new(
                        tok.kind,
                        Some(
                            self.content[prev + 2..=(prev + tok.len - 1)]
                                .trim()
                                .to_string(),
                        ),
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }

                TokenKind::PreAnnotation => {
                    self.lookahead = self.eat_newlines();
                    return Some(Token::new(
                        tok.kind,
                        Some(self.content[prev + 1..=prev + tok.len].trim().to_string()),
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }

                TokenKind::LiteralString => {
                    let text = if tok.len >= 2 {
                        self.content[prev + 1..=(prev + tok.len - 2)].to_string()
                    } else {
                        "".to_string()
                    };

                    return Some(Token::new(
                        tok.kind,
                        Some(text),
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }

                TokenKind::LiteralMultilineString { indent } => {
                    let text = if tok.len >= 6 && self.content.len() > prev + tok.len {
                        let raw_text = self.content[prev + 3..(prev + 3 + tok.len - 6)].to_string();
                        let lines: Vec<_> = raw_text.split('\n').map(|l| {
                            if l.len() > indent as usize {
                                l[(indent as usize)..].to_string()
                            } else {
                                "".to_string()
                            }
                        })
                            .collect();
                        lines.join("\n")
                    } else {
                        "".to_string()
                    };

                    return Some(Token::new(
                        tok.kind,
                        Some(text),
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }

                // Tokens that do not absorb newlines after and have text
                TokenKind::LiteralFloat | TokenKind::LiteralInt => {
                    return Some(Token::new(
                        tok.kind,
                        Some(self.content[prev..(prev + tok.len)].to_string()),
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }

                // Tokens that do not absorb newlines after (and do not have text)
                TokenKind::Keyword(_)
                | TokenKind::Dot
                | TokenKind::RightParen
                | TokenKind::RightCurly
                | TokenKind::RightSquare => {
                    return Some(Token::new(
                        tok.kind,
                        None,
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }

                // Tokens that eat the newlines after them
                TokenKind::Star
                | TokenKind::RightArrow
                | TokenKind::Slash
                | TokenKind::Minus
                | TokenKind::Plus
                | TokenKind::Equals
                | TokenKind::Semi
                | TokenKind::Comma
                | TokenKind::Colon
                | TokenKind::LeftParen
                | TokenKind::LeftCurly
                | TokenKind::LeftSquare => {
                    self.lookahead = self.eat_newlines();
                    return Some(Token::new(
                        tok.kind,
                        None,
                        self.file,
                        prev,
                        tok.len,
                        self.include_span,
                    ));
                }
            }
        }
    }

    fn peek_internal(&mut self, n: usize) -> Option<&Token> {
        if self.token_queue.len() > n {
            Some(self.token_queue.get(n).unwrap())
        } else {
            // Queue up as many tokens as we need
            while self.token_queue.len() <= n {
                let tok = self.next_internal()?;
                self.token_queue.push_back(tok);
            }

            Some(self.token_queue.get(n).unwrap())
        }
    }

    pub fn peek_span(&mut self, n: usize) -> Option<Span> {
        match self.peek_internal(n) {
            Some(tok) => Some(tok.span()),
            _ => None,
        }
    }

    /// Look ahead 'n' tokens and get the token kind
    /// This will pull in tokens from the lexer when needed
    pub fn peek(&mut self, n: usize) -> TokenKind {
        match self.peek_internal(n) {
            Some(tok) => tok.kind(),
            _ => TokenKind::EOF,
        }
    }

    pub fn last_token_span(&self) -> Span {
        self.last_consumed_span
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
            last: self.last_consumed_span,
            msg,
        }
    }

    pub fn err_unexpected_eof(&self) -> ParseError {
        ParseError::UnexpectedEof {
            last: self.last_consumed_span,
        }
    }

    /// Insert a single token into the front of the queue to be pulled next
    // pub fn insert(&mut self, token: Token) {
    //     self.token_queue.push_front(token)
    // }

    pub fn err_expected_one_of(
        &mut self,
        msg: &'static str,
        expected_one_of: Vec<TokenKind>,
    ) -> ParseError {
        match self.peek_internal(0) {
            None => self.err_unexpected_eof(),
            Some(got) => ParseError::ExpectedOneOf {
                expected: expected_one_of,
                got_span: got.span,
                got_kind: got.kind,
                msg,
            },
        }
    }

    /// Consume the next token in the stream
    /// Returns None if EOF has been reached
    pub fn next(&mut self) -> Option<Token> {
        // Try to pull token off the queue
        let tok = match self.token_queue.pop_front() {
            // No more tokens in our queue, go to the lexer
            None => self.next_internal(),
            Some(tok) => Some(tok),
        };

        match tok {
            Some(tok) => {
                self.last_consumed_span = tok.span();
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
