pub mod component;
pub mod state_machine;
pub mod topology;
pub mod visit;

pub use component::*;
use fpp_core::Annotated;
pub use state_machine::*;
pub use topology::*;
pub use visit::*;

use fpp_macros::ast;
use fpp_macros::AstAnnotated;

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
#[derive(Debug)]
pub struct Ident {
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

#[derive(Debug)]
pub enum TypeNameKind {
    Floating(FloatType),
    Integer(IntegerType),
    QualIdent(QualIdent),
    Bool(),
    String(Option<Expr>),
}

#[ast]
#[derive(Debug)]
pub struct TypeName {
    pub kind: TypeNameKind,
}

#[ast]
#[derive(Debug)]
pub struct Qualified {
    pub qualifier: Box<QualIdent>,
    pub name: Ident,
}

#[ast]
pub enum QualIdent {
    Unqualified(Ident),
    Qualified(Qualified),
}

#[derive(Debug)]
pub struct StructMember {
    pub name: Ident,
    pub value: Expr,
}

#[derive(Debug)]
pub enum ExprKind {
    Array(Vec<Expr>),
    ArraySubscript {
        e1: Box<Expr>,
        e2: Box<Expr>,
    },
    Binop {
        left: Box<Expr>,
        op: Binop,
        right: Box<Expr>,
    },
    Dot {
        e: Box<Expr>,
        id: Ident,
    },
    Ident(String),
    LiteralBool(bool),
    LiteralInt(String),
    LiteralFloat(String),
    LiteralString(String),
    Paren(Box<Expr>),
    Struct(Vec<StructMember>),
    Unop {
        op: Unop,
        e: Box<Expr>,
    },
}

#[ast]
#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum FormalParamKind {
    Ref,
    Value,
}

#[ast]
#[derive(AstAnnotated)]
pub struct FormalParam {
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
#[derive(AstAnnotated)]
pub struct DefAbsType {
    pub name: Ident,
}

/** Aliased type definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefAliasType {
    pub name: Ident,
    pub type_name: TypeName,
}

/** Array definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefArray {
    pub name: Ident,
    pub size: Expr,
    pub elt_type: TypeName,
    pub default: Option<Expr>,
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
#[derive(AstAnnotated)]
pub struct DefComponent {
    pub kind: ComponentKind,
    pub name: Ident,
    pub members: Vec<ComponentMember>,
}

/** Component instance definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefComponentInstance {
    pub name: Ident,
    pub component: QualIdent,
    pub base_id: Expr,
    pub impl_type: Option<LitString>,
    pub file: Option<LitString>,
    pub queue_size: Option<Expr>,
    pub stack_size: Option<Expr>,
    pub priority: Option<Expr>,
    pub cpu: Option<Expr>,
    pub init_specs: Vec<SpecInit>,
}

/** Init specifier */
#[ast]
#[derive(AstAnnotated)]
pub struct SpecInit {
    pub phase: Expr,
    pub code: LitString,
}

/** Constant definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefConstant {
    pub name: Ident,
    pub value: Expr,
}

/** Enum definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefEnum {
    pub name: Ident,
    pub type_name: Option<TypeName>,
    pub constants: Vec<DefEnumConstant>,
    pub default: Option<Expr>,
}

/** Enum constant definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefEnumConstant {
    pub name: Ident,
    pub value: Option<Expr>,
}

/** Module definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefModule {
    pub name: Ident,
    pub members: Vec<ModuleMember>,
}

#[ast]
#[derive(AstAnnotated)]
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
#[derive(AstAnnotated)]
pub struct SpecLoc {
    pub kind: SpecLocKind,
    pub symbol: QualIdent,
    pub file: LitString,
}

#[derive(Debug)]
pub enum SpecPortInstanceKind {
    General {
        kind: GeneralPortInstanceKind,
        name: Ident,
        size: Option<Expr>,
        port: Option<QualIdent>,
        priority: Option<Expr>,
        queue_full: Option<QueueFull>,
    },
    Special {
        input_kind: Option<InputPortKind>,
        kind: SpecialPortInstanceKind,
        name: Ident,
        priority: Option<Expr>,
        queue_full: Option<QueueFull>,
    },
}

#[ast]
#[derive(AstAnnotated)]
pub struct SpecPortInstance {
    pub kind: SpecPortInstanceKind,
}

/** Interface member */
#[ast]
#[derive(AstAnnotated)]
pub enum InterfaceMember {
    SpecPortInstance(SpecPortInstance),
    SpecImport(SpecImport),
}

/** Interface definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefInterface {
    pub name: Ident,
    pub members: Vec<InterfaceMember>,
}

#[ast]
#[derive(AstAnnotated)]
pub struct StructTypeMember {
    pub name: Ident,
    pub size: Option<Expr>,
    pub type_name: TypeName,
    pub format: Option<LitString>,
}

/** Struct definition */
#[ast]
#[derive(AstAnnotated)]
pub struct DefStruct {
    pub name: Ident,
    pub members: Vec<StructTypeMember>,
    pub default: Option<Expr>,
}

#[ast]
#[derive(AstAnnotated)]
pub struct DefPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub return_type: Option<TypeName>,
}

/** Include specifier */
#[ast]
#[derive(AstAnnotated)]
pub struct SpecInclude {
    pub file: LitString,
}

/** Import specifier */
#[ast]
#[derive(AstAnnotated)]
pub struct SpecImport {
    pub sym: QualIdent,
}
