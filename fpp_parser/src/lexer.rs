use crate::token::KeywordKind::*;
use crate::token::TokenKind::*;
use crate::token::{Token, TokenKind};
use fpp_core::{BytePos, SourceFile};
use std::str::Chars;

pub struct Lexer<'a> {
    pos: BytePos,
    file: SourceFile,

    indent: u32,
    escaped_identifier: bool,
    token_has_trailing_whitespace: bool,
    token_end_before_whitespace: BytePos,

    len_remaining: usize,
    content: &'a str,

    /// Iterator over chars. Slightly faster than a &str.
    chars: Chars<'a>,
}

const EOF_CHAR: char = '\0';

/** Verify if a character is within the number base system */
#[inline]
fn is_digit_in_base(ch: char, base: i32) -> bool {
    let num = match ch {
        '0'..='9' => (ch as i32) - ('0' as i32),
        'a'..='z' => (ch as i32) - ('a' as i32) + 10,
        'A'..='Z' => (ch as i32) - ('A' as i32) + 10,
        _ => -1,
    };

    0 <= num && num <= base
}

#[inline]
fn is_base_10_digit(ch: char) -> bool {
    match ch {
        '0'..='9' => true,
        _ => false,
    }
}

#[inline]
fn is_identifier_first(c: char) -> bool {
    match c {
        'A'..='Z' | '_' | 'a'..='z' => true,
        _ => false,
    }
}

#[inline]
fn is_identifier_rest(c: char) -> bool {
    match c {
        c if is_identifier_first(c) => true,
        '0'..='9' => true,
        _ => false,
    }
}

