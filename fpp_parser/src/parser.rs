use crate::cursor::Cursor;
use crate::error::{ParseError, ParseResult};
use crate::token::KeywordKind::*;
use crate::token::TokenKind::*;
use crate::token::{KeywordKind, Token, TokenKind};
use fpp_ast::*;
use fpp_core::{Positioned, SourceFile};

struct Parser<'a> {
    cursor: Cursor<'a>,
}

enum ElementParsingResult<T> {
    Terminated(T),
    Unterminated(T),
    Err(ParseError),
    None,
}

pub fn parse(source_file: SourceFile) -> ParseResult<AstNode<DefComponent>> {
    let content = source_file.read();
    let mut parser = Parser {
        cursor: Cursor::new(source_file, content.as_str()),
    };

    parser.component()
}

impl<'a> Parser<'a> {
    fn peek_span(&mut self, n: usize) -> Option<fpp_core::Span> {
        self.cursor.peek_span(n)
    }

    fn peek(&mut self, n: usize) -> TokenKind {
        self.cursor.peek(n)
    }

    fn next(&mut self) -> Option<Token> {
        self.cursor.next()
    }

    fn consume(&mut self, kind: TokenKind) -> ParseResult<Token> {
        self.cursor.consume(kind)
    }

    #[inline]
    fn node<T>(&self, data: T, first_token: fpp_core::Span) -> AstNode<T> {
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

    fn ident(&mut self) -> ParseResult<Ident> {
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
            _ => Err(self.cursor.err_expected_one_of(
                "component member expected",
                vec![
                    Keyword(Type),
                    Keyword(Array),
                    Keyword(Constant),
                    Keyword(Enum),
                    Keyword(State),
                    Keyword(Struct),
                    Keyword(Async),
                    Keyword(Sync),
                    Keyword(Guarded),
                    Keyword(Command),
                    Keyword(Text),
                    Keyword(Time),
                    Keyword(Product),
                    Keyword(Event),
                    Keyword(Include),
                    Keyword(Internal),
                    Keyword(Match),
                    Keyword(External),
                    Keyword(Param),
                    Keyword(Telemetry),
                    Keyword(Import),
                ],
            )),
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

        let priority = self.opt_expr(Priority)?;
        let cpu = self.opt_expr(Cpu)?;

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

        let priority = self.opt_expr(Priority)?;

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
        let elements = self.element_sequence(&Parser::ident, Comma, RightCurly)?;
        self.consume(RightCurly)?;

        Ok(DoExpr(elements))
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
        let first_span = self.current_span()?;

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

    fn transition_expr(&mut self) -> ParseResult<AstNode<TransitionExpr>> {
        let first_span = self.current_span()?;

        let do_expr = match self.peek(0) {
            Keyword(Do) => self.do_expr()?,
            _ => DoExpr(vec![]),
        };

        match self.peek(0) {
            Keyword(Enter) => {
                self.next();
                let target = self.qual_ident()?;
                Ok(self.node(
                    TransitionExpr {
                        actions: do_expr,
                        target,
                    },
                    first_span,
                ))
            }
            _ => Err(self.cursor.err_expected_one_of(
                "expected transition expression",
                vec![Keyword(Enter), Keyword(Do)],
            )),
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

    fn spec_container(&mut self) -> ParseResult<AstNode<SpecContainer>> {
        let first = self.consume_keyword(Product)?;
        self.consume_keyword(Container)?;
        let name = self.ident()?;
        let id = self.opt_expr(Id)?;
        let default_priority = match self.peek(0) {
            Keyword(Default) => {
                self.next();
                self.consume_keyword(Priority)?;
                Some(self.expr()?)
            }
            _ => None,
        };

        Ok(self.node(
            SpecContainer {
                name,
                id,
                default_priority,
            },
            first.span(),
        ))
    }

    fn spec_record(&mut self) -> ParseResult<AstNode<SpecRecord>> {
        let first = self.consume_keyword(Product)?;
        self.consume_keyword(Record)?;
        let name = self.ident()?;
        self.consume(Colon)?;
        let record_type = self.type_name()?;
        let is_array = match self.peek(0) {
            Keyword(Array) => {
                self.next();
                true
            }
            _ => false,
        };

        let id = self.opt_expr(Id)?;
        Ok(self.node(
            SpecRecord {
                name,
                record_type,
                is_array,
                id,
            },
            first.span(),
        ))
    }

    fn spec_event(&mut self) -> ParseResult<AstNode<SpecEvent>> {
        let first = self.consume_keyword(Event)?;
        let name = self.ident()?;
        let params = self.formal_param_list()?;
        self.consume_keyword(Severity)?;
        let severity = match self.peek(0) {
            Keyword(Activity) => {
                self.next();
                match self.peek(0) {
                    Keyword(High) => Ok(EventSeverity::ActivityHigh),
                    Keyword(Low) => Ok(EventSeverity::ActivityLow),
                    _ => Err(self.cursor.err_expected_one_of(
                        "severity level expected",
                        vec![Keyword(High), Keyword(Low)],
                    )),
                }
            }
            Keyword(Warning) => {
                self.next();
                match self.peek(0) {
                    Keyword(High) => Ok(EventSeverity::WarningHigh),
                    Keyword(Low) => Ok(EventSeverity::WarningLow),
                    _ => Err(self.cursor.err_expected_one_of(
                        "severity level expected",
                        vec![Keyword(High), Keyword(Low)],
                    )),
                }
            }
            Keyword(Command) => {
                self.next();
                Ok(EventSeverity::Command)
            }
            Keyword(Diagnostic) => {
                self.next();
                Ok(EventSeverity::Diagnostic)
            }
            Keyword(Fatal) => {
                self.next();
                Ok(EventSeverity::Fatal)
            }
            _ => Err(self.cursor.err_expected_one_of(
                "severity level expected",
                vec![
                    Keyword(Activity),
                    Keyword(Warning),
                    Keyword(Command),
                    Keyword(Diagnostic),
                    Keyword(Fatal),
                ],
            )),
        }?;

        let id = self.opt_expr(Id)?;
        let format = match self.peek(0) {
            Keyword(Format) => {
                self.next();
                Ok(self.lit_string()?)
            }
            _ => Err(self
                .cursor
                .err_expected_one_of("expected event format", vec![Keyword(Format)])),
        }?;

        let throttle = match self.peek(0) {
            Keyword(Throttle) => Some(self.event_throttle()?),
            _ => None,
        };

        Ok(self.node(
            SpecEvent {
                name,
                params,
                severity,
                id,
                format,
                throttle,
            },
            first.span(),
        ))
    }

    fn event_throttle(&mut self) -> ParseResult<AstNode<EventThrottle>> {
        let first = self.consume_keyword(Throttle)?;
        let count = self.expr()?;
        let every = self.opt_expr(Every)?;

        Ok(self.node(EventThrottle { count, every }, first.span()))
    }

    fn spec_include(&mut self) -> ParseResult<AstNode<SpecInclude>> {
        let first = self.consume_keyword(Include)?;
        let file = self.lit_string()?;
        Ok(self.node(SpecInclude { file }, first.span()))
    }

    fn spec_import_interface(&mut self) -> ParseResult<AstNode<SpecImport>> {
        let first = self.consume_keyword(Import)?;
        let sym = self.qual_ident()?;
        Ok(self.node(SpecImport { sym }, first.span()))
    }

    fn spec_internal_port(&mut self) -> ParseResult<AstNode<SpecInternalPort>> {
        let first = self.consume_keyword(Internal)?;
        self.consume_keyword(Port)?;
        let name = self.ident()?;
        let params = self.formal_param_list()?;
        let priority = self.opt_expr(Priority)?;
        let queue_full = self.opt_queue_full()?;
        Ok(self.node(
            SpecInternalPort {
                name,
                params,
                priority,
                queue_full,
            },
            first.span(),
        ))
    }

    fn spec_port_matching(&mut self) -> ParseResult<AstNode<SpecPortMatching>> {
        let first = self.consume_keyword(Match)?;
        let port1 = self.ident()?;
        self.consume_keyword(With)?;
        let port2 = self.ident()?;
        Ok(self.node(SpecPortMatching { port1, port2 }, first.span()))
    }

    fn spec_param(&mut self) -> ParseResult<AstNode<SpecParam>> {
        let first_span = self.current_span()?;
        let is_external = match self.peek(0) {
            Keyword(External) => {
                self.next();
                true
            }
            _ => false,
        };

        self.consume_keyword(Param)?;
        let name = self.ident()?;
        self.consume(Colon)?;
        let type_name = self.type_name()?;
        let default = self.opt_expr(Default)?;
        let id = self.opt_expr(Id)?;

        let set_opcode = match self.peek(0) {
            Keyword(Set) => {
                self.next();
                self.consume_keyword(Opcode)?;
                Some(self.expr()?)
            }
            _ => None,
        };

        let save_opcode = match self.peek(0) {
            Keyword(Save) => {
                self.next();
                self.consume_keyword(Opcode)?;
                Some(self.expr()?)
            }
            _ => None,
        };

        Ok(self.node(
            SpecParam {
                name,
                type_name,
                default,
                id,
                set_opcode,
                save_opcode,
                is_external,
            },
            first_span,
        ))
    }

    fn spec_tlm_channel(&mut self) -> ParseResult<AstNode<SpecTlmChannel>> {
        let first = self.consume_keyword(Telemetry)?;
        let name = self.ident()?;
        self.consume(Colon)?;
        let type_name = self.type_name()?;
        let id = self.opt_expr(Id)?;
        let update = match self.peek(0) {
            Keyword(Update) => {
                self.next();
                match self.peek(0) {
                    Keyword(Always) => {
                        self.next();
                        Ok(Some(TlmChannelUpdate::Always))
                    }
                    Keyword(On) => {
                        self.next();
                        self.consume_keyword(Change)?;
                        Ok(Some(TlmChannelUpdate::OnChange))
                    }
                    _ => Err(self.cursor.err_expected_one_of(
                        "update kind expected",
                        vec![Keyword(Always), Keyword(On)],
                    )),
                }
            }
            _ => Ok(None),
        }?;

        let format = match self.peek(0) {
            Keyword(Format) => {
                self.next();
                Some(self.lit_string()?)
            }
            _ => None,
        };

        let low = match self.peek(0) {
            Keyword(Low) => {
                self.next();
                self.limit_sequence()?
            }
            _ => vec![],
        };
        let high = match self.peek(0) {
            Keyword(High) => {
                self.next();
                self.limit_sequence()?
            }
            _ => vec![],
        };

        Ok(self.node(
            SpecTlmChannel {
                name,
                type_name,
                id,
                update,
                format,
                low,
                high,
            },
            first.span(),
        ))
    }

    fn limit_sequence(&mut self) -> ParseResult<Vec<TlmChannelLimit>> {
        self.consume(LeftCurly)?;
        let out = self.element_sequence(&Parser::limit, Comma, RightCurly)?;
        self.consume(RightCurly)?;
        Ok(out)
    }

    fn limit(&mut self) -> ParseResult<TlmChannelLimit> {
        let kind = self.limit_kind()?;
        let value = self.expr()?;
        Ok(TlmChannelLimit { kind, value })
    }

    fn limit_kind(&mut self) -> ParseResult<AstNode<TlmChannelLimitKind>> {
        let first_span = self.current_span()?;

        let kind = match self.peek(0) {
            Keyword(Orange) => Ok(TlmChannelLimitKind::Orange),
            Keyword(Red) => Ok(TlmChannelLimitKind::Red),
            Keyword(Yellow) => Ok(TlmChannelLimitKind::Yellow),
            _ => Err(self.cursor.err_expected_one_of(
                "telemetry channel limit kind expected",
                vec![Keyword(Orange), Keyword(Red), Keyword(Yellow)],
            )),
        }?;

        Ok(self.node(kind, first_span))
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

        let queue_full = self.opt_queue_full()?;

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

    fn spec_command(&mut self) -> ParseResult<AstNode<SpecCommand>> {
        let first_span = self.current_span()?;

        let kind = match self.peek(0) {
            Keyword(Async) => {
                self.next();
                self.consume_keyword(Input)?;
                Ok(InputPortKind::Async)
            }
            Keyword(Guarded) => {
                self.next();
                self.consume_keyword(Input)?;
                Ok(InputPortKind::Async)
            }
            Keyword(Sync) => {
                self.next();
                self.consume_keyword(Input)?;
                Ok(InputPortKind::Async)
            }
            _ => Err(self.cursor.err_expected_one_of(
                "command kind expected",
                vec![Keyword(Async), Keyword(Guarded), Keyword(Sync)],
            )),
        }?;

        let name = self.ident()?;
        let params = self.formal_param_list()?;
        let opcode = self.opt_expr(Opcode)?;
        let priority = self.opt_expr(Priority)?;
        let queue_full = self.opt_queue_full()?;

        Ok(self.node(
            SpecCommand {
                kind,
                name,
                params,
                opcode,
                priority,
                queue_full,
            },
            first_span,
        ))
    }

    fn opt_queue_full(&mut self) -> ParseResult<Option<QueueFull>> {
        match self.peek(0) {
            Keyword(Assert) | Keyword(Block) | Keyword(Drop) | Keyword(Hook) => {
                Ok(Some(self.queue_full()?))
            }
            _ => Ok(None),
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
        let first = self.ident()?;
        let first_span = first.span();
        let mut out = vec![];
        while self.peek(0) == Dot {
            self.next();
            out.push(self.ident()?)
        }

        Ok(out.into_iter().fold(
            self.node(QualIdent::Unqualified(first), first_span),
            |q, ident| {
                let q_span = q.span();
                self.node(
                    QualIdent::Qualified {
                        qualifier: Box::new(q),
                        name: ident,
                    },
                    q_span,
                )
            },
        ))
    }

    fn type_name(&mut self) -> ParseResult<AstNode<TypeName>> {
        let first_span = self.current_span()?;
        let tn = match self.peek(0) {
            Keyword(Bool) => {
                self.next();
                Ok(TypeName::Bool())
            }
            Keyword(I8) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::I8))
            }
            Keyword(U8) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::U8))
            }
            Keyword(I16) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::I16))
            }
            Keyword(U16) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::U16))
            }
            Keyword(I32) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::I32))
            }
            Keyword(U32) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::U32))
            }
            Keyword(I64) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::I64))
            }
            Keyword(U64) => {
                self.next();
                Ok(TypeName::Integer(IntegerType::U64))
            }
            Keyword(F32) => {
                self.next();
                Ok(TypeName::Floating(FloatType::F32))
            }
            Keyword(F64) => {
                self.next();
                Ok(TypeName::Floating(FloatType::F64))
            }
            Keyword(String_) => {
                self.next();
                let size = match self.peek(0) {
                    Keyword(Size) => {
                        self.next();
                        Some(self.expr()?)
                    }
                    _ => None,
                };
                Ok(TypeName::String(size))
            }
            Identifier => Ok(TypeName::QualIdent(self.qual_ident()?)),
            _ => Err(self.cursor.err_expected_one_of(
                "type name expected",
                vec![
                    Keyword(Bool),
                    Keyword(I8),
                    Keyword(U8),
                    Keyword(I16),
                    Keyword(U16),
                    Keyword(I32),
                    Keyword(U32),
                    Keyword(I64),
                    Keyword(U64),
                    Keyword(F32),
                    Keyword(F64),
                    Keyword(String_),
                    Identifier,
                ],
            )),
        }?;

        Ok(self.node(tn, first_span))
    }

    fn expr(&mut self) -> ParseResult<AstNode<Expr>> {
        todo!()
    }

    fn lit_string(&mut self) -> ParseResult<AstNode<String>> {
        let tok = self.consume(LiteralString)?;
        Ok(self.node(tok.text().to_string(), tok.span()))
    }

    fn formal_param_list(&mut self) -> ParseResult<FormalParamList> {
        match self.peek(0) {
            LeftParen => {
                self.next();
                let members =
                    self.annotated_element_sequence(&Parser::formal_param, Comma, RightParen)?;

                self.consume(RightParen)?;
                Ok(members)
            }
            _ => Ok(vec![]),
        }
    }

    fn formal_param(&mut self) -> ParseResult<AstNode<FormalParam>> {
        let first_span = self.current_span()?;

        let kind = match self.peek(0) {
            Keyword(Ref) => {
                self.next();
                FormalParamKind::Ref
            }
            _ => FormalParamKind::Value,
        };

        let name = self.ident()?;
        self.consume(Colon)?;

        let type_name = self.type_name()?;

        Ok(self.node(
            FormalParam {
                kind,
                name,
                type_name,
            },
            first_span,
        ))
    }

    fn opt_expr(&mut self, prefix_keyword: KeywordKind) -> ParseResult<Option<AstNode<Expr>>> {
        if self.peek(0) == Keyword(prefix_keyword) {
            self.next();
            Ok(Some(self.expr()?))
        } else {
            Ok(None)
        }
    }

    fn current_span(&mut self) -> ParseResult<fpp_core::Span> {
        match self.peek_span(0) {
            Some(span) => Ok(span),
            None => Err(self.cursor.err_unexpected_eof()),
        }
    }

    /// Convenience function to consume a keyword
    #[inline]
    fn consume_keyword(&mut self, kind: KeywordKind) -> ParseResult<Token> {
        self.consume(Keyword(kind))
    }
}
