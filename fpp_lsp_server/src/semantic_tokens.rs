use fpp_core::{RawFileLines, RawFilePosition};
use fpp_lsp_parser::{NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, VisitorResult};
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens};

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
    PortInstance,
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
    Class,
    Interface,
    Struct,
    Parameter,
    Variable,
    EnumMember,
    Event,
    Function,
    Modifier,
    Keyword,
    Comment,
    String,
    Number,
}

#[repr(u32)]
enum SemanticTokenModifierRaw {
    None = 0x0,
    Readonly = 1 << 1,
    Documentation = 1 << 2,
}

impl SemanticTokenKind {
    pub const TOKEN_TYPES: [SemanticTokenType; 15] = [
        SemanticTokenType::NAMESPACE,
        SemanticTokenType::TYPE,
        SemanticTokenType::CLASS,
        SemanticTokenType::INTERFACE,
        SemanticTokenType::STRUCT,
        SemanticTokenType::PARAMETER,
        SemanticTokenType::VARIABLE,
        SemanticTokenType::ENUM_MEMBER,
        SemanticTokenType::EVENT,
        SemanticTokenType::FUNCTION,
        SemanticTokenType::MODIFIER,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::COMMENT,
        SemanticTokenType::STRING,
        SemanticTokenType::NUMBER,
    ];

    pub const TOKEN_MODIFIERS: [SemanticTokenModifier; 2] = [
        SemanticTokenModifier::READONLY,
        SemanticTokenModifier::DOCUMENTATION,
    ];

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
            SemanticTokenKind::Port => SemanticTokenKindRaw::Class,
            SemanticTokenKind::PortInstance => SemanticTokenKindRaw::Function,
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

    fn token_modifiers(self) -> u32 {
        match self {
            SemanticTokenKind::Constant => SemanticTokenModifierRaw::Readonly as u32,
            SemanticTokenKind::EnumConstant => SemanticTokenModifierRaw::Readonly as u32,
            SemanticTokenKind::Annotation => SemanticTokenModifierRaw::Documentation as u32,
            _ => SemanticTokenModifierRaw::None as u32,
        }
    }

    pub fn type_and_modifier(self) -> (u32, u32) {
        (self.token_type() as u32, self.token_modifiers() as u32)
    }
}

pub(crate) struct SemanticTokensState {
    lines: RawFileLines,
    raw: Vec<(TextRange, SemanticTokenKind)>,
}

impl SemanticTokensState {
    fn new(text: &str) -> SemanticTokensState {
        SemanticTokensState {
            lines: RawFileLines::new(text),
            raw: Default::default(),
        }
    }

    pub(crate) fn finish(mut self, filter_range: Option<lsp_types::Range>) -> SemanticTokens {
        let mut tokens = SemanticTokens::default();
        self.raw.sort_by(|a, b| a.0.ordering(b.0));

        let mut last: RawFilePosition = RawFilePosition {
            pos: 0,
            line: 0,
            column: 0,
        };

        let filter_range = match filter_range {
            Some(filter_range) => Some(TextRange::new(
                self.lines
                    .position_of(filter_range.start.line, filter_range.start.character)
                    .into(),
                self.lines
                    .position_of(filter_range.start.line, filter_range.start.character)
                    .into(),
            )),
            None => None,
        };

        for (range, kind) in self.raw {
            match filter_range {
                Some(filter_range) => {
                    if filter_range.intersect(range).is_none() {
                        continue;
                    }
                }
                None => {}
            }

            // TODO(tumbar) This can be heavily optimized
            let start = self.lines.position(range.start().into());
            let end = self.lines.position(range.end().into());

            // We only support single line tokens
            assert_eq!(start.line, end.line);

            let delta_line = start.line - last.line;
            let delta_start = if delta_line == 0 {
                // Same line, offset the start position
                start.column - last.column
            } else {
                // Token is on a different line, don't alter the column
                start.column
            };

            let (token_type, token_modifiers_bitset) = kind.type_and_modifier();
            let length = end.column - start.column;

            last = start;
            tokens.data.push(SemanticToken {
                delta_line,
                delta_start,
                length,
                token_type,
                token_modifiers_bitset,
            });
        }

        tokens
    }

    #[inline]
    fn add_text_range(&mut self, range: TextRange, kind: SemanticTokenKind) {
        self.raw.push((range, kind));
    }

    fn add_token(&mut self, token: &SyntaxToken, kind: SemanticTokenKind) {
        self.add_text_range(token.text_range(), kind);
    }

    fn add_node(&mut self, node: &SyntaxNode, kind: SemanticTokenKind) {
        self.add_text_range(node.text_range(), kind);
    }
}

struct SemanticTokenVisitor {}
impl fpp_lsp_parser::Visitor for SemanticTokenVisitor {
    type State = SemanticTokensState;

    fn visit_token(&self, state: &mut Self::State, token: &SyntaxToken) {
        let kind = match token.kind() {
            pk if pk.is_type_primitive_keyword() => SemanticTokenKind::Type,
            SyntaxKind::POST_ANNOTATION | SyntaxKind::PRE_ANNOTATION => {
                SemanticTokenKind::Annotation
            }
            SyntaxKind::LITERAL_FLOAT | SyntaxKind::LITERAL_INT => SemanticTokenKind::Number,
            SyntaxKind::LITERAL_STRING => SemanticTokenKind::String,
            _ => return,
        };

        state.add_token(token, kind);
    }

