use crate::cursor::Cursor;
use crate::error::{ParseError, ParseResult};
use crate::token::{KeywordKind, Token, TokenKind};
use fpp_ast::*;
use fpp_core::{Positioned, SourceFile};
use std::str::Chars;

struct Parser<'a> {
    cursor: Cursor<'a>,
}

enum ElementParsingResult<T> {
    Terminated(T),
    Unterminated(T),
    Err(ParseError),
    None,
}

impl<'a> Parser<'a> {
    pub fn new(source_file: SourceFile, chars: Chars<'a>) -> Parser<'a> {
        Parser {
            cursor: Cursor::new(source_file, chars),
        }
    }

    pub(crate) fn peek(&mut self, n: usize) -> TokenKind {
        self.cursor.peek(n)
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        self.cursor.next()
    }

    pub(crate) fn consume(&mut self, kind: TokenKind) -> ParseResult<Token> {
        self.cursor.consume(kind)
    }

    #[inline]
    pub(crate) fn node<T>(&self, data: T, first_token: fpp_core::Span) -> AstNode<T> {
        let last_token_span = self
            .cursor
            .last_token_span()
            .expect("last token should exist");

        AstNode {
            id: fpp_core::NodeId::new(fpp_core::Span::new(
                first_token.file(),
                first_token.start().pos(),
                last_token_span.end().pos() - first_token.start().pos(),
            )),
            data,
        }
    }

    pub(crate) fn ident(&mut self) -> ParseResult<Ident> {
        let ident = self.consume(TokenKind::Identifier)?;
        Ok(self.node(ident.text().to_string(), ident.span()))
    }

    fn alias_type(&mut self) -> ParseResult<AstNode<DefAliasType>> {
        let first = self.consume_keyword(KeywordKind::Type)?;
        self.consume(TokenKind::Equals)?;
        let name = self.ident()?;
        let type_name = self.type_name()?;

        Ok(self.node(DefAliasType { name, type_name }, first.span()))
    }

    fn abs_type(&mut self) -> ParseResult<AstNode<DefAbsType>> {
        let first = self.consume_keyword(KeywordKind::Type)?;
        let name = self.ident()?;

        Ok(self.node(DefAbsType { name }, first.span()))
    }

    fn def_action(&mut self) -> ParseResult<AstNode<DefAction>> {
        let first = self.consume_keyword(KeywordKind::Action)?;
        let name = self.ident()?;
        let type_name = match self.peek(0) {
            TokenKind::Colon => {
                self.next();
                Some(self.type_name()?)
            }
            _ => None,
        };

        Ok(self.node(DefAction { name, type_name }, first.span()))
    }

    fn def_array(&mut self) -> ParseResult<AstNode<DefArray>> {
        let first = self.consume_keyword(KeywordKind::Array)?;
        let name = self.ident()?;

        self.consume(TokenKind::Equals)?;

        let size = self.index()?;
        let elt_type = self.type_name()?;

        let default = match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Default) => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        let format = match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Format) => {
                self.next();
                Some(self.lit_string()?)
            }
            _ => None,
        };

        Ok(self.node(
            DefArray {
                name,
                size,
                elt_type,
                default,
                format,
            },
            first.span(),
        ))
    }

    fn def_choice(&mut self) -> ParseResult<AstNode<DefChoice>> {
        let first = self.consume_keyword(KeywordKind::Choice)?;
        let name = self.ident()?;

        self.consume(TokenKind::LeftCurly)?;

        self.consume_keyword(KeywordKind::If)?;
        let guard = self.ident()?;
        let if_transition = self.transition_expr()?;

        self.consume_keyword(KeywordKind::Else)?;
        let else_transition = self.transition_expr()?;

        self.consume(TokenKind::RightCurly)?;

        Ok(self.node(
            DefChoice {
                name,
                guard,
                if_transition,
                else_transition,
            },
            first.span(),
        ))
    }

    fn component(&mut self) -> ParseResult<AstNode<DefComponent>> {
        let (kind, first) = self.component_kind()?;
        self.consume_keyword(KeywordKind::Component)?;
        let name = self.ident()?;

        self.consume(TokenKind::LeftCurly)?;
        let members = self.annotated_element_sequence(
            &Parser::component_member,
            TokenKind::Semi,
            TokenKind::RightCurly,
        )?;
        self.consume(TokenKind::RightCurly)?;

        Ok(self.node(
            DefComponent {
                kind,
                name,
                members,
            },
            first.span(),
        ))
    }

    fn component_kind(&mut self) -> ParseResult<(ComponentKind, Token)> {
        let ck = match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Active) => Ok(ComponentKind::Active),
            TokenKind::Keyword(KeywordKind::Passive) => Ok(ComponentKind::Passive),
            TokenKind::Keyword(KeywordKind::Queued) => Ok(ComponentKind::Queued),
            _ => Err(self.cursor.err_expected_one_of(
                "component kind expected",
                vec![
                    TokenKind::Keyword(KeywordKind::Active),
                    TokenKind::Keyword(KeywordKind::Passive),
                    TokenKind::Keyword(KeywordKind::Queued),
                ],
            )),
        }?;

        Ok((ck, self.next().unwrap()))
    }

    fn component_member(&mut self) -> ParseResult<ComponentMember> {
        match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Type) => match self.peek(0) {
                TokenKind::Equals => Ok(ComponentMember::DefAliasType(self.alias_type()?)),
                _ => Ok(ComponentMember::DefAbsType(self.abs_type()?)),
            },
            TokenKind::Keyword(KeywordKind::Array) => {
                Ok(ComponentMember::DefArray(self.def_array()?))
            }
            TokenKind::Keyword(KeywordKind::Constant) => {
                Ok(ComponentMember::DefConstant(self.def_constant()?))
            }
            TokenKind::Keyword(KeywordKind::Enum) => Ok(ComponentMember::DefEnum(self.def_enum()?)),
            TokenKind::Keyword(KeywordKind::State) => {
                if self.peek(2) == TokenKind::Keyword(KeywordKind::Instance) {
                    Ok(ComponentMember::SpecStateMachineInstance(
                        self.spec_state_machine_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::DefStateMachine(self.def_state_machine()?))
                }
            }
            TokenKind::Keyword(KeywordKind::Struct) => {
                Ok(ComponentMember::DefStruct(self.def_struct()?))
            }
            TokenKind::Keyword(KeywordKind::Async | KeywordKind::Guarded | KeywordKind::Sync) => {
                if self.peek(1) == TokenKind::Keyword(KeywordKind::Command) {
                    Ok(ComponentMember::SpecCommand(self.spec_command()?))
                } else {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                }
            }
            TokenKind::Keyword(KeywordKind::Command | KeywordKind::Text | KeywordKind::Time) => {
                // Special command port
                Ok(ComponentMember::SpecPortInstance(
                    self.spec_port_instance()?,
                ))
            }
            TokenKind::Keyword(KeywordKind::Product) => {
                if self.peek(1) == TokenKind::Keyword(KeywordKind::Container) {
                    Ok(ComponentMember::SpecContainer(self.spec_container()?))
                } else if self.peek(1) == TokenKind::Keyword(KeywordKind::Record) {
                    Ok(ComponentMember::SpecRecord(self.spec_record()?))
                } else {
                    // Special port kind
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                }
            }
            TokenKind::Keyword(KeywordKind::Event) => {
                if self.peek(1) == TokenKind::Keyword(KeywordKind::Port) {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::SpecEvent(self.spec_event()?))
                }
            }
            TokenKind::Keyword(KeywordKind::Include) => {
                Ok(ComponentMember::SpecInclude(self.spec_include()?))
            }
            TokenKind::Keyword(KeywordKind::Internal) => Ok(ComponentMember::SpecInternalPort(
                self.spec_internal_port()?,
            )),
            TokenKind::Keyword(KeywordKind::Match) => Ok(ComponentMember::SpecPortMatching(
                self.spec_port_matching()?,
            )),
            TokenKind::Keyword(KeywordKind::External) => {
                Ok(ComponentMember::SpecParam(self.spec_param()?))
            }
            TokenKind::Keyword(KeywordKind::Param) => {
                if self.peek(1) == TokenKind::Keyword(KeywordKind::Port) {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::SpecParam(self.spec_param()?))
                }
            }
            TokenKind::Keyword(KeywordKind::Telemetry) => {
                if self.peek(1) == TokenKind::Keyword(KeywordKind::Port) {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::SpecTlmChannel(self.spec_tlm_channel()?))
                }
            }
            TokenKind::Keyword(KeywordKind::Import) => Ok(ComponentMember::SpecImportInterface(
                self.spec_import_interface()?,
            )),
            _ => Err(self
                .cursor
                .err_expected_one_of("component member expected", vec![])),
        }
    }

    fn def_component_instance(&mut self) -> ParseResult<AstNode<DefComponentInstance>> {
        let first = self.consume_keyword(KeywordKind::Instance)?;
        let name = self.ident()?;
        self.consume(TokenKind::Colon)?;
        let component = self.qual_ident()?;

        self.consume_keyword(KeywordKind::Base)?;
        self.consume_keyword(KeywordKind::Id)?;
        let base_id = self.expr()?;

        let impl_type = {
            match self.peek(0) {
                TokenKind::Keyword(KeywordKind::Type) => {
                    self.next();
                    Some(self.lit_string()?)
                }
                _ => None,
            }
        };

        let file = {
            match self.peek(0) {
                TokenKind::Keyword(KeywordKind::At) => {
                    self.next();
                    Some(self.lit_string()?)
                }
                _ => None,
            }
        };

        let queue_size = {
            match self.peek(0) {
                TokenKind::Keyword(KeywordKind::Queue) => {
                    self.next();
                    self.consume_keyword(KeywordKind::Size)?;
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let stack_size = {
            match self.peek(0) {
                TokenKind::Keyword(KeywordKind::Stack) => {
                    self.next();
                    self.consume_keyword(KeywordKind::Size)?;
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let priority = {
            match self.peek(0) {
                TokenKind::Keyword(KeywordKind::Priority) => {
                    self.next();
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let cpu = {
            match self.peek(0) {
                TokenKind::Keyword(KeywordKind::Cpu) => {
                    self.next();
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let init_specs = {
            match self.peek(0) {
                TokenKind::LeftCurly => {
                    self.next();
                    let seq = self.annotated_element_sequence(
                        &Parser::spec_init,
                        TokenKind::Semi,
                        TokenKind::RightCurly,
                    )?;
                    self.consume(TokenKind::RightCurly)?;
                    seq
                }
                _ => vec![],
            }
        };

        Ok(self.node(
            DefComponentInstance {
                name,
                component,
                base_id,
                impl_type,
                file,
                queue_size,
                stack_size,
                priority,
                cpu,
                init_specs,
            },
            first.span(),
        ))
    }

    fn spec_init(&mut self) -> ParseResult<AstNode<SpecInit>> {
        let first = self.consume_keyword(KeywordKind::Phase)?;
        let phase = self.expr()?;
        let code = self.lit_string()?;

        Ok(self.node(SpecInit { phase, code }, first.span()))
    }

    fn def_constant(&mut self) -> ParseResult<AstNode<DefConstant>> {
        let first = self.consume_keyword(KeywordKind::Constant)?;
        let name = self.ident()?;

        self.consume(TokenKind::Equals)?;
        let value = self.expr()?;

        Ok(self.node(DefConstant { name, value }, first.span()))
    }

    fn def_enum(&mut self) -> ParseResult<AstNode<DefEnum>> {
        let first = self.consume_keyword(KeywordKind::Enum)?;
        let name = self.ident()?;

        let type_name = match self.peek(0) {
            TokenKind::Colon => {
                self.next();
                Some(self.type_name()?)
            }
            _ => None,
        };

        self.consume(TokenKind::LeftCurly)?;
        let constants = self.annotated_element_sequence(
            &Parser::def_enum_constant,
            TokenKind::Comma,
            TokenKind::RightCurly,
        )?;
        self.consume(TokenKind::RightCurly)?;

        let default = match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Default) => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        Ok(self.node(
            DefEnum {
                name,
                type_name,
                constants,
                default,
            },
            first.span(),
        ))
    }

    fn def_enum_constant(&mut self) -> ParseResult<AstNode<DefEnumConstant>> {
        let name = self.ident()?;
        let first_span = name.span();

        let value = match self.peek(0) {
            TokenKind::Equals => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        Ok(self.node(DefEnumConstant { name, value }, first_span))
    }

    fn def_state_machine(&mut self) -> ParseResult<AstNode<DefStateMachine>> {
        let first = self.consume_keyword(KeywordKind::State)?;
        self.consume_keyword(KeywordKind::Machine)?;

        let name = self.ident()?;
        let members = match self.peek(0) {
            TokenKind::LeftCurly => {
                self.next();
                let out = self.annotated_element_sequence(
                    &Parser::state_machine_member,
                    TokenKind::Semi,
                    TokenKind::RightCurly,
                )?;

                self.consume(TokenKind::RightCurly)?;

                Some(out)
            }
            _ => None,
        };

        Ok(self.node(DefStateMachine { name, members }, first.span()))
    }

    fn state_machine_member(&mut self) -> ParseResult<StateMachineMember> {
        match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Initial) => Ok(
                StateMachineMember::SpecInitialTransition(self.spec_initial_transition()?),
            ),
            TokenKind::Keyword(KeywordKind::State) => {
                Ok(StateMachineMember::DefState(self.def_state()?))
            }
            TokenKind::Keyword(KeywordKind::Signal) => {
                Ok(StateMachineMember::DefSignal(self.def_signal()?))
            }
            TokenKind::Keyword(KeywordKind::Action) => {
                Ok(StateMachineMember::DefSignal(self.def_action()?))
            }
            TokenKind::Keyword(KeywordKind::Guard) => {
                Ok(StateMachineMember::DefGuard(self.def_guard()?))
            }
            TokenKind::Keyword(KeywordKind::Choice) => {
                Ok(StateMachineMember::DefChoice(self.def_choice()?))
            }
            _ => Err(self.cursor.err_expected_one_of(
                "state machine member expected",
                vec![
                    TokenKind::Keyword(KeywordKind::Initial),
                    TokenKind::Keyword(KeywordKind::State),
                    TokenKind::Keyword(KeywordKind::Signal),
                    TokenKind::Keyword(KeywordKind::Action),
                    TokenKind::Keyword(KeywordKind::Guard),
                    TokenKind::Keyword(KeywordKind::Choice),
                ],
            )),
        }
    }

    fn state_member(&mut self) -> ParseResult<StateMember> {
        match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Choice) => {
                Ok(StateMember::DefChoice(self.def_choice()?))
            }
            TokenKind::Keyword(KeywordKind::State) => Ok(StateMember::DefState(self.def_state()?)),
            TokenKind::Keyword(KeywordKind::Initial) => Ok(StateMember::SpecInitialTransition(
                self.spec_initial_transition()?,
            )),
            TokenKind::Keyword(KeywordKind::Entry) => {
                Ok(StateMember::SpecStateEntry(self.spec_state_entry()?))
            }
            TokenKind::Keyword(KeywordKind::Exit) => {
                Ok(StateMember::SpecStateExit(self.spec_state_exit()?))
            }
            TokenKind::Keyword(KeywordKind::On) => Ok(StateMember::SpecStateTransition(
                self.spec_state_transition()?,
            )),
            _ => Err(self.cursor.err_expected_one_of(
                "state member expected",
                vec![
                    TokenKind::Keyword(KeywordKind::Choice),
                    TokenKind::Keyword(KeywordKind::State),
                    TokenKind::Keyword(KeywordKind::Initial),
                    TokenKind::Keyword(KeywordKind::Entry),
                    TokenKind::Keyword(KeywordKind::Exit),
                    TokenKind::Keyword(KeywordKind::On),
                ],
            )),
        }
    }

    fn spec_initial_transition(&mut self) -> ParseResult<AstNode<SpecInitialTransition>> {
        let first = self.consume_keyword(KeywordKind::Initial)?;
        let transition = self.transition_expr()?;

        Ok(self.node(SpecInitialTransition { transition }, first.span()))
    }

    fn spec_state_entry(&mut self) -> ParseResult<AstNode<SpecStateEntry>> {
        let first = self.consume_keyword(KeywordKind::Entry)?;
        let actions = self.do_expr()?;

        Ok(self.node(SpecStateEntry { actions }, first.span()))
    }

    fn do_expr(&mut self) -> ParseResult<AstNode<DoExpr>> {
        let first = self.consume_keyword(KeywordKind::Do)?;
        self.consume(TokenKind::LeftCurly)?;
        let elts =
            self.element_sequence(&Parser::ident, TokenKind::Comma, TokenKind::RightCurly)?;
        self.consume(TokenKind::RightCurly)?;

        Ok(self.node(DoExpr(elts), first.span()))
    }

    fn def_struct(&mut self) -> ParseResult<AstNode<DefStruct>> {
        let first = self.consume_keyword(KeywordKind::Struct)?;
        let name = self.ident()?;

        self.consume(TokenKind::LeftCurly)?;
        let members = self.annotated_element_sequence(
            &Parser::struct_type_member,
            TokenKind::Comma,
            TokenKind::RightCurly,
        )?;
        self.consume(TokenKind::RightCurly)?;

        let default = match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Default) => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        Ok(self.node(
            DefStruct {
                name,
                members,
                default,
            },
            first.span(),
        ))
    }

    fn struct_type_member(&mut self) -> ParseResult<AstNode<StructTypeMember>> {
        let name = self.ident()?;
        let name_span = name.span();

        self.consume(TokenKind::Colon)?;
        let size = match self.peek(0) {
            TokenKind::LeftSquare => Some(self.index()?),
            _ => None,
        };

        let type_name = self.type_name()?;
        let format = match self.peek(0) {
            TokenKind::Keyword(KeywordKind::Format) => Some(self.lit_string()?),
            _ => None,
        };

        Ok(self.node(
            StructTypeMember {
                name,
                size,
                type_name,
                format,
            },
            name_span,
        ))
    }

    fn pre_annotation(&mut self) -> Vec<String> {
        let mut out: Vec<String> = vec![];

        while self.peek(0) == TokenKind::PreAnnotation {
            out.push(
                self.consume(TokenKind::PreAnnotation)
                    .unwrap()
                    .text()
                    .to_string(),
            )
        }

        out
    }

    fn post_annotation(&mut self) -> Vec<String> {
        let mut out: Vec<String> = vec![];

        while self.peek(0) == TokenKind::PostAnnotation {
            out.push(
                self.consume(TokenKind::PostAnnotation)
                    .unwrap()
                    .text()
                    .to_string(),
            )
        }

        out
    }

    fn annotated_element<T>(
        &mut self,
        element_parser: &dyn Fn(&mut Parser<'a>) -> ParseResult<T>,
        punct: &TokenKind,
        end: &TokenKind,
    ) -> ElementParsingResult<Annotated<T>> {
        let pre_annotation = self.pre_annotation();

        // Check if we reached the end
        if self.peek(0) == *end {
            // Stop parsing elements
            return ElementParsingResult::None;
        }

        let data = match element_parser(self) {
            Ok(data) => data,
            Err(err) => return ElementParsingResult::Err(err),
        };

        // Check if the punctuation exists
        let punct_tok = self.peek(0);
        if punct_tok == *punct || punct_tok == TokenKind::Eol {
            self.next();
            let post_annotation = self.post_annotation();
            ElementParsingResult::Terminated(Annotated {
                pre_annotation,
                data,
                post_annotation,
            })
        } else if self.peek(0) == TokenKind::PostAnnotation {
            let post_annotation = self.post_annotation();
            ElementParsingResult::Terminated(Annotated {
                pre_annotation,
                data,
                post_annotation,
            })
        } else {
            ElementParsingResult::Unterminated(Annotated {
                pre_annotation,
                data,
                post_annotation: vec![],
            })
        }
    }

    #[inline]
    fn annotated_element_sequence<T>(
        &mut self,
        element_parser: &dyn Fn(&mut Parser<'a>) -> ParseResult<T>,
        punct: TokenKind,
        end: TokenKind,
    ) -> ParseResult<Vec<Annotated<T>>> {
        // Eat up all the EOLs
        while self.peek(0) == TokenKind::Eol {
            self.next();
        }

        // Keep reading terminated elements until we can't
        let mut out: Vec<Annotated<T>> = vec![];

        loop {
            match self.annotated_element(element_parser, &punct, &end) {
                ElementParsingResult::Terminated(el) => {
                    out.push(el);
                }
                ElementParsingResult::Unterminated(el) => {
                    out.push(el);
                    break;
                }
                ElementParsingResult::None => {
                    break;
                }
                ElementParsingResult::Err(err) => {
                    // TODO(tumbar) We can recover from this by pulling tokens until the next element
                    //   This is needed for the language server
                    return Err(err);
                }
            }
        }

        Ok(out)
    }

    fn element<T>(
        &mut self,
        element_parser: &dyn Fn(&mut Parser<'a>) -> ParseResult<T>,
        punct: &TokenKind,
        end: &TokenKind,
    ) -> ElementParsingResult<T> {
        // Check if we reached the end
        if self.peek(0) == *end {
            // Stop parsing elements
            return ElementParsingResult::None;
        }

        let data = match element_parser(self) {
            Ok(data) => data,
            Err(err) => return ElementParsingResult::Err(err),
        };

        // Check if the punctuation exists
        let punct_tok = self.peek(0);
        if punct_tok == *punct || punct_tok == TokenKind::Eol {
            self.next();
            ElementParsingResult::Terminated(data)
        } else if self.peek(0) == TokenKind::PostAnnotation {
            ElementParsingResult::Terminated(data)
        } else {
            ElementParsingResult::Unterminated(data)
        }
    }

    #[inline]
    fn element_sequence<T>(
        &mut self,
        element_parser: &dyn Fn(&mut Parser<'a>) -> ParseResult<T>,
        punct: TokenKind,
        end: TokenKind,
    ) -> ParseResult<Vec<T>> {
        // Eat up all the EOLs
        while self.peek(0) == TokenKind::Eol {
            self.next();
        }

        // Keep reading terminated elements until we can't
        let mut out: Vec<T> = vec![];

        loop {
            match self.element(&element_parser, &punct, &end) {
                ElementParsingResult::Terminated(el) => {
                    out.push(el);
                }
                ElementParsingResult::Unterminated(el) => {
                    out.push(el);
                    break;
                }
                ElementParsingResult::None => {
                    break;
                }
                ElementParsingResult::Err(err) => {
                    // TODO(tumbar) We can recover from this by pulling tokens until the next element
                    //   This is needed for the language server
                    return Err(err);
                }
            }
        }

        Ok(out)
    }

    fn index(&mut self) -> ParseResult<AstNode<Expr>> {
        self.consume(TokenKind::LeftSquare)?;
        let out = self.expr()?;
        self.consume(TokenKind::RightSquare)?;
        Ok(out)
    }

    fn qual_ident(&mut self) -> ParseResult<AstNode<QualIdent>> {
        Err(ParseError::NotImplemented {})
    }

    fn type_name(&mut self) -> ParseResult<AstNode<TypeName>> {
        Err(ParseError::NotImplemented {})
    }

    fn expr(&mut self) -> ParseResult<AstNode<Expr>> {
        Err(ParseError::NotImplemented {})
    }

    fn lit_string(&mut self) -> ParseResult<AstNode<String>> {
        Err(ParseError::NotImplemented {})
    }

    fn transition_expr(&mut self) -> ParseResult<AstNode<TransitionExpr>> {
        Err(ParseError::NotImplemented {})
    }

    /// Convenience function to consume a keyword
    #[inline]
    pub fn consume_keyword(&mut self, kind: KeywordKind) -> ParseResult<Token> {
        self.consume(TokenKind::Keyword(kind))
    }

    /// Check to make sure a token at a certain position (number of tokens) away matches
    /// an expected token kind
    pub fn check(&mut self, n: usize, kind: TokenKind) -> ParseResult<()> {
        let p = self.peek(n);

        if kind == p {
            Ok(())
        } else {
            Err(self.err_expected_token("unexpected token", kind, p))
        }
    }
}
