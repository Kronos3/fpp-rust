use crate::lexer::Lexer;
use crate::token::TokenKind;

/** Verify if a character is within the number base system */
fn is_digit_in_base(ch: char, base: i32) -> bool {
    let num = match ch {
        '0'..='9' => (ch as i32) - ('0' as i32),
        'a'..='z' => (ch as i32) - ('a' as i32) + 10,
        'A'..='Z' => (ch as i32) - ('A' as i32) + 10,
        _ => -1,
    };

    0 <= num && num <= base
}

fn is_base_10_digit(ch: char) -> bool {
    match ch {
        '0'..='9' => true,
        _ => false,
    }
}

fn is_identifier_first(c: char) -> bool {
    match c {
        'A'..='Z' | '_' | 'a'..='z' => true,
        _ => false,
    }
}

fn is_identifier_rest(c: char) -> bool {
    match c {
        c if is_identifier_first(c) => true,
        '0'..='9' => true,
        _ => false,
    }
}

impl<'a> Lexer<'a> {
    pub fn next_token_kind(&mut self) -> TokenKind {
        let first_char = match self.bump() {
            Some(c) => c,
            None => return TokenKind::EOF,
        };

        match first_char {
            // Skip whitespace
            ' ' => {
                self.eat_while(|c| c == ' ');
                TokenKind::Whitespace
            }

            // Escaped identifier
            '$' => {
                if is_identifier_first(self.first()) {
                    self.eat_while(is_identifier_rest);
                    TokenKind::Identifier {
                        force_identifier: true,
                    }
                } else {
                    TokenKind::Unknown('$')
                }
            }

            // Numeric literal
            '0' => {
                let base = match self.first() {
                    'x' | 'X' => {
                        self.bump();
                        16
                    }
                    _ => 10,
                };

                self.eat_number(base)
            }

            // More numeric literals
            '1'..='9' => self.eat_number(10),

            // Escaped newline
            '\\' => {
                self.bump();

                // Absorb spaces until newline
                self.eat_while(|c| c == ' ');
                if self.first() != '\n' {
                    self.eat_while(|c| c != '\n');
                    self.bump();
                    TokenKind::Error("Non whitespace character illegal after line continuation")
                } else {
                    // Skip the newline
                    self.bump();
                    TokenKind::Whitespace
                }
            }

            // String literals
            '\"' => {
                match (self.first(), self.second()) {
                    // Multi-line string literal
                    ('\"', '\"') => {
                        // Skip the initial newline if it exists
                        if self.third() == '\n' {
                            self.bump_bytes(3);
                        } else {
                            self.bump_bytes(2);
                        }

                        self.eat_multiline_string_literal()
                    }

                    // Empty string literal
                    ('\"', _) => {
                        self.bump();
                        TokenKind::LiteralString {
                            multi_line_indent: 0,
                        }
                    }

                    // String literal
                    _ => self.eat_string_literal(),
                }
            }

            // Fractional floating literal (or DOT)
            '.' => match self.first() {
                '0'..='9' => self.eat_fraction(),
                _ => TokenKind::Dot,
            },

            // Newline (or line comment)
            '\n' | '#' => {
                self.eat_newlines();

                // Check the next token
                // Some tokens eat newlines and others don't
                match self.first() {
                    ')' => {
                        self.bump();
                        TokenKind::RightParen
                    }
                    ']' => {
                        self.bump();
                        TokenKind::RightSquare
                    }
                    '}' => {
                        self.bump();
                        TokenKind::RightCurly
                    }

                    _ => TokenKind::Eol,
                }
            }

            // Annotation
            '@' => {
                if self.first() == '<' {
                    self.bump();
                    self.eat_until('\n' as u8);
                    self.eat_newlines();
                    TokenKind::PostAnnotation
                } else {
                    self.eat_until('\n' as u8);
                    self.eat_newlines();
                    TokenKind::PreAnnotation
                }
            }

            '*' => {
                self.eat_newlines();
                TokenKind::Star
            }

            '-' => {
                if self.first() == '>' {
                    self.bump();
                    self.eat_newlines();
                    TokenKind::RightArrow
                } else {
                    self.eat_newlines();
                    TokenKind::Minus
                }
            }

            '+' => {
                self.eat_newlines();
                TokenKind::Plus
            }

            '/' => {
                self.eat_newlines();
                TokenKind::Slash
            }

            '=' => {
                self.eat_newlines();
                TokenKind::Equals
            }

            ';' => {
                self.eat_newlines();
                TokenKind::Semi
            }

            ':' => {
                self.eat_newlines();
                TokenKind::Colon
            }

            '(' => {
                self.eat_newlines();
                TokenKind::LeftParen
            }

            '{' => {
                self.eat_newlines();
                TokenKind::LeftCurly
            }

            '[' => TokenKind::LeftSquare,

            ')' => TokenKind::RightParen,
            ']' => TokenKind::RightSquare,
            '}' => TokenKind::RightCurly,

            _ => TokenKind::Unknown(first_char),
        }
    }

    fn eat_newlines(&mut self) {
        loop {
            match self.first() {
                '\n' | ' ' => {
                    self.bump();
                }

                '#' => {
                    // Eat line comment
                    self.eat_until('\n' as u8);
                }

                _ => return,
            }
        }
    }

    fn eat_multiline_string_literal(&mut self) -> TokenKind {
        // Count the indent on the first line
        let mut indent = 0;
        while self.first() == ' ' {
            self.bump();
            indent += 1;
        }

        loop {
            match self.bump() {
                // Search for triple quotes """
                Some('\"') => {
                    if self.first() == '\"' && self.second() == '\"' {
                        self.bump_bytes(2);
                        return TokenKind::LiteralString {
                            multi_line_indent: indent,
                        };
                    }
                }

                Some('\\') => {
                    // Skip over escaped character
                    self.bump();
                }

                Some(_) => {}
                None => return TokenKind::Error("Unclosed multi-line string literal"),
            }
        }
    }

    fn eat_string_literal(&mut self) -> TokenKind {
        loop {
            match self.bump() {
                // Search for triple quotes """
                Some('\"') => {
                    return TokenKind::LiteralString {
                        multi_line_indent: -1,
                    };
                }

                Some('\n') => return TokenKind::Error("Unclosed line string literal"),

                Some('\\') => {
                    // Skip over escaped character
                    self.bump();
                }

                Some(_) => {}

                None => return TokenKind::Error("Unclosed multi-line string literal"),
            }
        }
    }

    fn eat_number(&mut self, base: i32) -> TokenKind {
        self.eat_while(|ch| -> bool { is_digit_in_base(ch, base) });

        match self.first() {
            '.' if base == 10 => {
                self.bump();
                self.eat_fraction();
                TokenKind::LiteralFloat
            }

            // Exponential
            'e' | 'E' => self.eat_fraction(),

            _ => TokenKind::LiteralInt,
        }
    }

    fn eat_to_next(&mut self) {
        self.eat_while(is_identifier_rest)
    }

    fn eat_fraction(&mut self) -> TokenKind {
        self.eat_while(is_base_10_digit);

        match (self.first(), self.second()) {
            ('e' | 'E', '0'..='9') => {
                self.bump();
                self.eat_while(is_base_10_digit);
                if is_identifier_first(self.first()) {
                    self.eat_to_next();
                    TokenKind::Error("invalid literal number")
                } else {
                    TokenKind::LiteralFloat
                }
            }

            (c, _) if is_identifier_first(c) => {
                self.eat_to_next();
                TokenKind::Error("invalid literal number")
            }

            _ => TokenKind::LiteralFloat,
        }
    }
}
