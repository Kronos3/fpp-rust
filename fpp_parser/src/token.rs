use fpp_core::{BytePos, SourceFile, Span, Spanned};
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum KeywordKind {
    Action,
    Active,
    Activity,
    Always,
    Array,
    Assert,
    Async,
    At,
    Base,
    Block,
    Bool,
    Change,
    Command,
    Component,
    Connections,
    Constant,
    Container,
    Cpu,
    Default,
    Diagnostic,
    Do,
    Drop,
    Else,
    Enter,
    Entry,
    Enum,
    Event,
    Every,
    Exit,
    External,
    F32,
    F64,
    False,
    Fatal,
    Format,
    Get,
    Group,
    Guard,
    Guarded,
    Health,
    High,
    Hook,
    I16,
    I32,
    I64,
    I8,
    Id,
    If,
    Implements,
    Import,
    Include,
    Initial,
    Input,
    Instance,
    Interface,
    Internal,
    Choice,
    Locate,
    Low,
    Machine,
    Match,
    Module,
    Omit,
    On,
    Opcode,
    Orange,
    Output,
    Packet,
    Packets,
    Param,
    Passive,
    Phase,
    Port,
    Priority,
    Product,
    Queue,
    Queued,
    Record,
    Recv,
    Red,
    Ref,
    Reg,
    Request,
    Resp,
    Save,
    Send,
    Serial,
    Set,
    Severity,
    Signal,
    Size,
    Stack,
    State,
    String_,
    Struct,
    Sync,
    Telemetry,
    Text,
    Throttle,
    Time,
    Topology,
    True,
    Type,
    U16,
    U32,
    U64,
    U8,
    Unmatched,
    Update,
    Warning,
    With,
    Yellow,
}

impl std::fmt::Display for KeywordKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TokenKind {
    EOF,
    Whitespace,
    Unknown(char),
    Error(&'static str),

    Identifier,

    // Annotations,
    PostAnnotation,
    PreAnnotation,

    // Literals,
    LiteralFloat,
    LiteralInt,
    LiteralString,

    // Keywords,
    Keyword(KeywordKind),

    // Symbols,
    Colon,
    Comma,
    Dot,
    Eol,
    Equals,

    LeftParen,
    LeftCurly,
    LeftSquare,

    RightParen,
    RightCurly,
    RightSquare,

    RightArrow,
    Minus,
    Plus,
    Semi,
    Slash,
    Star,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TokenKind::EOF => "eof",
            TokenKind::Whitespace => "<whitespace>",
            TokenKind::Unknown(c) => {
                return f.write_str(&format!("{:#?}", c));
            }
            TokenKind::Error(err) => *err,
            TokenKind::Identifier => "identifier",
            TokenKind::PostAnnotation => "post-annotation",
            TokenKind::PreAnnotation => "pre-annotation",
            TokenKind::LiteralFloat => "literal float",
            TokenKind::LiteralInt => "literal integer",
            TokenKind::LiteralString => "literal string",
            TokenKind::Keyword(keyword) => return std::fmt::Display::fmt(&keyword, f),
            TokenKind::Colon => ":",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",
            TokenKind::Eol => "end of line",
            TokenKind::Equals => "=",
            TokenKind::LeftParen => "(",
            TokenKind::LeftCurly => "{",
            TokenKind::LeftSquare => "[",
            TokenKind::RightParen => ")",
            TokenKind::RightCurly => "}",
            TokenKind::RightSquare => "]",
            TokenKind::RightArrow => "->",
            TokenKind::Minus => "-",
            TokenKind::Plus => "+",
            TokenKind::Semi => ";",
            TokenKind::Slash => "/",
            TokenKind::Star => "*",
        };

        f.write_str(s)
    }
}

#[derive(Debug)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: Option<String>,
}

impl Token {
    pub fn new(
        kind: TokenKind,
        text: Option<String>,
        file: SourceFile,
        start: BytePos,
        length: BytePos,
    ) -> Token {
        Token {
            kind,
            span: Span::new(file, start, length),
            text,
        }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn text(&self) -> &str {
        match self.text.as_ref() {
            None => "",
            Some(txt) => txt,
        }
    }
}

impl Spanned for Token {
    fn span(&self) -> Span {
        self.span
    }
}
