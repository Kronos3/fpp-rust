use fpp_core::{BytePos, Positioned, SourceFile, Span};

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

pub struct Token {
    kind: TokenKind,
    span: Span,
    text: Option<String>,
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

impl Positioned for Token {
    fn span(&self) -> Span {
        self.span
    }
}
