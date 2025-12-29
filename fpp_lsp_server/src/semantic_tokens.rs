use fpp_core::{RawFileLines, RawFilePosition};
use fpp_lsp_parser::{SyntaxKind, SyntaxNode, VisitorResult};
use lsp_types::{SemanticToken, SemanticTokens};

#[derive(Debug, Copy, Clone)]
pub enum SemanticTokenKind {
    Module,
    Topology,
    Component,
    Interface,
    ComponentInstance,
    Constant,
    EnumConstant,
    GraphGroup,
    Port,
    Type,
    FormalParameter,

    StateMachine,
    StateMachineInstance,

    Action,
    Guard,
    Signal,
    State,

    // Other
    Annotation,
    Comment,
    Number,
    String,
    Keyword,
}

#[repr(u32)]
enum SemanticTokenKindRaw {
    Namespace,
    Type,
    Enum,
    Class,
    Interface,
    Struct,
    Parameter,
    Variable,
    EnumMember,
    Function,
    Event,
    Modifier,
    Keyword,
    Comment,
    String,
    Number,
}

#[repr(u32)]
enum SemanticTokenModifierRaw {
    None = 0x0,
    Readonly = 0x1,
    Documentation = 0x2,
}

impl SemanticTokenKind {
    fn token_type(self) -> SemanticTokenKindRaw {
        match self {
            SemanticTokenKind::Module => SemanticTokenKindRaw::Namespace,
            SemanticTokenKind::Topology => SemanticTokenKindRaw::Variable,
            SemanticTokenKind::Component => SemanticTokenKindRaw::Class,
            SemanticTokenKind::Interface => SemanticTokenKindRaw::Interface,
            SemanticTokenKind::ComponentInstance => SemanticTokenKindRaw::Variable,
            SemanticTokenKind::Constant => SemanticTokenKindRaw::Variable,
            SemanticTokenKind::EnumConstant => SemanticTokenKindRaw::EnumMember,
            SemanticTokenKind::GraphGroup => SemanticTokenKindRaw::Namespace,
            SemanticTokenKind::Port => SemanticTokenKindRaw::Function,
            SemanticTokenKind::Type => SemanticTokenKindRaw::Type,
            SemanticTokenKind::FormalParameter => SemanticTokenKindRaw::Parameter,
            SemanticTokenKind::StateMachine => SemanticTokenKindRaw::Type,
            SemanticTokenKind::StateMachineInstance => SemanticTokenKindRaw::Variable,
            SemanticTokenKind::Action => SemanticTokenKindRaw::Function,
            SemanticTokenKind::Guard => SemanticTokenKindRaw::Variable,
            SemanticTokenKind::Signal => SemanticTokenKindRaw::Event,
            SemanticTokenKind::State => SemanticTokenKindRaw::Variable,
            SemanticTokenKind::Annotation => SemanticTokenKindRaw::Comment,
            SemanticTokenKind::Comment => SemanticTokenKindRaw::Comment,
            SemanticTokenKind::Number => SemanticTokenKindRaw::Number,
            SemanticTokenKind::String => SemanticTokenKindRaw::String,
            SemanticTokenKind::Keyword => SemanticTokenKindRaw::Keyword,
        }
    }

    fn token_modifiers(self) -> SemanticTokenModifierRaw {
        match self {
            SemanticTokenKind::Constant => SemanticTokenModifierRaw::Readonly,
            SemanticTokenKind::EnumConstant => SemanticTokenModifierRaw::Readonly,
            SemanticTokenKind::Annotation => SemanticTokenModifierRaw::Documentation,
            _ => SemanticTokenModifierRaw::None,
        }
    }

    pub fn type_and_modifier(self) -> (u32, u32) {
        (self.token_type() as u32, self.token_modifiers() as u32)
    }
}

struct SemanticTokensState {
    lines: RawFileLines,
    tokens: SemanticTokens,
    last: RawFilePosition,
}

impl SemanticTokensState {
    fn new(text: &str) -> SemanticTokensState {
        SemanticTokensState {
            lines: RawFileLines::new(text),
            tokens: Default::default(),
            last: Default::default(),
        }
    }

