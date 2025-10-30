use crate::cursor::Cursor;
use crate::error::{ParseError, ParseResult};
use crate::token::{KeywordKind, Token, TokenKind};
use fpp_ast::*;

struct Parser<'a> {
    cursor: Cursor<'a>,
}

enum AnnotationElementResult<T> {
    Terminated(T),
    Unterminated(T),
    Err(ParseError),
    None,
}

impl Parser {
    pub(crate) fn peek(&mut self, n: usize) -> TokenKind {
        self.cursor.peek(n)
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        self.cursor.next()
    }

    pub(crate) fn consume(&mut self, kind: TokenKind) -> ParseResult<Token> {
        self.consume(kind)
    }

    #[inline]
    pub(crate) fn node<T>(&self, data: T) -> AstNode<T> {
        AstNode {
            // TODO(tumbar) Track the location of this node
            id: 0,
            data,
        }
    }

    pub(crate) fn ident(&mut self) -> ParseResult<Ident> {
        let ident = self.consume(TokenKind::Identifier)?;
        Ok(self.node(ident.text().to_string()))
    }

    fn alias_type(&mut self) -> ParseResult<AstNode<DefAliasType>> {
        self.consume_keyword(KeywordKind::Type)?;
        self.consume(TokenKind::Equals)?;
        let name = self.ident()?;
        let type_name = self.type_name()?;

        Ok(self.node(DefAliasType { name, type_name }))
    }

    fn abs_type(&mut self) -> ParseResult<AstNode<DefAbsType>> {
        self.consume_keyword(KeywordKind::Type)?;
        let name = self.ident()?;

        Ok(self.node(DefAbsType { name }))
    }

    fn def_action(&mut self) -> ParseResult<DefAction> {
        self.consume_keyword(KeywordKind::Action)?;
        let name = self.ident()?;
        let type_name = match self.peek(0) {
            TokenKind::Colon => {
                self.next();
                Some(self.type_name()?)
            }
            _ => None,
        };

        Ok(DefAction { name, type_name })
    }

    fn def_array(&mut self) -> ParseResult<AstNode<DefArray>> {
        self.consume_keyword(KeywordKind::Array)?;
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

        Ok(self.node(DefArray {
            name,
            size,
            elt_type,
            default,
            format,
        }))
    }

    fn def_choice(&mut self) -> ParseResult<DefChoice> {
        self.consume_keyword(KeywordKind::Choice)?;
        let name = self.ident()?;

        self.consume(TokenKind::LeftCurly)?;

        self.consume_keyword(KeywordKind::If)?;
        let guard = self.node(self.ident()?);
        let if_transition = self.node(self.transition_expr()?);

        self.consume_keyword(KeywordKind::Else)?;
        let else_transition = self.node(self.transition_expr()?);

        self.consume(TokenKind::RightCurly)?;

        Ok(DefChoice {
            name,
            guard,
            if_transition,
            else_transition,
        })
    }

    fn component(&mut self) -> ParseResult<DefComponent> {
        let kind = self.component_kind()?;
        self.consume_keyword(KeywordKind::Component)?;
        let name = self.ident()?;

        self.consume(TokenKind::LeftCurly)?;
        let members = self.annotated_element_sequence(
            || self.component_member(),
            TokenKind::Semi,
            TokenKind::RightCurly,
        )?;
        self.consume(TokenKind::RightCurly)?;

        Ok(DefComponent {
            kind,
            name,
            members,
        })
    }

    fn component_kind(&mut self) -> ParseResult<ComponentKind> {
        match self.peek(0) {
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
        }
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

    fn def_component_instance(&mut self) -> ParseResult<DefComponentInstance> {
        self.consume_keyword(KeywordKind::Instance)?;
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
                        || self.spec_init(),
                        TokenKind::Semi,
                        TokenKind::RightCurly,
                    )?;
                    self.consume(TokenKind::RightCurly)?;
                    seq
                }
                _ => vec![],
            }
        };

        Ok(DefComponentInstance {
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
        })
    }

    fn spec_init(&mut self) -> ParseResult<AstNode<SpecInit>> {
        self.consume_keyword(KeywordKind::Phase)?;
        let phase = self.expr()?;
        let code = self.lit_string()?;

        Ok(self.node(SpecInit { phase, code }))
    }

    fn def_constant(&mut self) -> ParseResult<AstNode<DefConstant>> {
        self.consume_keyword(KeywordKind::Constant)?;
        let name = self.ident()?;

        self.consume(TokenKind::Equals)?;
        let value = self.expr()?;

        Ok(self.node(DefConstant { name, value }))
    }

    fn def_enum(&mut self) -> ParseResult<AstNode<DefEnum>> {
        self.consume_keyword(KeywordKind::Enum)?;
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
            || self.def_enum_constant(),
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

        Ok(self.node(DefEnum {
            name,
            type_name,
            constants,
            default,
        }))
    }

    fn def_enum_constant(&mut self) -> ParseResult<AstNode<DefEnumConstant>> {
        let name = self.ident()?;
        let value = match self.peek(0) {
            TokenKind::Equals=> {
                self.next();
                Some(self.expr()?)
            }
            _ => None
        };

        Ok(self.node(DefEnumConstant {
            name,
            value,
        }))
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
        element_parser: impl Fn() -> ParseResult<T>,
        punct: &TokenKind,
        end: &TokenKind,
    ) -> AnnotationElementResult<Annotated<T>> {
        let pre_annotation = self.pre_annotation();

        // Check if we reached the end
        if self.peek(0) == *end {
            // Stop parsing elements
            return AnnotationElementResult::None;
        }

        let data = element_parser()?;

        // Check if the punctuation exists
        let punct_tok = self.peek(0);
        if punct_tok == *punct || punct_tok == TokenKind::Eol {
            self.next();
            let post_annotation = self.post_annotation();
            AnnotationElementResult::Terminated(Annotated {
                pre_annotation,
                data,
                post_annotation,
            })
        } else if self.peek(0) == TokenKind::PostAnnotation {
            let post_annotation = self.post_annotation();
            AnnotationElementResult::Terminated(Annotated {
                pre_annotation,
                data,
                post_annotation,
            })
        } else {
            AnnotationElementResult::Unterminated(Annotated {
                pre_annotation,
                data,
                post_annotation: vec![],
            })
        }
    }

    #[inline]
    fn annotated_element_sequence<T>(
        &mut self,
        element_parser: impl Fn() -> ParseResult<T>,
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
            match self.annotated_element(&element_parser, &punct, &end) {
                AnnotationElementResult::Terminated(el) => {
                    out.push(el);
                }
                AnnotationElementResult::Unterminated(el) => {
                    out.push(el);
                    break;
                }
                AnnotationElementResult::None => {
                    break;
                }
                AnnotationElementResult::Err(err) => {
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

    fn qual_ident(&mut self) -> ParseResult<AstNode<QualIdent>> {}

    fn type_name(&mut self) -> ParseResult<AstNode<TypeName>> {}

    fn expr(&mut self) -> ParseResult<AstNode<Expr>> {}

    fn lit_string(&mut self) -> ParseResult<AstNode<String>> {}

    fn transition_expr(&mut self) -> ParseResult<TransitionExpr> {}

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
