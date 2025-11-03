pub mod component;
pub mod state_machine;
pub mod topology;
pub mod visit;

pub use component::*;
use fpp_core::Annotated;
pub use state_machine::*;
pub use topology::*;
pub use visit::*;

use fpp_macros::AstAnnotated;
use fpp_macros::{ast, Walkable};

pub trait AstNode: fpp_core::Spanned {
    fn id(&self) -> fpp_core::Node;
}

#[ast]
#[derive(Debug)]
pub struct LitString {
    pub data: String,
}

/** Identifier */
#[ast]
#[derive(Debug, Walkable)]
pub struct Ident {
    #[visitable(ignore)]
    pub data: String,
}

#[derive(Debug)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Debug)]
pub enum IntegerType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
}

#[derive(Debug, Walkable)]
pub enum TypeNameKind {
    #[visitable(ignore)]
    Floating(FloatType),
    #[visitable(ignore)]
    Integer(IntegerType),
    QualIdent(QualIdent),
    #[visitable(ignore)]
    Bool(),
    String(Option<Expr>),
}

#[ast]
#[derive(Debug, Walkable)]
pub struct TypeName {
    pub kind: TypeNameKind,
}

#[ast]
#[derive(Debug, Walkable)]
pub struct Qualified {
    pub qualifier: Box<QualIdent>,
    pub name: Ident,
}

#[ast]
#[derive(Walkable)]
pub enum QualIdent {
    Unqualified(Ident),
    Qualified(Qualified),
}

#[derive(Debug, Walkable)]
pub struct StructMember {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, Walkable)]
#[visitable(no_self)]
pub enum ExprKind {
    Array(Vec<Expr>),
    ArraySubscript {
        e1: Box<Expr>,
        e2: Box<Expr>,
    },
    Binop {
        left: Box<Expr>,
        #[visitable(ignore)]
        op: Binop,
        right: Box<Expr>,
    },
    Dot {
        e: Box<Expr>,
        id: Ident,
    },
    #[visitable(ignore)]
    Ident(String),
    #[visitable(ignore)]
    LiteralBool(bool),
    #[visitable(ignore)]
    LiteralInt(String),
    #[visitable(ignore)]
    LiteralFloat(String),
    #[visitable(ignore)]
    LiteralString(String),
    Paren(Box<Expr>),
    Struct(Vec<StructMember>),
    Unop {
        #[visitable(ignore)]
        op: Unop,
        e: Box<Expr>,
    },
}

#[ast]
#[derive(Debug, Walkable)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum FormalParamKind {
    Ref,
    Value,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct FormalParam {
    #[visitable(ignore)]
    pub kind: FormalParamKind,
    pub name: Ident,
    pub type_name: TypeName,
}

pub type FormalParamList = Vec<FormalParam>;

/** Binary operation */
#[derive(Debug)]
pub enum Binop {
    Add,
    Div,
    Mul,
    Sub,
}

#[derive(Debug)]
pub enum Unop {
    Minus,
}

/** Abstract type definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefAbsType {
    pub name: Ident,
}

/** Aliased type definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefAliasType {
    pub name: Ident,
    pub type_name: TypeName,
}

/** Array definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefArray {
    pub name: Ident,
    pub size: Expr,
    pub elt_type: TypeName,
    pub default: Option<Expr>,
    #[visitable(ignore)]
    pub format: Option<LitString>,
}

#[derive(Debug)]
pub enum ComponentKind {
    Active,
    Passive,
    Queued,
}

/** Component definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefComponent {
    #[visitable(ignore)]
    pub kind: ComponentKind,
    pub name: Ident,
    pub members: Vec<ComponentMember>,
}

/** Component instance definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefComponentInstance {
    pub name: Ident,
    pub component: QualIdent,
    pub base_id: Expr,
    #[visitable(ignore)]
    pub impl_type: Option<LitString>,
    #[visitable(ignore)]
    pub file: Option<LitString>,
    pub queue_size: Option<Expr>,
    pub stack_size: Option<Expr>,
    pub priority: Option<Expr>,
    pub cpu: Option<Expr>,
    pub init_specs: Vec<SpecInit>,
}

/** Init specifier */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecInit {
    pub phase: Expr,
    #[visitable(ignore)]
    pub code: LitString,
}

/** Constant definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefConstant {
    pub name: Ident,
    pub value: Expr,
}

/** Enum definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefEnum {
    pub name: Ident,
    pub type_name: Option<TypeName>,
    pub constants: Vec<DefEnumConstant>,
    pub default: Option<Expr>,
}

/** Enum constant definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefEnumConstant {
    pub name: Ident,
    pub value: Option<Expr>,
}

/** Module definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefModule {
    pub name: Ident,
    pub members: Vec<ModuleMember>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
#[visitable(no_self)]
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

#[derive(Debug)]
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
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecLoc {
    #[visitable(ignore)]
    pub kind: SpecLocKind,
    pub symbol: QualIdent,
    #[visitable(ignore)]
    pub file: LitString,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecGeneralPortInstance {
    #[visitable(ignore)]
    pub kind: GeneralPortInstanceKind,
    pub name: Ident,
    pub size: Option<Expr>,
    pub port: Option<QualIdent>,
    pub priority: Option<Expr>,
    #[visitable(ignore)]
    pub queue_full: Option<QueueFull>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecSpecialPortInstance {
    #[visitable(ignore)]
    pub input_kind: Option<InputPortKind>,
    #[visitable(ignore)]
    pub kind: SpecialPortInstanceKind,
    pub name: Ident,
    pub priority: Option<Expr>,
    #[visitable(ignore)]
    pub queue_full: Option<QueueFull>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
#[visitable(no_self)]
pub enum SpecPortInstance {
    General(SpecGeneralPortInstance),
    Special(SpecSpecialPortInstance),
}

/** Interface member */
#[ast]
#[derive(AstAnnotated, Walkable)]
#[visitable(no_self)]
pub enum InterfaceMember {
    SpecPortInstance(SpecPortInstance),
    SpecImport(SpecImport),
}

/** Interface definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefInterface {
    pub name: Ident,
    pub members: Vec<InterfaceMember>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct StructTypeMember {
    pub name: Ident,
    pub size: Option<Expr>,
    pub type_name: TypeName,
    #[visitable(ignore)]
    pub format: Option<LitString>,
}

/** Struct definition */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefStruct {
    pub name: Ident,
    pub members: Vec<StructTypeMember>,
    pub default: Option<Expr>,
}

#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct DefPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub return_type: Option<TypeName>,
}

/** Include specifier */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecInclude {
    #[visitable(ignore)]
    pub file: LitString,
}

/** Import specifier */
#[ast]
#[derive(AstAnnotated, Walkable)]
pub struct SpecImport {
    #[visitable(ignore)]
    pub sym: QualIdent,
}
