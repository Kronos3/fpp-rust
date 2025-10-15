use fpp_core::span::Span;
use crate::common::*;
use crate::component::{DefComponent, SpecPortInstance};
use crate::state_machine::DefStateMachine;
use crate::topology::DefTopology;

pub mod common;
pub mod component;
pub mod state_machine;
pub mod topology;
pub mod visit;


/** Abstract type definition */
#[derive(fpp_derive::Ast)]
pub struct DefAbsType {
    name: Ident,
}

/** Aliased type definition */
pub struct DefAliasType {
    name: Ident,
    type_name: TypeName,
}

/** Array definition */
pub struct DefArray {
    name: Ident,
    size: Expr,
    elt_type: TypeName,
    default: Option<Expr>,
    format: Option<Expr>,
}

/** Component instance definition */
pub struct DefComponentInstance {
    name: Ident,
    component: QualIdent,
    base_id: Expr,
    impl_type: Option<StringNode>,
    file: Option<StringNode>,
    queue_size: Option<Expr>,
    stack_size: Option<Expr>,
    priority: Option<Expr>,
    cpu: Option<Expr>,
    init_specs: Vec<Annotated<SpecInit>>,
}

/** Init specifier */
pub struct SpecInit {
    phase: Expr,
    code: StringNode,
}

impl Node for SpecInit {
    fn span(&self) -> Span {
        Span::merge(self.phase.span(), self.code.span())
    }
}

/** Constant definition */
pub struct DefConstant {
    name: Ident,
    value: Expr,
}

impl Node for DefConstant {
    fn span(&self) -> Span {
        self.name.span()
    }
}

/** Enum definition */
pub struct DefEnum {
    name: Ident,
    type_name: Option<TypeName>,
    constants: Vec<Annotated<DefEnumConstant>>,
}

impl Node for DefEnum {
    fn span(&self) -> Span {
        self.name.span()
    }
}

/** Enum constant definition */
pub struct DefEnumConstant {
    name: Ident,
    value: Option<Expr>,
}

impl Node for DefEnumConstant {
    fn span(&self) -> Span {
        self.name.span()
    }
}

/** Module definition */
#[derive(fpp_derive::Ast)]
pub struct DefModule {
    name: Ident,
    members: Vec<Annotated<ModuleMember>>,
}

#[derive(fpp_derive::Ast)]
pub enum ModuleMember {
    DefAbsType(DefAbsType),
    DefAliasType(DefAliasType),
    DefArray(DefArray),
    DefComponent(DefComponent),
    DefComponentInstance(DefComponentInstance),
    DefConstant(DefConstant),
    DefEnum(DefEnum),
    DefInterface(DefInterface),
    DefModule(DefModule),
    DefPort(DefPort),
    DefStateMachine(DefStateMachine),
    DefStruct(DefStruct),
    DefTopology(DefTopology),
    SpecInclude(SpecInclude),
    SpecLoc(SpecLoc),
}

pub enum SpecLocKind {
    Component,
    Instance,
    Constant,
    Port,
    StateMachine,
    Topology,
    Type,
    Interface,
}

/** Location specifier */
pub struct SpecLoc {
    kind: SpecLocKind,
    symbol: QualIdent,
    file: StringNode,
}

/** Interface member */
pub enum InterfaceMember {
    SpecPortInstance(SpecPortInstance),
    SpecImport(SpecImport),
}

/** Interface definition */
pub struct DefInterface {
    name: Ident,
    members: Vec<InterfaceMember>,
}

pub struct StructTypeMember {
    name: Ident,
    size: Option<Expr>,
    type_name: TypeName,
    format: Option<StringNode>,
}

impl Node for StructTypeMember {
    fn span(&self) -> Span { self.name.span() }
}

/** Struct definition */
pub struct DefStruct {
    name: Ident,
    members: NodeList<Annotated<StructTypeMember>>,
    default: Option<Expr>,
}

pub struct DefPort {
    name: Ident,
    params: FormalParamList,
}

/** Include specifier */
pub struct SpecInclude {
    file: StringNode,
}

impl Node for SpecInclude {
    fn span(&self) -> Span { self.file.span() }
}

/** Import specifier */
pub struct SpecImport {
    sym: QualIdent,
}

impl Node for SpecImport {
    fn span(&self) -> Span { self.sym.span() }
}