impl<'a> Lexer<'a> {
    pub fn new(file: SourceFile, content: &'a str) -> Lexer<'a> {
        let chars = content.chars();
        Lexer {
            pos: 0,
            file,
            indent: 0,
            content,
            chars,
            len_remaining: file.len(),
            escaped_identifier: false,
            token_has_trailing_whitespace: false,
            token_end_before_whitespace: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        loop {
            self.pos += self.pos_within_token();
            self.reset_token();
            let start = self.pos;

            match self.next_token_kind() {
                EOF => return None,
                Whitespace => {}
                kind => {
                    let length = self.token_length();
                    let end = start + length;
                    let (text, real_kind) = match kind {
                        LiteralString => {
                            // Ignore the quotes
                            // TODO(tumbar) Process using the indent
                            (
                                Some(self.content[start + 1..end - 1].to_string()),
                                LiteralString,
                            )
                        }
                        LiteralFloat => (Some(self.content[start..end].to_string()), LiteralFloat),
                        LiteralInt => (Some(self.content[start..end].to_string()), LiteralInt),
                        PreAnnotation => {
                            // Remove the '@' and end at the first newline
                            (
                                Some(self.content[start + 1..end].trim().to_string()),
                                PreAnnotation,
                            )
                        }
                        PostAnnotation => {
                            // Remove the '@<' and end at the first newline
                            (
                                Some(self.content[start + 2..end].trim().to_string()),
                                PostAnnotation,
                            )
                        }
                        Identifier => {
                            if self.escaped_identifier {
                                // Skip over the '$'
                                (Some(self.content[start + 1..end].to_string()), Identifier)
                            } else {
                                // Check if this is a keyword
                                match &self.content[start..end] {
                                    "action" => (None, Keyword(Action)),
                                    "active" => (None, Keyword(Active)),
                                    "activity" => (None, Keyword(Activity)),
                                    "always" => (None, Keyword(Always)),
                                    "array" => (None, Keyword(Array)),
                                    "assert" => (None, Keyword(Assert)),
                                    "async" => (None, Keyword(Async)),
                                    "at" => (None, Keyword(At)),
                                    "base" => (None, Keyword(Base)),
                                    "block" => (None, Keyword(Block)),
                                    "bool" => (None, Keyword(Bool)),
                                    "change" => (None, Keyword(Change)),
                                    "command" => (None, Keyword(Command)),
                                    "component" => (None, Keyword(Component)),
                                    "connections" => (None, Keyword(Connections)),
                                    "constant" => (None, Keyword(Constant)),
                                    "container" => (None, Keyword(Container)),
                                    "cpu" => (None, Keyword(Cpu)),
                                    "default" => (None, Keyword(Default)),
                                    "diagnostic" => (None, Keyword(Diagnostic)),
                                    "do" => (None, Keyword(Do)),
                                    "drop" => (None, Keyword(Drop)),
                                    "else" => (None, Keyword(Else)),
                                    "enter" => (None, Keyword(Enter)),
                                    "entry" => (None, Keyword(Entry)),
                                    "enum" => (None, Keyword(Enum)),
                                    "event" => (None, Keyword(Event)),
                                    "every" => (None, Keyword(Every)),
                                    "exit" => (None, Keyword(Exit)),
                                    "external" => (None, Keyword(External)),
                                    "F32" => (None, Keyword(F32)),
                                    "F64" => (None, Keyword(F64)),
                                    "false" => (None, Keyword(False)),
                                    "fatal" => (None, Keyword(Fatal)),
                                    "format" => (None, Keyword(Format)),
                                    "get" => (None, Keyword(Get)),
                                    "group" => (None, Keyword(Group)),
                                    "guard" => (None, Keyword(Guard)),
                                    "guarded" => (None, Keyword(Guarded)),
                                    "health" => (None, Keyword(Health)),
                                    "high" => (None, Keyword(High)),
                                    "hook" => (None, Keyword(Hook)),
                                    "I16" => (None, Keyword(I16)),
                                    "I32" => (None, Keyword(I32)),
                                    "I64" => (None, Keyword(I64)),
                                    "I8" => (None, Keyword(I8)),
                                    "id" => (None, Keyword(Id)),
                                    "if" => (None, Keyword(If)),
                                    "implements" => (None, Keyword(Implements)),
                                    "import" => (None, Keyword(Import)),
                                    "include" => (None, Keyword(Include)),
                                    "initial" => (None, Keyword(Initial)),
                                    "input" => (None, Keyword(Input)),
                                    "instance" => (None, Keyword(Instance)),
                                    "interface" => (None, Keyword(Interface)),
                                    "internal" => (None, Keyword(Internal)),
                                    "choice" => (None, Keyword(Choice)),
                                    "locate" => (None, Keyword(Locate)),
                                    "low" => (None, Keyword(Low)),
                                    "machine" => (None, Keyword(Machine)),
                                    "match" => (None, Keyword(Match)),
                                    "module" => (None, Keyword(Module)),
                                    "omit" => (None, Keyword(Omit)),
                                    "on" => (None, Keyword(On)),
                                    "opcode" => (None, Keyword(Opcode)),
                                    "orange" => (None, Keyword(Orange)),
                                    "output" => (None, Keyword(Output)),
                                    "packet" => (None, Keyword(Packet)),
                                    "packets" => (None, Keyword(Packets)),
                                    "param" => (None, Keyword(Param)),
                                    "passive" => (None, Keyword(Passive)),
                                    "phase" => (None, Keyword(Phase)),
                                    "port" => (None, Keyword(Port)),
                                    "priority" => (None, Keyword(Priority)),
                                    "product" => (None, Keyword(Product)),
                                    "queue" => (None, Keyword(Queue)),
                                    "queued" => (None, Keyword(Queued)),
                                    "record" => (None, Keyword(Record)),
                                    "recv" => (None, Keyword(Recv)),
                                    "red" => (None, Keyword(Red)),
                                    "ref" => (None, Keyword(Ref)),
                                    "reg" => (None, Keyword(Reg)),
                                    "request" => (None, Keyword(Request)),
                                    "resp" => (None, Keyword(Resp)),
                                    "save" => (None, Keyword(Save)),
                                    "send" => (None, Keyword(Send)),
                                    "serial" => (None, Keyword(Serial)),
                                    "set" => (None, Keyword(Set)),
                                    "severity" => (None, Keyword(Severity)),
                                    "signal" => (None, Keyword(Signal)),
                                    "size" => (None, Keyword(Size)),
                                    "stack" => (None, Keyword(Stack)),
                                    "state" => (None, Keyword(State)),
                                    "string" => (None, Keyword(String_)),
                                    "struct" => (None, Keyword(Struct)),
                                    "sync" => (None, Keyword(Sync)),
                                    "telemetry" => (None, Keyword(Telemetry)),
                                    "text" => (None, Keyword(Text)),
                                    "throttle" => (None, Keyword(Throttle)),
                                    "time" => (None, Keyword(Time)),
                                    "topology" => (None, Keyword(Topology)),
                                    "true" => (None, Keyword(True)),
                                    "type" => (None, Keyword(Type)),
                                    "U16" => (None, Keyword(U16)),
                                    "U32" => (None, Keyword(U32)),
                                    "U64" => (None, Keyword(U64)),
                                    "U8" => (None, Keyword(U8)),
                                    "unmatched" => (None, Keyword(Unmatched)),
                                    "update" => (None, Keyword(Update)),
                                    "warning" => (None, Keyword(Warning)),
                                    "with" => (None, Keyword(With)),
                                    "yellow" => (None, Keyword(Yellow)),
                                    _ => (Some(self.content[start..end].to_string()), Identifier),
                                }
                            }
                        }
                        _ => (None, kind),
                    };

                    return Some(Token::new(real_kind, text, self.file, start, length));
                }
            }
        }
    }

    pub fn next_token_kind(&mut self) -> TokenKind {
        let first_char = match self.bump() {
            Some(c) => c,
            None => return EOF,
        };

        match first_char {
            // Skip whitespace
            ' ' => {
                self.eat_while(|c| c == ' ');
                Whitespace
            }

            // Escaped identifier
            '$' => {
                if is_identifier_first(self.first()) {
                    self.eat_while(is_identifier_rest);
                    self.escaped_identifier = true;
                    Identifier
                } else {
                    Unknown('$')
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

            'A'..='Z' | 'a'..='z' | '_' => {
                self.eat_while(is_identifier_rest);
                Identifier
            }

            // Escaped newline
            '\\' => {
                // Absorb spaces until newline
                self.eat_while(|c| c == ' ');
                if self.first() != '\n' {
                    self.eat_while(|c| c != '\n');
                    self.bump();
                    Error("Non whitespace character illegal after line continuation")
                } else {
                    // Skip the newline
                    self.bump();
                    Whitespace
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
                        LiteralString
                    }

                    // String literal
                    _ => self.eat_string_literal(),
                }
            }

            // Fractional floating literal (or DOT)
            '.' => match self.first() {
                '0'..='9' => self.eat_fraction(),
                _ => Dot,
            },

            // Newline (or line comment)
            '\n' | '#' => {
                if first_char == '#' {
                    self.eat_until('\n' as u8);
                }

                self.eat_newlines();

                // Check the next token
                // Some tokens eat newlines and others don't
                match self.first() {
                    ')' => {
                        self.bump();
                        RightParen
                    }
                    ']' => {
                        self.bump();
                        RightSquare
                    }
                    '}' => {
                        self.bump();
                        RightCurly
                    }

                    _ => Eol,
                }
            }

            // Annotation
            '@' => {
                if self.first() == '<' {
                    self.bump();
                    self.eat_until('\n' as u8);
                    self.eat_newlines();
                    PostAnnotation
                } else {
                    self.eat_until('\n' as u8);
                    self.eat_newlines();
                    PreAnnotation
                }
            }

            '*' => {
                self.eat_newlines();
                Star
            }

            '-' => {
                if self.first() == '>' {
                    self.bump();
                    self.eat_newlines();
                    RightArrow
                } else {
                    self.eat_newlines();
                    Minus
                }
            }

            '+' => {
                self.eat_newlines();
                Plus
            }

            '/' => {
                self.eat_newlines();
                Slash
            }

            '=' => {
                self.eat_newlines();
                Equals
            }

            ';' => {
                self.eat_newlines();
                Semi
            }

            ':' => {
                self.eat_newlines();
                Colon
            }

            ',' => {
                self.eat_newlines();
                Comma
            }

            '(' => {
                self.eat_newlines();
                LeftParen
            }

            '{' => {
                self.eat_newlines();
                LeftCurly
            }

            '[' => LeftSquare,

            ')' => RightParen,
            ']' => RightSquare,
            '}' => RightCurly,

            _ => Unknown(first_char),
        }
    }

    fn eat_newlines(&mut self) {
        self.token_end_before_whitespace = self.pos_within_token();
        self.token_has_trailing_whitespace = true;

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
        self.indent = 0;
        while self.first() == ' ' {
            self.bump();
            self.indent += 1;
        }

        loop {
            match self.bump() {
                // Search for triple quotes "".to_string()"
                Some('\"') => {
                    if self.first() == '\"' && self.second() == '\"' {
                        self.bump_bytes(2);
                        return LiteralString;
                    }
                }

                Some('\\') => {
                    // Skip over escaped character
                    self.bump();
                }

                Some(_) => {}
                None => return Error("unclosed multi-line string literal"),
            }
        }
    }

    fn eat_string_literal(&mut self) -> TokenKind {
        loop {
            match self.bump() {
                // Search for triple quotes "".to_string()"
                Some('\"') => {
                    self.indent = 0;
                    return LiteralString;
                }

                Some('\n') => return Error("unclosed string literal"),

                Some('\\') => {
                    // Skip over escaped character
                    self.bump();
                }

                Some(_) => {}

                None => return Error("unclosed string literal"),
            }
        }
    }

    fn eat_number(&mut self, base: i32) -> TokenKind {
        self.eat_while(|ch| -> bool { is_digit_in_base(ch, base) });

        match self.first() {
            '.' if base == 10 => {
                self.bump();
                self.eat_fraction();
                LiteralFloat
            }

            // Exponential
            'e' | 'E' => self.eat_fraction(),

            _ => LiteralInt,
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
                    Error("invalid literal number")
                } else {
                    LiteralFloat
                }
            }

            (c, _) if is_identifier_first(c) => {
                self.eat_to_next();
                Error("invalid literal number")
            }

            _ => LiteralFloat,
        }
    }

    pub fn as_str(&self) -> &'a str {
        self.chars.as_str()
    }

