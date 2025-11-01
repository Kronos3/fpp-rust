pub mod component;
pub mod state_machine;
pub mod topology;
pub mod visit;

pub use component::*;
pub use state_machine::*;
pub use topology::*;
pub use visit::*;

use fpp_macros::ast_annotated;
use fpp_macros::ast_node;

pub trait AstNode: fpp_core::Spanned {
    fn id(&self) -> fpp_core::Node;
}

#[ast_node]
#[derive(Debug)]
pub struct LitString {
    pub data: String,
}

/** Identifier */
#[ast_node]
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

#[ast_node]
#[derive(Debug)]
pub struct TypeName {
    pub kind: TypeNameKind,
}

#[ast_node]
#[derive(Debug)]
pub struct Qualified {
    pub qualifier: Box<QualIdent>,
    pub name: Ident,
}

#[ast_node]
#[derive(Debug)]
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

#[ast_node]
#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
}

#[derive(Debug)]
pub enum FormalParamKind {
    Ref,
    Value,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
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
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefAbsType {
    pub name: Ident,
}

/** Aliased type definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefAliasType {
    pub name: Ident,
    pub type_name: TypeName,
}

/** Array definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
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
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefComponent {
    pub kind: ComponentKind,
    pub name: Ident,
    pub members: Vec<ComponentMember>,
}

/** Component instance definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
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
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecInit {
    pub phase: Expr,
    pub code: LitString,
}

/** Constant definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefConstant {
    pub name: Ident,
    pub value: Expr,
}

/** Enum definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefEnum {
    pub name: Ident,
    pub type_name: Option<TypeName>,
    pub constants: Vec<DefEnumConstant>,
    pub default: Option<Expr>,
}

/** Enum constant definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefEnumConstant {
    pub name: Ident,
    pub value: Option<Expr>,
}

/** Module definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefModule {
    pub name: Ident,
    pub members: Vec<ModuleMember>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
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
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
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

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecPortInstance {
    pub kind: SpecPortInstanceKind,
}

/** Interface member */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub enum InterfaceMember {
    SpecPortInstance(SpecPortInstance),
    SpecImport(SpecImport),
}

/** Interface definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefInterface {
    pub name: Ident,
    pub members: Vec<InterfaceMember>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct StructTypeMember {
    pub name: Ident,
    pub size: Option<Expr>,
    pub type_name: TypeName,
    pub format: Option<LitString>,
}

/** Struct definition */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefStruct {
    pub name: Ident,
    pub members: Vec<StructTypeMember>,
    pub default: Option<Expr>,
}

#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct DefPort {
    pub name: Ident,
    pub params: FormalParamList,
    pub return_type: Option<TypeName>,
}

/** Include specifier */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecInclude {
    pub file: LitString,
}

/** Import specifier */
#[ast_node]
#[ast_annotated]
#[derive(Debug)]
pub struct SpecImport {
    pub sym: QualIdent,
}
