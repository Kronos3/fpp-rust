use crate::cursor::Cursor;
use crate::error::{ParseError, ParseResult};
use crate::token::{KeywordKind, Token, TokenKind};
use fpp_ast::*;
use fpp_core::{Positioned, SourceFile};
use std::str::Chars;

use crate::token::KeywordKind::*;
use crate::token::TokenKind::*;
pub use std::string::String;

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

    pub(crate) fn peek_span(&mut self, n: usize) -> Option<fpp_core::Span> {
        self.cursor.peek_span(n)
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
        let ident = self.consume(Identifier)?;
        Ok(self.node(ident.text().to_string(), ident.span()))
    }

    fn alias_type(&mut self) -> ParseResult<AstNode<DefAliasType>> {
        let first = self.consume_keyword(Type)?;
        self.consume(Equals)?;
        let name = self.ident()?;
        let type_name = self.type_name()?;

        Ok(self.node(DefAliasType { name, type_name }, first.span()))
    }

    fn abs_type(&mut self) -> ParseResult<AstNode<DefAbsType>> {
        let first = self.consume_keyword(Type)?;
        let name = self.ident()?;

        Ok(self.node(DefAbsType { name }, first.span()))
    }

    fn def_action(&mut self) -> ParseResult<AstNode<DefAction>> {
        let first = self.consume_keyword(Action)?;
        let name = self.ident()?;
        let type_name = match self.peek(0) {
            Colon => {
                self.next();
                Some(self.type_name()?)
            }
            _ => None,
        };

        Ok(self.node(DefAction { name, type_name }, first.span()))
    }

    fn def_array(&mut self) -> ParseResult<AstNode<DefArray>> {
        let first = self.consume_keyword(Array)?;
        let name = self.ident()?;

        self.consume(Equals)?;

        let size = self.index()?;
        let elt_type = self.type_name()?;

        let default = match self.peek(0) {
            Keyword(Default) => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        let format = match self.peek(0) {
            Keyword(Format) => {
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
        let first = self.consume_keyword(Choice)?;
        let name = self.ident()?;

        self.consume(LeftCurly)?;

        self.consume_keyword(If)?;
        let guard = self.ident()?;
        let if_transition = self.transition_expr()?;

        self.consume_keyword(Else)?;
        let else_transition = self.transition_expr()?;

        self.consume(RightCurly)?;

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
        self.consume_keyword(Component)?;
        let name = self.ident()?;

        self.consume(LeftCurly)?;
        let members =
            self.annotated_element_sequence(&Parser::component_member, Semi, RightCurly)?;
        self.consume(RightCurly)?;

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
            Keyword(Active) => Ok(ComponentKind::Active),
            Keyword(Passive) => Ok(ComponentKind::Passive),
            Keyword(Queued) => Ok(ComponentKind::Queued),
            _ => Err(self.cursor.err_expected_one_of(
                "component kind expected",
                vec![Keyword(Active), Keyword(Passive), Keyword(Queued)],
            )),
        }?;

        Ok((ck, self.next().unwrap()))
    }

    fn component_member(&mut self) -> ParseResult<ComponentMember> {
        match self.peek(0) {
            Keyword(Type) => match self.peek(0) {
                Equals => Ok(ComponentMember::DefAliasType(self.alias_type()?)),
                _ => Ok(ComponentMember::DefAbsType(self.abs_type()?)),
            },
            Keyword(Array) => Ok(ComponentMember::DefArray(self.def_array()?)),
            Keyword(Constant) => Ok(ComponentMember::DefConstant(self.def_constant()?)),
            Keyword(Enum) => Ok(ComponentMember::DefEnum(self.def_enum()?)),
            Keyword(State) => {
                if self.peek(2) == Keyword(Instance) {
                    Ok(ComponentMember::SpecStateMachineInstance(
                        self.spec_state_machine_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::DefStateMachine(self.def_state_machine()?))
                }
            }
            Keyword(Struct) => Ok(ComponentMember::DefStruct(self.def_struct()?)),
            Keyword(Async | Guarded | Sync) => {
                if self.peek(1) == Keyword(Command) {
                    Ok(ComponentMember::SpecCommand(self.spec_command()?))
                } else {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                }
            }
            Keyword(Command | Text | Time) => {
                // Special command port
                Ok(ComponentMember::SpecPortInstance(
                    self.spec_port_instance()?,
                ))
            }
            Keyword(Product) => {
                if self.peek(1) == Keyword(Container) {
                    Ok(ComponentMember::SpecContainer(self.spec_container()?))
                } else if self.peek(1) == Keyword(Record) {
                    Ok(ComponentMember::SpecRecord(self.spec_record()?))
                } else {
                    // Special port kind
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                }
            }
            Keyword(Event) => {
                if self.peek(1) == Keyword(Port) {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::SpecEvent(self.spec_event()?))
                }
            }
            Keyword(Include) => Ok(ComponentMember::SpecInclude(self.spec_include()?)),
            Keyword(Internal) => Ok(ComponentMember::SpecInternalPort(
                self.spec_internal_port()?,
            )),
            Keyword(Match) => Ok(ComponentMember::SpecPortMatching(
                self.spec_port_matching()?,
            )),
            Keyword(External) => Ok(ComponentMember::SpecParam(self.spec_param()?)),
            Keyword(Param) => {
                if self.peek(1) == Keyword(Port) {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::SpecParam(self.spec_param()?))
                }
            }
            Keyword(Telemetry) => {
                if self.peek(1) == Keyword(Port) {
                    Ok(ComponentMember::SpecPortInstance(
                        self.spec_port_instance()?,
                    ))
                } else {
                    Ok(ComponentMember::SpecTlmChannel(self.spec_tlm_channel()?))
                }
            }
            Keyword(Import) => Ok(ComponentMember::SpecImportInterface(
                self.spec_import_interface()?,
            )),
            _ => Err(self
                .cursor
                .err_expected_one_of("component member expected", vec![])),
        }
    }

    fn def_component_instance(&mut self) -> ParseResult<AstNode<DefComponentInstance>> {
        let first = self.consume_keyword(Instance)?;
        let name = self.ident()?;
        self.consume(Colon)?;
        let component = self.qual_ident()?;

        self.consume_keyword(Base)?;
        self.consume_keyword(Id)?;
        let base_id = self.expr()?;

        let impl_type = {
            match self.peek(0) {
                Keyword(Type) => {
                    self.next();
                    Some(self.lit_string()?)
                }
                _ => None,
            }
        };

        let file = {
            match self.peek(0) {
                Keyword(At) => {
                    self.next();
                    Some(self.lit_string()?)
                }
                _ => None,
            }
        };

        let queue_size = {
            match self.peek(0) {
                Keyword(Queue) => {
                    self.next();
                    self.consume_keyword(Size)?;
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let stack_size = {
            match self.peek(0) {
                Keyword(Stack) => {
                    self.next();
                    self.consume_keyword(Size)?;
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let priority = {
            match self.peek(0) {
                Keyword(Priority) => {
                    self.next();
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let cpu = {
            match self.peek(0) {
                Keyword(Cpu) => {
                    self.next();
                    Some(self.expr()?)
                }
                _ => None,
            }
        };

        let init_specs = {
            match self.peek(0) {
                LeftCurly => {
                    self.next();
                    let seq =
                        self.annotated_element_sequence(&Parser::spec_init, Semi, RightCurly)?;
                    self.consume(RightCurly)?;
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
        let first = self.consume_keyword(Phase)?;
        let phase = self.expr()?;
        let code = self.lit_string()?;

        Ok(self.node(SpecInit { phase, code }, first.span()))
    }

    fn def_constant(&mut self) -> ParseResult<AstNode<DefConstant>> {
        let first = self.consume_keyword(Constant)?;
        let name = self.ident()?;

        self.consume(Equals)?;
        let value = self.expr()?;

        Ok(self.node(DefConstant { name, value }, first.span()))
    }

    fn def_enum(&mut self) -> ParseResult<AstNode<DefEnum>> {
        let first = self.consume_keyword(Enum)?;
        let name = self.ident()?;

        let type_name = match self.peek(0) {
            Colon => {
                self.next();
                Some(self.type_name()?)
            }
            _ => None,
        };

        self.consume(LeftCurly)?;
        let constants =
            self.annotated_element_sequence(&Parser::def_enum_constant, Comma, RightCurly)?;
        self.consume(RightCurly)?;

        let default = match self.peek(0) {
            Keyword(Default) => {
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
            Equals => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        Ok(self.node(DefEnumConstant { name, value }, first_span))
    }

    fn spec_state_machine_instance(&mut self) -> ParseResult<AstNode<SpecStateMachineInstance>> {
        let first = self.consume_keyword(State)?;
        self.consume_keyword(Machine)?;
        self.consume_keyword(Instance)?;

        let name = self.ident()?;

        self.consume(Colon)?;
        let state_machine = self.qual_ident()?;

        let priority = match self.peek(0) {
            Keyword(Priority) => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        let queue_full = match self.peek(0) {
            Keyword(Assert) => {
                self.next();
                Some(QueueFull::Assert)
            }
            Keyword(Block) => {
                self.next();
                Some(QueueFull::Block)
            }
            Keyword(Drop) => {
                self.next();
                Some(QueueFull::Drop)
            }
            Keyword(Hook) => {
                self.next();
                Some(QueueFull::Hook)
            }
            _ => None,
        };

        Ok(self.node(
            SpecStateMachineInstance {
                name,
                state_machine,
                priority,
                queue_full,
            },
            first.span(),
        ))
    }

    fn def_state_machine(&mut self) -> ParseResult<AstNode<DefStateMachine>> {
        let first = self.consume_keyword(State)?;
        self.consume_keyword(Machine)?;

        let name = self.ident()?;
        let members = match self.peek(0) {
            LeftCurly => {
                self.next();
                let out = self.annotated_element_sequence(
                    &Parser::state_machine_member,
                    Semi,
                    RightCurly,
                )?;

                self.consume(RightCurly)?;

                Some(out)
            }
            _ => None,
        };

        Ok(self.node(DefStateMachine { name, members }, first.span()))
    }

    fn state_machine_member(&mut self) -> ParseResult<StateMachineMember> {
        match self.peek(0) {
            Keyword(Initial) => Ok(StateMachineMember::SpecInitialTransition(
                self.spec_initial_transition()?,
            )),
            Keyword(State) => Ok(StateMachineMember::DefState(self.def_state()?)),
            Keyword(Signal) => Ok(StateMachineMember::DefSignal(self.def_signal()?)),
            Keyword(Action) => Ok(StateMachineMember::DefAction(self.def_action()?)),
            Keyword(Guard) => Ok(StateMachineMember::DefGuard(self.def_guard()?)),
            Keyword(Choice) => Ok(StateMachineMember::DefChoice(self.def_choice()?)),
            _ => Err(self.cursor.err_expected_one_of(
                "state machine member expected",
                vec![
                    Keyword(Initial),
                    Keyword(State),
                    Keyword(Signal),
                    Keyword(Action),
                    Keyword(Guard),
                    Keyword(Choice),
                ],
            )),
        }
    }

    fn state_member(&mut self) -> ParseResult<StateMember> {
        match self.peek(0) {
            Keyword(Choice) => Ok(StateMember::DefChoice(self.def_choice()?)),
            Keyword(State) => Ok(StateMember::DefState(self.def_state()?)),
            Keyword(Initial) => Ok(StateMember::SpecInitialTransition(
                self.spec_initial_transition()?,
            )),
            Keyword(Entry) => Ok(StateMember::SpecStateEntry(self.spec_state_entry()?)),
            Keyword(Exit) => Ok(StateMember::SpecStateExit(self.spec_state_exit()?)),
            Keyword(On) => Ok(StateMember::SpecStateTransition(
                self.spec_state_transition()?,
            )),
            _ => Err(self.cursor.err_expected_one_of(
                "state member expected",
                vec![
                    Keyword(Choice),
                    Keyword(State),
                    Keyword(Initial),
                    Keyword(Entry),
                    Keyword(Exit),
                    Keyword(On),
                ],
            )),
        }
    }

    fn spec_initial_transition(&mut self) -> ParseResult<AstNode<SpecInitialTransition>> {
        let first = self.consume_keyword(Initial)?;
        let transition = self.transition_expr()?;

        Ok(self.node(SpecInitialTransition { transition }, first.span()))
    }

    fn spec_state_entry(&mut self) -> ParseResult<AstNode<SpecStateEntry>> {
        let first = self.consume_keyword(Entry)?;
        let actions = self.do_expr()?;

        Ok(self.node(SpecStateEntry { actions }, first.span()))
    }

    fn do_expr(&mut self) -> ParseResult<DoExpr> {
        self.consume_keyword(Do)?;
        self.consume(LeftCurly)?;
        let elts = self.element_sequence(&Parser::ident, Comma, RightCurly)?;
        self.consume(RightCurly)?;

        Ok(DoExpr(elts))
    }

    fn spec_state_exit(&mut self) -> ParseResult<AstNode<SpecStateExit>> {
        let first = self.consume_keyword(Exit)?;
        let actions = self.do_expr()?;
        Ok(self.node(SpecStateExit { actions }, first.span()))
    }

    fn spec_state_transition(&mut self) -> ParseResult<AstNode<SpecStateTransition>> {
        let first = self.consume_keyword(On)?;
        let signal = self.ident()?;
        let guard = match self.peek(0) {
            Keyword(If) => {
                self.next();
                Some(self.ident()?)
            }
            _ => None,
        };

        let transition_or_do = self.transition_or_do()?;

        Ok(self.node(
            SpecStateTransition {
                signal,
                guard,
                transition_or_do,
            },
            first.span(),
        ))
    }

    fn transition_or_do(&mut self) -> ParseResult<TransitionOrDo> {
        let first_span = match self.peek_span(0) {
            Some(span) => span,
            _ => return Err(self.cursor.err_unexpected_eof()),
        };

        let do_expr = match self.peek(0) {
            Keyword(Do) => Some(self.do_expr()?),
            _ => None,
        };

        match self.peek(0) {
            Keyword(Enter) => {
                self.next();
                let target = self.qual_ident()?;
                Ok(TransitionOrDo::Transition(self.node(
                    TransitionExpr {
                        actions: do_expr.unwrap_or(DoExpr(vec![])),
                        target,
                    },
                    first_span,
                )))
            }
            _ => match do_expr {
                Some(de) => Ok(TransitionOrDo::Do(de)),
                None => Err(self.cursor.err_expected_one_of(
                    "expected transition or do",
                    vec![Keyword(Do), Keyword(Enter)],
                )),
            },
        }
    }

    fn def_guard(&mut self) -> ParseResult<AstNode<DefGuard>> {
        let first = self.consume_keyword(Guard)?;
        let name = self.ident()?;
        let type_name = match self.peek(0) {
            Colon => {
                self.next();
                Some(self.type_name()?)
            }
            _ => None,
        };

        Ok(self.node(DefGuard { name, type_name }, first.span()))
    }

    fn def_struct(&mut self) -> ParseResult<AstNode<DefStruct>> {
        let first = self.consume_keyword(Struct)?;
        let name = self.ident()?;

        self.consume(LeftCurly)?;
        let members =
            self.annotated_element_sequence(&Parser::struct_type_member, Comma, RightCurly)?;
        self.consume(RightCurly)?;

        let default = match self.peek(0) {
            Keyword(Default) => {
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

        self.consume(Colon)?;
        let size = match self.peek(0) {
            LeftSquare => Some(self.index()?),
            _ => None,
        };

        let type_name = self.type_name()?;
        let format = match self.peek(0) {
            Keyword(Format) => {
                self.next();
                Some(self.lit_string()?)
            }
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

    fn def_state(&mut self) -> ParseResult<AstNode<DefState>> {
        let first = self.consume_keyword(State)?;
        let name = self.ident()?;

        let members = match self.peek(0) {
            LeftCurly => {
                self.next();
                let members =
                    self.annotated_element_sequence(&Parser::state_member, Semi, RightCurly)?;
                self.consume(RightCurly)?;
                members
            }
            _ => vec![],
        };

        Ok(self.node(DefState { name, members }, first.span()))
    }

    fn def_signal(&mut self) -> ParseResult<AstNode<DefSignal>> {
        let first = self.consume_keyword(Signal)?;
        let name = self.ident()?;
        self.consume(Colon)?;
        let type_name = match self.peek(0) {
            Colon => {
                self.next();
                Some(self.type_name()?)
            }
            _ => None,
        };

        Ok(self.node(DefSignal { name, type_name }, first.span()))
    }

    fn spec_port_general(&mut self) -> ParseResult<AstNode<SpecPortInstance>> {
        let first = self.peek_span(0).unwrap();

        let kind = match self.peek(0) {
            Keyword(Async) => {
                self.next();
                self.consume_keyword(Input)?;
                Ok(GeneralPortInstanceKind::Input(InputPortKind::Async))
            }
            Keyword(Guarded) => {
                self.next();
                self.consume_keyword(Input)?;
                Ok(GeneralPortInstanceKind::Input(InputPortKind::Guarded))
            }
            Keyword(Sync) => {
                self.next();
                self.consume_keyword(Input)?;
                Ok(GeneralPortInstanceKind::Input(InputPortKind::Sync))
            }
            Keyword(Output) => {
                self.next();
                Ok(GeneralPortInstanceKind::Output)
            }
            _ => Err(self.cursor.err_expected_one_of(
                "expected general port instance",
                vec![
                    Keyword(Async),
                    Keyword(Guarded),
                    Keyword(Sync),
                    Keyword(Output),
                ],
            )),
        }?;

        self.consume_keyword(Port)?;
        let name = self.ident()?;

        self.consume(Colon)?;
        let size = match self.peek(0) {
            LeftSquare => Some(self.index()?),
            _ => None,
        };

        let port = match self.peek(0) {
            Keyword(Serial) => None,
            _ => Some(self.qual_ident()?),
        };

        let priority = match self.peek(0) {
            Keyword(Priority) => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        let queue_full = match self.peek(0) {
            Keyword(Assert) | Keyword(Block) | Keyword(Drop) | Keyword(Hook) => {
                Some(self.queue_full()?)
            }
            _ => None,
        };

        Ok(self.node(
            SpecPortInstance::General {
                kind,
                name,
                size,
                port,
                priority,
                queue_full,
            },
            first,
        ))
    }

    fn spec_port_special(&mut self) -> ParseResult<AstNode<SpecPortInstance>> {
        let first = match self.peek_span(0) {
            Some(span) => span,
            None => {
                return Err(self.cursor.err_expected_one_of(
                    "special port expected",
                    vec![
                        Keyword(Async),
                        Keyword(Sync),
                        Keyword(Guarded),
                        Keyword(Command),
                        Keyword(Event),
                        Keyword(Param),
                        Keyword(Product),
                        Keyword(Telemetry),
                        Keyword(Text),
                        Keyword(Time),
                    ],
                ));
            }
        };

        let input_kind = match self.peek(0) {
            Keyword(Async) => {
                self.next();
                Some(InputPortKind::Async)
            }
            Keyword(Sync) => {
                self.next();
                Some(InputPortKind::Sync)
            }
            Keyword(Guarded) => {
                self.next();
                Some(InputPortKind::Guarded)
            }
            _ => None,
        };

        let kind = match self.peek(0) {
            Keyword(Command) => match self.peek(1) {
                Keyword(Recv) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::CommandRecv)
                }
                Keyword(Reg) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::CommandReg)
                }
                Keyword(Resp) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::CommandResp)
                }
                _ => Err(self.cursor.err_expected_one_of(
                    "command special port expected",
                    vec![Keyword(Recv), Keyword(Reg), Keyword(Resp)],
                )),
            },
            Keyword(Event) => {
                self.next();
                Ok(SpecialPortInstanceKind::Event)
            }
            Keyword(Param) => match self.peek(1) {
                Keyword(Get) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::ParamGet)
                }
                Keyword(Set) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::ParamSet)
                }
                _ => Err(self.cursor.err_expected_one_of(
                    "param special port expected",
                    vec![Keyword(Get), Keyword(Set)],
                )),
            },
            Keyword(Product) => match self.peek(1) {
                Keyword(Get) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::ProductGet)
                }
                Keyword(Recv) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::ProductRecv)
                }
                Keyword(Request) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::ProductRequest)
                }
                Keyword(Send) => {
                    self.next();
                    self.next();
                    Ok(SpecialPortInstanceKind::ProductSend)
                }
                _ => Err(self.cursor.err_expected_one_of(
                    "product special port expected",
                    vec![Keyword(Get), Keyword(Recv), Keyword(Request), Keyword(Send)],
                )),
            },
            Keyword(Telemetry) => {
                self.next();
                Ok(SpecialPortInstanceKind::Telemetry)
            }
            Keyword(Text) => {
                self.next();
                self.consume_keyword(Event)?;
                Ok(SpecialPortInstanceKind::TextEvent)
            }
            Keyword(Time) => {
                self.next();
                self.consume_keyword(Get)?;
                Ok(SpecialPortInstanceKind::TimeGet)
            }
            _ => Err(self.cursor.err_expected_one_of(
                "special port expected",
                vec![
                    Keyword(Command),
                    Keyword(Event),
                    Keyword(Param),
                    Keyword(Product),
                    Keyword(Telemetry),
                    Keyword(Text),
                    Keyword(Time),
                ],
            )),
        }?;

        self.consume_keyword(Port)?;
        let name = self.ident()?;

        let priority = match self.peek(0) {
            Keyword(Priority) => {
                self.next();
                Some(self.expr()?)
            }
            _ => None,
        };

        let queue_full = match self.peek(0) {
            Keyword(Assert) | Keyword(Block) | Keyword(Drop) | Keyword(Hook) => {
                Some(self.queue_full()?)
            }
            _ => None,
        };

        Ok(self.node(
            SpecPortInstance::Special {
                input_kind,
                kind,
                name,
                priority,
                queue_full,
            },
            first,
        ))
    }

    fn spec_port_instance(&mut self) -> ParseResult<AstNode<SpecPortInstance>> {
        match self.peek(0) {
            Keyword(Async) | Keyword(Guarded) | Keyword(Sync) => match self.peek(1) {
                Keyword(Input) => self.spec_port_general(),
                _ => self.spec_port_special(),
            },
            Keyword(Output) => self.spec_port_general(),
            _ => self.spec_port_special(),
        }
    }

    fn queue_full(&mut self) -> ParseResult<QueueFull> {
        let out = match self.peek(0) {
            Keyword(Assert) => Ok(QueueFull::Assert),
            Keyword(Block) => Ok(QueueFull::Block),
            Keyword(Drop) => Ok(QueueFull::Drop),
            Keyword(Hook) => Ok(QueueFull::Hook),
            _ => Err(self.cursor.err_expected_one_of(
                "queue full expected",
                vec![
                    Keyword(Assert),
                    Keyword(Block),
                    Keyword(Drop),
                    Keyword(Hook),
                ],
            )),
        }?;

        self.next();
        Ok(out)
    }

    fn pre_annotation(&mut self) -> Vec<String> {
        let mut out: Vec<String> = vec![];

        while self.peek(0) == PreAnnotation {
            out.push(self.consume(PreAnnotation).unwrap().text().to_string())
        }

        out
    }

    fn post_annotation(&mut self) -> Vec<String> {
        let mut out: Vec<String> = vec![];

        while self.peek(0) == PostAnnotation {
            out.push(self.consume(PostAnnotation).unwrap().text().to_string())
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
        if punct_tok == *punct || punct_tok == Eol {
            self.next();
            let post_annotation = self.post_annotation();
            ElementParsingResult::Terminated(Annotated {
                pre_annotation,
                data,
                post_annotation,
            })
        } else if self.peek(0) == PostAnnotation {
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
        while self.peek(0) == Eol {
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
        if punct_tok == *punct || punct_tok == Eol {
            self.next();
            ElementParsingResult::Terminated(data)
        } else if self.peek(0) == PostAnnotation {
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
        while self.peek(0) == Eol {
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
        self.consume(LeftSquare)?;
        let out = self.expr()?;
        self.consume(RightSquare)?;
        Ok(out)
    }

    fn qual_ident(&mut self) -> ParseResult<AstNode<QualIdent>> {
        todo!()
    }

    fn type_name(&mut self) -> ParseResult<AstNode<TypeName>> {
        todo!()
    }

    fn expr(&mut self) -> ParseResult<AstNode<Expr>> {
        todo!()
    }

    fn lit_string(&mut self) -> ParseResult<AstNode<String>> {
        todo!()
    }

    fn transition_expr(&mut self) -> ParseResult<AstNode<TransitionExpr>> {
        todo!()
    }

    /// Convenience function to consume a keyword
    #[inline]
    pub fn consume_keyword(&mut self, kind: KeywordKind) -> ParseResult<Token> {
        self.consume(Keyword(kind))
    }

    /// Check to make sure a token at a certain position (number of tokens) away matches
    /// an expected token kind
    pub fn check(&mut self, n: usize, kind: TokenKind) -> ParseResult<()> {
        let p = self.peek(n);

        if kind == p {
            Ok(())
        } else {
            Err(self.cursor.err_expected_token("unexpected token", kind, p))
        }
    }
}