    /// Peeks the next symbol from the input stream without consuming it.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    pub fn first(&self) -> char {
        // `.next()` optimizes better than `.nth(0)`
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    /// Peeks the second symbol from the input stream without consuming it.
    pub fn second(&self) -> char {
        // `.next()` optimizes better than `.nth(1)`
        let mut iter = self.chars.clone();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    /// Peeks the third symbol from the input stream without consuming it.
    pub fn third(&self) -> char {
        // `.next()` optimizes better than `.nth(1)`
        let mut iter = self.chars.clone();
        iter.next();
        iter.next();
        iter.next().unwrap_or(EOF_CHAR)
    }

    /// Checks if there is nothing more to consume.
    pub fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Returns amount of already consumed symbols.
    pub fn pos_within_token(&self) -> BytePos {
        self.len_remaining - self.chars.as_str().len()
    }

    fn token_length(&self) -> BytePos {
        if self.token_has_trailing_whitespace {
            self.token_end_before_whitespace
        } else {
            self.pos_within_token()
        }
    }

    /// Resets the number of bytes consumed to 0.
    fn reset_token(&mut self) {
        self.len_remaining = self.chars.as_str().len();
        self.token_has_trailing_whitespace = false;
        self.token_end_before_whitespace = 0;
        self.indent = 0;
        self.escaped_identifier = false;
    }

    /// Moves to the next character.
    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        Some(c)
    }

    /// Moves to a substring by a number of bytes.
    fn bump_bytes(&mut self, n: usize) {
        self.chars = self.as_str()[n..].chars();
    }

    /// Eats symbols while predicate returns true or until the end of file is reached.
    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        // It was tried making optimized version of this for e.g. line comments, but
        // LLVM can inline all of this and compile it down to fast iteration over bytes.
        while predicate(self.first()) && !self.is_eof() {
            self.bump();
        }
    }

    fn eat_until(&mut self, byte: u8) {
        self.chars = match memchr::memchr(byte, self.as_str().as_bytes()) {
            Some(index) => self.as_str()[index..].chars(),
            None => "".chars(),
        }
    }
}