    fn visit_node(&self, state: &mut Self::State, node: &SyntaxNode) -> VisitorResult {
        match node.kind() {
            SyntaxKind::EXPR => {
                // TODO(tumbar) Use analysis information
                for token in node.descendants_with_tokens() {
                    if let NodeOrToken::Token(token) = token {
                        if token.kind() == SyntaxKind::IDENT {
                            state.add_token(&token, SemanticTokenKind::Constant);
                        }
                    }
                }

                VisitorResult::Next
            }

            SyntaxKind::PORT_INSTANCE_IDENTIFIER => {
                // TODO(tumbar) Use analysis information
                let ident_list = node.descendants_with_tokens().filter_map(|f| match f {
                    NodeOrToken::Token(token) if token.kind() == SyntaxKind::IDENT => Some(token),
                    _ => None,
                });

                for ident in ident_list {
                    state.add_token(&ident, SemanticTokenKind::ComponentInstance);
                }

                VisitorResult::Next
            }
            SyntaxKind::QUAL_IDENT => {
                // TODO(tumbar) Use analysis information
                let ident_list = node.descendants_with_tokens().filter_map(|f| match f {
                    NodeOrToken::Token(token) if token.kind() == SyntaxKind::IDENT => Some(token),
                    _ => None,
                });

                if let Some(parent_node_kind) = node.parent().map(|f| f.kind()) {
                    let name_kind = match parent_node_kind {
                        SyntaxKind::TYPE_NAME => SemanticTokenKind::Type,
                        SyntaxKind::SPEC_PORT_INSTANCE_GENERAL => SemanticTokenKind::Port,
                        SyntaxKind::DEF_COMPONENT_INSTANCE => SemanticTokenKind::Component,
                        SyntaxKind::SPEC_INSTANCE => SemanticTokenKind::ComponentInstance,
                        SyntaxKind::SPEC_CONNECTION_GRAPH_PATTERN => {
                            SemanticTokenKind::ComponentInstance
                        }
                        _ => return VisitorResult::Next,
                    };

                    for ident in ident_list {
                        state.add_token(&ident, name_kind);
                    }
                }

                VisitorResult::Next
            }
            // These are typed by definitions above them
            SyntaxKind::NAME => {
                if let Some(parent_node_kind) = node.parent().map(|f| f.kind()) {
                    let name_kind = match parent_node_kind {
                        SyntaxKind::DEF_ABSTRACT_TYPE
                        | SyntaxKind::DEF_ALIAS_TYPE
                        | SyntaxKind::DEF_ARRAY
                        | SyntaxKind::DEF_ENUM
                        | SyntaxKind::DEF_STRUCT => SemanticTokenKind::Type,
                        SyntaxKind::FORMAL_PARAM => SemanticTokenKind::FormalParameter,
                        SyntaxKind::DEF_COMPONENT => SemanticTokenKind::Component,
                        SyntaxKind::DEF_COMPONENT_INSTANCE => SemanticTokenKind::ComponentInstance,
                        SyntaxKind::DEF_ENUM_CONSTANT => SemanticTokenKind::EnumConstant,
                        SyntaxKind::DEF_CONSTANT => SemanticTokenKind::Constant,
                        SyntaxKind::DEF_INTERFACE => SemanticTokenKind::Interface,
                        SyntaxKind::DEF_TOPOLOGY => SemanticTokenKind::Topology,
                        SyntaxKind::SPEC_CONNECTION_GRAPH_DIRECT => SemanticTokenKind::GraphGroup,
                        SyntaxKind::SPEC_PORT_INSTANCE_GENERAL => SemanticTokenKind::PortInstance,
                        SyntaxKind::SPEC_PORT_INSTANCE_SPECIAL => SemanticTokenKind::PortInstance,
                        SyntaxKind::SPEC_PORT_INSTANCE_INTERNAL => SemanticTokenKind::PortInstance,
                        SyntaxKind::DEF_MODULE => SemanticTokenKind::Module,
                        SyntaxKind::DEF_PORT => SemanticTokenKind::Port,
                        SyntaxKind::DEF_ACTION => SemanticTokenKind::Action,
                        SyntaxKind::DEF_GUARD => SemanticTokenKind::Guard,
                        SyntaxKind::DEF_SIGNAL => SemanticTokenKind::Signal,
                        SyntaxKind::DEF_STATE => SemanticTokenKind::State,
                        SyntaxKind::DEF_STATE_MACHINE => SemanticTokenKind::StateMachine,
                        // We do not recognize this name's parent rule
                        _ => return VisitorResult::Next,
                    };

                    state.add_node(node, name_kind);
                }

                VisitorResult::Next
            }

            // Keep going deeper to look at children
            _ => VisitorResult::Recurse,
        }
    }
}

pub(crate) fn compute(text: &str, parse: &fpp_lsp_parser::Parse) -> SemanticTokensState {
    let mut state = SemanticTokensState::new(text);
    parse.visit(&mut state, &SemanticTokenVisitor {});
    state
}