    fn add(&mut self, node: &SyntaxNode, kind: SemanticTokenKind) {
        let range = node.text_range();
        let start = self.lines.position(range.start().into());
        let end = self.lines.position(range.end().into());

        // We only support single line tokens
        assert_eq!(start.line, end.line);

        let delta_line = start.line - self.last.line;
        let delta_start = if delta_line == 0 {
            // Same line, offset the start position
            start.column - self.last.column
        } else {
            // Token is on a different line, don't alter the column
            start.column
        };

        let (token_type, token_modifiers_bitset) = kind.type_and_modifier();
        let length = end.column - start.column;

        tracing::info!(?start, %delta_line, %delta_start, %length, ?kind, "token");

        self.last = end;
        self.tokens.data.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type,
            token_modifiers_bitset,
        });
    }
}

struct SemanticTokenVisitor {}
impl fpp_lsp_parser::Visitor for SemanticTokenVisitor {
    type State = SemanticTokensState;

    fn visit(&self, state: &mut Self::State, node: &SyntaxNode) -> VisitorResult {
        let kind = match node.kind() {
            SyntaxKind::POST_ANNOTATION => SemanticTokenKind::Annotation,
            SyntaxKind::PRE_ANNOTATION => SemanticTokenKind::Annotation,
            SyntaxKind::LITERAL_FLOAT => SemanticTokenKind::Number,
            SyntaxKind::LITERAL_INT => SemanticTokenKind::Number,
            SyntaxKind::LITERAL_STRING => SemanticTokenKind::String,
            keyword if keyword.is_keyword() => SemanticTokenKind::Keyword,
            SyntaxKind::COMMENT => SemanticTokenKind::Comment,

            SyntaxKind::NAME_REF => {
                if let Some(parent_node_kind) = node.parent().map(|f| f.kind()) {
                    match parent_node_kind {
                        SyntaxKind::FORMAL_PARAM => SemanticTokenKind::FormalParameter,
                        SyntaxKind::TYPE_NAME => SemanticTokenKind::Type,

                        // We do not recognize this name's parent rule
                        _ => return VisitorResult::Next,
                    }
                } else {
                    // Top level names should not exist
                    return VisitorResult::Next;
                }
            }

            // These are typed by definitions above them
            SyntaxKind::NAME => {
                if let Some(parent_node_kind) = node.parent().map(|f| f.kind()) {
                    match parent_node_kind {
                        SyntaxKind::DEF_ABSTRACT_TYPE
                        | SyntaxKind::DEF_ALIAS_TYPE
                        | SyntaxKind::DEF_ARRAY
                        | SyntaxKind::DEF_ENUM
                        | SyntaxKind::DEF_STRUCT => SemanticTokenKind::Type,
                        SyntaxKind::DEF_COMPONENT => SemanticTokenKind::Component,
                        SyntaxKind::DEF_COMPONENT_INSTANCE => SemanticTokenKind::ComponentInstance,
                        SyntaxKind::DEF_ENUM_CONSTANT => SemanticTokenKind::EnumConstant,
                        SyntaxKind::DEF_CONSTANT => SemanticTokenKind::Constant,
                        SyntaxKind::DEF_INTERFACE => SemanticTokenKind::Interface,
                        SyntaxKind::DEF_TOPOLOGY => SemanticTokenKind::Topology,
                        SyntaxKind::SPEC_CONNECTION_GRAPH_DIRECT => SemanticTokenKind::GraphGroup,
                        SyntaxKind::DEF_MODULE => SemanticTokenKind::Module,
                        SyntaxKind::DEF_PORT => SemanticTokenKind::Port,
                        SyntaxKind::DEF_ACTION => SemanticTokenKind::Action,
                        SyntaxKind::DEF_GUARD => SemanticTokenKind::Guard,
                        SyntaxKind::DEF_SIGNAL => SemanticTokenKind::Signal,
                        SyntaxKind::DEF_STATE => SemanticTokenKind::State,
                        SyntaxKind::DEF_STATE_MACHINE => SemanticTokenKind::StateMachine,
                        // We do not recognize this name's parent rule
                        _ => return VisitorResult::Next,
                    }
                } else {
                    // Top level names should not exist
                    return VisitorResult::Next;
                }
            }

            // Keep going deeper to look at children
            _ => return VisitorResult::Recurse,
        };

        state.add(node, kind);
        VisitorResult::Next
    }
}

pub(crate) fn compute(text: &str, parse: &fpp_lsp_parser::Parse) -> SemanticTokens {
    let mut state = SemanticTokensState::new(text);
    parse.visit(&mut state, &SemanticTokenVisitor {});

    state.tokens
}
