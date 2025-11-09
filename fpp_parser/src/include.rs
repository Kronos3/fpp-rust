use crate::error::{ParseError, ParseResult};
use crate::{parse, Parser};
use fpp_ast::{ComponentMember, DefComponent, DefModule, DefTopology, ModuleMember, MutVisitable, MutVisitor, SpecInclude, SpecTlmPacket, SpecTlmPacketSet, TlmPacketMember, TlmPacketSetMember, TopologyMember, TransUnit};
use fpp_core::{SourceFile, Span, Spanned};
use std::collections::HashSet;
use std::ops::ControlFlow;

pub struct ResolveSpecInclude {}

impl ResolveSpecInclude {
    pub fn new() -> ResolveSpecInclude {
        ResolveSpecInclude {}
    }

    fn check_loc_for_cycle(
        including_span: Span,
        including_path: String,
        loc_opt: Option<Span>,
        mut visited_paths: Vec<String>,
    ) -> ParseResult<()> {
        match loc_opt {
            None => Ok(()),
            Some(loc) => {
                let path = loc.file().path();
                match path {
                    None => Ok(()),
                    Some(path) => {
                        visited_paths.push(path.to_string());
                        if path == including_path {
                            Err(ParseError::IncludeCycle {
                                span: including_span,
                                include_path: visited_paths.iter().map(|p| p.to_string()).collect(),
                            })
                        } else {
                            Self::check_loc_for_cycle(
                                including_span,
                                including_path,
                                loc.including_span(),
                                visited_paths,
                            )
                        }
                    }
                }
            }
        }
    }

    fn check_for_cycle(including_span: Span, including_path: String) -> ParseResult<()> {
        match including_span.file().path() {
            None => Ok(()), // Stdin
            Some(_) => Self::check_loc_for_cycle(
                including_span,
                including_path.clone(),
                Some(including_span),
                vec![including_path],
            ),
        }
    }

    fn resolve_spec_include<T>(
        &self,
        a: &mut HashSet<SourceFile>,
        spec_include: &SpecInclude,
        parser: fn(&mut Parser) -> Vec<T>,
        transformer: fn(&ResolveSpecInclude, &mut HashSet<SourceFile>, T, &mut Vec<T>),
        out: &mut Vec<T>,
    ) {
        let file = match spec_include
            .span()
            .file()
            .open_relative_path(&spec_include.file.data)
        {
            Ok(file) => file,
            Err(err) => {
                fpp_core::Diagnostic::spanned(
                    spec_include.file.span(),
                    fpp_core::Level::Error,
                    err.to_string(),
                )
                .emit();
                return;
            }
        };

        match Self::check_for_cycle(spec_include.span(), file.path().unwrap()) {
            Ok(_) => {}
            Err(err) => {
                fpp_core::Diagnostic::from(err.into()).emit();
                return;
            }
        };

        a.insert(file);
        let members = parse(file, parser, Some(spec_include.span()));
        for member in members {
            transformer(self, a, member, out);
        }
    }

    fn component_member(
        &self,
        a: &mut HashSet<SourceFile>,
        mut member: ComponentMember,
        out: &mut Vec<ComponentMember>,
    ) {
        match &member {
            ComponentMember::SpecInclude(spec_include) => self.resolve_spec_include(
                a,
                spec_include,
                |p| p.component_members(),
                Self::component_member,
                out,
            ),
            _ => {
                let _ = member.visit(a, self);
                out.push(member)
            }
        }
    }

    fn topology_member(
        &self,
        a: &mut HashSet<SourceFile>,
        mut member: TopologyMember,
        out: &mut Vec<TopologyMember>,
    ) {
        match &mut member {
            TopologyMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    spec_include,
                    |p| p.topology_members(),
                    Self::topology_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit(a, self);
                out.push(member)
            }
        }
    }

    fn module_member(
        &self,
        a: &mut HashSet<SourceFile>,
        mut member: ModuleMember,
        out: &mut Vec<ModuleMember>,
    ) {
        match &mut member {
            ModuleMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    spec_include,
                    |p| p.module_members(),
                    Self::module_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit(a, self);
                out.push(member)
            }
        }
    }

    fn tlm_packet_member(
        &self,
        a: &mut HashSet<SourceFile>,
        mut member: TlmPacketMember,
        out: &mut Vec<TlmPacketMember>,
    ) {
        match &mut member {
            TlmPacketMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    spec_include,
                    |p| p.tlm_packet_members(),
                    Self::tlm_packet_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit(a, self);
                out.push(member)
            }
        }
    }

    fn tlm_packet_set_member(
        &self,
        a: &mut HashSet<SourceFile>,
        mut member: TlmPacketSetMember,
        out: &mut Vec<TlmPacketSetMember>,
    ) {
        match &mut member {
            TlmPacketSetMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    spec_include,
                    |p| p.tlm_packet_set_members(),
                    Self::tlm_packet_set_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit(a, self);
                out.push(member)
            }
        }
    }
}

impl MutVisitor for ResolveSpecInclude {
    type Break = ();
    type State = HashSet<SourceFile>;

    fn visit_def_component(
        &self,
        a: &mut Self::State,
        node: &mut DefComponent,
    ) -> ControlFlow<Self::Break> {
        let old_members = std::mem::replace(&mut node.members, vec![]);
        for member in old_members.into_iter() {
            self.component_member(a, member, &mut node.members)
        }

        ControlFlow::Continue(())
    }

    fn visit_def_module(
        &self,
        a: &mut Self::State,
        node: &mut DefModule,
    ) -> ControlFlow<Self::Break> {
        let old_members = std::mem::replace(&mut node.members, vec![]);
        for member in old_members.into_iter() {
            self.module_member(a, member, &mut node.members)
        }

        ControlFlow::Continue(())
    }

    fn visit_def_topology(
        &self,
        a: &mut Self::State,
        node: &mut DefTopology,
    ) -> ControlFlow<Self::Break> {
        let old_members = std::mem::replace(&mut node.members, vec![]);
        for member in old_members.into_iter() {
            self.topology_member(a, member, &mut node.members)
        }

        ControlFlow::Continue(())
    }

    fn visit_spec_tlm_packet(
        &self,
        a: &mut Self::State,
        node: &mut SpecTlmPacket,
    ) -> ControlFlow<Self::Break> {
        let old_members = std::mem::replace(&mut node.members, vec![]);
        for member in old_members.into_iter() {
            self.tlm_packet_member(a, member, &mut node.members)
        }

        ControlFlow::Continue(())
    }

    fn visit_spec_tlm_packet_set(
        &self,
        a: &mut Self::State,
        node: &mut SpecTlmPacketSet,
    ) -> ControlFlow<Self::Break> {
        let old_members = std::mem::replace(&mut node.members, vec![]);
        for member in old_members.into_iter() {
            self.tlm_packet_set_member(a, member, &mut node.members)
        }

        ControlFlow::Continue(())
    }

    fn visit_trans_unit(
        &self,
        a: &mut Self::State,
        node: &mut TransUnit,
    ) -> ControlFlow<Self::Break> {
        let old_members = std::mem::replace(&mut node.members, vec![]);
        for member in old_members.into_iter() {
            self.module_member(a, member, &mut node.members)
        }

        ControlFlow::Continue(())
    }
}
