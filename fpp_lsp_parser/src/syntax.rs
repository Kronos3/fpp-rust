#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types)]
#[repr(u16)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,

    IDENT,
    POST_ANNOTATION,
    PRE_ANNOTATION,

    LITERAL_FLOAT,
    LITERAL_INT,
    LITERAL_STRING,

    // Keywords
    ACTION_KW,
    ACTIVE_KW,
    ACTIVITY_KW,
    ALWAYS_KW,
    ARRAY_KW,
    ASSERT_KW,
    ASYNC_KW,
    AT_KW,
    BASE_KW,
    BLOCK_KW,
    BOOL_KW,
    CHANGE_KW,
    COMMAND_KW,
    COMPONENT_KW,
    CONNECTIONS_KW,
    CONSTANT_KW,
    CONTAINER_KW,
    CPU_KW,
    DEFAULT_KW,
    DIAGNOSTIC_KW,
    DO_KW,
    DROP_KW,
    ELSE_KW,
    ENTER_KW,
    ENTRY_KW,
    ENUM_KW,
    EVENT_KW,
    EVERY_KW,
    EXIT_KW,
    EXTERNAL_KW,
    F32_KW,
    F64_KW,
    FALSE_KW,
    FATAL_KW,
    FORMAT_KW,
    GET_KW,
    GROUP_KW,
    GUARD_KW,
    GUARDED_KW,
    HEALTH_KW,
    HIGH_KW,
    HOOK_KW,
    I16_KW,
    I32_KW,
    I64_KW,
    I8_KW,
    ID_KW,
    IF_KW,
    IMPLEMENTS_KW,
    IMPORT_KW,
    INCLUDE_KW,
    INITIAL_KW,
    INPUT_KW,
    INSTANCE_KW,
    INTERFACE_KW,
    INTERNAL_KW,
    CHOICE_KW,
    LOCATE_KW,
    LOW_KW,
    MACHINE_KW,
    MATCH_KW,
    MODULE_KW,
    OMIT_KW,
    ON_KW,
    OPCODE_KW,
    ORANGE_KW,
    OUTPUT_KW,
    PACKET_KW,
    PACKETS_KW,
    PARAM_KW,
    PASSIVE_KW,
    PHASE_KW,
    PORT_KW,
    PRIORITY_KW,
    PRODUCT_KW,
    QUEUE_KW,
    QUEUED_KW,
    RECORD_KW,
    RECV_KW,
    RED_KW,
    REF_KW,
    REG_KW,
    REQUEST_KW,
    RESP_KW,
    SAVE_KW,
    SEND_KW,
    SERIAL_KW,
    SET_KW,
    SEVERITY_KW,
    SIGNAL_KW,
    SIZE_KW,
    STACK_KW,
    STATE_KW,
    STRING_KW,
    STRUCT_KW,
    SYNC_KW,
    TELEMETRY_KW,
    TEXT_KW,
    THROTTLE_KW,
    TIME_KW,
    TOPOLOGY_KW,
    TRUE_KW,
    TYPE_KW,
    U16_KW,
    U32_KW,
    U64_KW,
    U8_KW,
    UNMATCHED_KW,
    UPDATE_KW,
    WARNING_KW,
    WITH_KW,
    YELLOW_KW,

    // Symbols
    COLON,
    COMMA,
    DOT,
    EOL,
    EQUALS,

    LEFT_PAREN,
    LEFT_CURLY,
    LEFT_SQUARE,

    RIGHT_PAREN,
    RIGHT_CURLY,
    RIGHT_SQUARE,

    RIGHT_ARROW,
    MINUS,
    PLUS,
    SEMI,
    SLASH,
    STAR,

    // Special
    WHITESPACE,
    COMMENT,
    ERROR,
    UNKNOWN,

    // Definitions
    DEF_ALIAS_TYPE,
    DEF_ABSTRACT_TYPE,
    DEF_ARRAY,
    DEF_STRUCT,
    DEF_ENUM,
    DEF_ENUM_CONSTANT,
    DEF_CONSTANT,
    DEF_COMPONENT,
    DEF_MODULE,
    DEF_PORT,
    DEF_INTERFACE,
    DEF_COMPONENT_INSTANCE,

    DEF_STATE_MACHINE,
    DEF_STATE,
    DEF_SIGNAL,
    DEF_ACTION,
    DEF_GUARD,
    DEF_CHOICE,

    // Specifiers
    SPEC_COMMAND,
    SPEC_PORT_INSTANCE_GENERAL,
    SPEC_PORT_INSTANCE_SPECIAL,
    SPEC_CONTAINER,
    SPEC_RECORD,
    SPEC_EVENT,
    SPEC_TELEMETRY,
    SPEC_INTERFACE_IMPORT,
    SPEC_STATE_MACHINE_INSTANCE,
    SPEC_INITIAL_TRANSITION,
    SPEC_STATE_ENTRY,
    SPEC_STATE_EXIT,
    SPEC_STATE_TRANSITION,
    SPEC_LOC,
    SPEC_INIT,

    // Helper nodes
    NAME,
    NAME_REF,
    TYPE_NAME,
    SIZE,
    DEFAULT,
    FORMAT,
    OPCODE,
    PRIORITY,
    STRUCT_MEMBER_LIST,
    STRUCT_MEMBER,
    ENUM_MEMBER_LIST,
    FORMAL_PARAM_LIST,
    FORMAL_PARAM,
    QUEUE_FULL,
    ID,
    DEFAULT_PRIORITY,
    EVENT_THROTTLE,
    EVERY,
    SET_OPCODE,
    SAVE_OPCODE,
    LIMIT_SEQUENCE,
    LIMIT,
    MODULE_MEMBER_LIST,
    STATE_MACHINE_MEMBER_LIST,
    STATE_MEMBER_LIST,
    COMPONENT_MEMBER_LIST,
    INTERFACE_MEMBER_LIST,
    THEN_CLAUSE,
    ELSE_CLAUSE,
    DO_EXPR,
    TRANSITION_OR_DO,
    TRANSITION_EXPR,
    BASE_ID,
    COMPONENT_INSTANCE_TYPE,
    QUEUE_SIZE,
    STACK_SIZE,
    CPU,
    INIT_SPEC_LIST,

    ROOT,

    #[doc(hidden)]
    __LAST,
}

use SyntaxKind::*;

impl From<u16> for SyntaxKind {
    #[inline]
    fn from(d: u16) -> SyntaxKind {
        assert!(d <= (SyntaxKind::__LAST as u16));
        unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
    }
}

/// Some boilerplate is needed, as rowan settled on using its own
/// `struct SyntaxKind(u16)` internally, instead of accepting the
/// user's `enum SyntaxKind` as a type parameter.
///
/// First, to easily pass the enum variants into rowan via `.into()`:
impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

/// Second, implementing the `Language` trait teaches rowan to convert between
/// these two SyntaxKind types, allowing for a nicer SyntaxNode API where
/// "kinds" are values from our `enum SyntaxKind`, instead of plain u16 values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FppLang {}
impl rowan::Language for FppLang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}
