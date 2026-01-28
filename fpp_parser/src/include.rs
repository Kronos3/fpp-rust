use crate::error::{ParseError, ParseResult};
use crate::{Parser, parse};
use fpp_ast::*;
use fpp_core::{FileReader, Position, SourceFile, Span, Spanned};
use rustc_hash::FxHashMap;
use std::ops::ControlFlow;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum IncludeParentKind {
    Component,
    Module,
    TlmPacket,
    TlmPacketSet,
    Topology,
}

type ResolveIncludesState = FxHashMap<SourceFile, (SourceFile, IncludeParentKind)>;

pub struct ResolveIncludes<Reader: FileReader> {
    reader: Reader,
}

impl<Reader: FileReader> ResolveIncludes<Reader> {
    pub fn new(reader: Reader) -> ResolveIncludes<Reader> {
        ResolveIncludes { reader }
    }

    fn check_loc_for_cycle(
        including_span: Span,
        including_path: String,
        loc_opt: Option<Span>,
        mut visited_paths: Vec<Position>,
    ) -> ParseResult<()> {
        match loc_opt {
            None => Ok(()),
            Some(loc) => {
                let uri = loc.file().uri();
                visited_paths.push(loc.start());
                if uri == including_path {
                    Err(ParseError::IncludeCycle {
                        span: including_span,
                        include_cycle: visited_paths,
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

    fn check_for_cycle(including_span: Span, including_path: String) -> ParseResult<()> {
        Self::check_loc_for_cycle(
            including_span,
            including_path.clone(),
            Some(including_span),
            vec![],
        )
    }

    fn resolve_spec_include<T>(
        &self,
        a: &mut ResolveIncludesState,
        kind: IncludeParentKind,
        spec_include: &SpecInclude,
        parser: fn(&mut Parser) -> Vec<T>,
        transformer: fn(&ResolveIncludes<Reader>, &mut ResolveIncludesState, T, &mut Vec<T>),
        out: &mut Vec<T>,
    ) {
        let file = match self
            .reader
            .include(spec_include.span().file(), &spec_include.file.data)
        {
            Ok(file) => file,
            Err(err) => {
                fpp_core::Diagnostic::new(
                    spec_include.file.span(),
                    fpp_core::Level::Error,
                    err.to_string(),
                )
                .emit();
                return;
            }
        };

        match Self::check_for_cycle(spec_include.span(), file.uri()) {
            Ok(_) => {}
            Err(err) => {
                fpp_core::Diagnostic::from(err.into()).emit();
                return;
            }
        };

        a.insert(file, (spec_include.span().file(), kind));
        let members = parse(file, parser, Some(spec_include.span()));
        for member in members {
            transformer(self, a, member, out);
        }
    }

    fn component_member(
        &self,
        a: &mut ResolveIncludesState,
        mut member: ComponentMember,
        out: &mut Vec<ComponentMember>,
    ) {
        match &member {
            ComponentMember::SpecInclude(spec_include) => self.resolve_spec_include(
                a,
                IncludeParentKind::Component,
                spec_include,
                |p| p.component_members(),
                Self::component_member,
                out,
            ),
            _ => {
                let _ = member.visit_mut(a, self);
                out.push(member)
            }
        }
    }

    fn topology_member(
        &self,
        a: &mut ResolveIncludesState,
        mut member: TopologyMember,
        out: &mut Vec<TopologyMember>,
    ) {
        match &mut member {
            TopologyMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    IncludeParentKind::Topology,
                    spec_include,
                    |p| p.topology_members(),
                    Self::topology_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit_mut(a, self);
                out.push(member)
            }
        }
    }

    fn module_member(
        &self,
        a: &mut ResolveIncludesState,
        mut member: ModuleMember,
        out: &mut Vec<ModuleMember>,
    ) {
        match &mut member {
            ModuleMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    IncludeParentKind::Module,
                    spec_include,
                    |p| p.module_members(),
                    Self::module_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit_mut(a, self);
                out.push(member)
            }
        }
    }

    fn tlm_packet_member(
        &self,
        a: &mut ResolveIncludesState,
        mut member: TlmPacketMember,
        out: &mut Vec<TlmPacketMember>,
    ) {
        match &mut member {
            TlmPacketMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    IncludeParentKind::TlmPacket,
                    spec_include,
                    |p| p.tlm_packet_members(),
                    Self::tlm_packet_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit_mut(a, self);
                out.push(member)
            }
        }
    }

    fn tlm_packet_set_member(
        &self,
        a: &mut ResolveIncludesState,
        mut member: TlmPacketSetMember,
        out: &mut Vec<TlmPacketSetMember>,
    ) {
        match &mut member {
            TlmPacketSetMember::SpecInclude(spec_include) => {
                self.resolve_spec_include(
                    a,
                    IncludeParentKind::TlmPacketSet,
                    spec_include,
                    |p| p.tlm_packet_set_members(),
                    Self::tlm_packet_set_member,
                    out,
                );
            }
            _ => {
                let _ = member.visit_mut(a, self);
                out.push(member)
            }
        }
    }
}

impl<Reader: FileReader> MutVisitor for ResolveIncludes<Reader> {
    type Break = ();
    type State = ResolveIncludesState;

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
        let old_members = std::mem::replace(&mut node.0, vec![]);
        for member in old_members.into_iter() {
            self.module_member(a, member, &mut node.0)
        }

        ControlFlow::Continue(())
    }
}
