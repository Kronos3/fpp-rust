pub mod component;
pub mod state_machine;
pub mod topology;
pub mod visit;

pub use component::*;
use fpp_core::Annotated;
pub use state_machine::*;
pub use topology::*;
pub use visit::*;

use fpp_macros::{ast, AstAnnotated, DirectWalkable, VisitorWalkable};

pub trait AstNode: fpp_core::Spanned + Sized {
    fn id(&self) -> fpp_core::Node;
}

pub type TranslationUnit = Vec<ModuleMember>;

#[ast]
#[derive(Debug, VisitorWalkable)]
pub struct LitString {
    #[visitable(ignore)]
    pub data: String,
}

/** Identifier */
#[ast]
#[derive(Debug, VisitorWalkable, Clone)]
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

#[derive(Debug, VisitorWalkable)]
pub enum TypeNameKind {
    #[visitable(ignore)]
    Bool(),
    #[visitable(ignore)]
    Floating(FloatType),
    #[visitable(ignore)]
    Integer(IntegerType),
    QualIdent(QualIdent),
    String(Option<Expr>),
}

#[ast]
#[derive(Debug, VisitorWalkable)]
pub struct TypeName {
    pub kind: TypeNameKind,
}

#[ast]
#[derive(Debug, VisitorWalkable)]
pub struct Qualified {
    pub qualifier: Box<QualIdent>,
    pub name: Ident,
}

#[ast]
#[derive(VisitorWalkable)]
pub enum QualIdent {
    Unqualified(Ident),
    Qualified(Qualified),
}

#[ast]
#[derive(Debug, VisitorWalkable)]
pub struct StructMember {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug, DirectWalkable)]
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
#[derive(Debug, VisitorWalkable)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum FormalParamKind {
    Ref,
    Value,
}

#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
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
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefAbsType {
    pub name: Ident,
}

/** Aliased type definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefAliasType {
    pub name: Ident,
    pub type_name: TypeName,
}

/** Array definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
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
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefComponent {
    #[visitable(ignore)]
    pub kind: ComponentKind,
    pub name: Ident,
    pub members: Vec<ComponentMember>,
}

/** Component instance definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
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
#[derive(AstAnnotated, VisitorWalkable)]
pub struct SpecInit {
    pub phase: Expr,
    #[visitable(ignore)]
    pub code: LitString,
}

/** Constant definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefConstant {
    pub name: Ident,
    pub value: Expr,
}

/** Enum definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefEnum {
    pub name: Ident,
    pub type_name: Option<TypeName>,
    pub constants: Vec<DefEnumConstant>,
    pub default: Option<Expr>,
}

/** Enum constant definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefEnumConstant {
    pub name: Ident,
    pub value: Option<Expr>,
}

/** Module definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefModule {
    pub name: Ident,
    pub members: Vec<ModuleMember>,
}

#[ast]
#[derive(AstAnnotated, DirectWalkable)]
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
#[derive(AstAnnotated, VisitorWalkable)]
pub struct SpecLoc {
    #[visitable(ignore)]
    pub kind: SpecLocKind,
    pub symbol: QualIdent,
    #[visitable(ignore)]
    pub file: LitString,
}

#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
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
#[derive(AstAnnotated, VisitorWalkable)]
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
#[derive(AstAnnotated, DirectWalkable)]
pub enum SpecPortInstance {
    General(SpecGeneralPortInstance),
    Special(SpecSpecialPortInstance),
}

/** Interface member */
#[ast]
#[derive(AstAnnotated, DirectWalkable)]
pub enum InterfaceMember {
    SpecPortInstance(SpecPortInstance),
    SpecImport(SpecImport),
}

/** Interface definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefInterface {
    pub name: Ident,
    pub members: Vec<InterfaceMember>,
}

#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct StructTypeMember {
    pub name: Ident,
    pub size: Option<Expr>,
    pub type_name: TypeName,
    #[visitable(ignore)]
    pub format: Option<LitString>,
}

/** Struct definition */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefStruct {
    pub name: Ident,
    pub members: Vec<StructTypeMember>,
    pub default: Option<Expr>,
}

#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct DefPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub return_type: Option<TypeName>,
}

/** Include specifier */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct SpecInclude {
    #[visitable(ignore)]
    pub file: LitString,
}

/** Import specifier */
#[ast]
#[derive(AstAnnotated, VisitorWalkable)]
pub struct SpecImport {
    #[visitable(ignore)]
    pub sym: QualIdent,
}
