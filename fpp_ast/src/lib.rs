use crate::component::{ComponentMember, GeneralPortInstanceKind, InputPortKind, QueueFull, SpecialPortInstanceKind};
use crate::state_machine::DefStateMachine;
use crate::topology::DefTopology;

pub mod component;
pub mod state_machine;
pub mod topology;
pub mod visit;

pub struct AstNode<T> {
    pub id: i64,
    pub data: T,
}

pub struct Annotated<T> {
    pub pre_annotation: String,
    pub data: T,
    pub post_annotation: String,
}

pub struct AnnotatedNode<T>(Annotated<AstNode<T>>);

/** Identifier */
pub type Ident = String;

pub enum FloatType {
    F32,
    F64,
}

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

pub enum TypeName {
    Floating(FloatType),
    Integer(IntegerType),
    QualIdent(QualIdent),
    Bool(),
    String(Option<Expr>),
}

// #[derive(fpp_derive::Ast)]
pub enum QualIdent {
    Unqualified(Ident),
    Qualified {
        qualifier: Box<QualIdent>,
        name: Ident,
    },
}

pub enum Expr {
    Array(Vec<Expr>),
    Binop {
        left: Box<Expr>,
        op: Binop,
        right: Box<Expr>,
    },
    Dot {
        left: Box<Expr>,
        right: Ident,
    },
    Ident(Ident),
    LiteralBool(AstNode<bool>),
    LiteralInt(AstNode<String>),
    LiteralFloat(AstNode<String>),
    Paren(Box<Expr>),
    Unop {
        op: AstNode<Unop>,
        e: Box<Expr>,
    },
}

pub enum FormalParamKind {
    Ref,
    Value,
}

pub struct FormalParam {
    kind: FormalParamKind,
    name: Ident,
    type_name: AstNode<TypeName>,
}

pub type FormalParamList = Vec<Annotated<FormalParam>>;


/** Binary operation */
pub enum Binop {
    Add,
    Div,
    Mul,
    Sub,
}

pub enum Unop {
    Minus,
}

/** Abstract type definition */
pub struct DefAbsType {
    pub name: Ident,
}

/** Aliased type definition */
pub struct DefAliasType {
    pub name: Ident,
    pub type_name: AstNode<TypeName>,
}

/** Array definition */
pub struct DefArray {
    pub name: Ident,
    pub size: AstNode<Expr>,
    pub elt_type: AstNode<TypeName>,
    pub default: Option<AstNode<Expr>>,
    pub format: Option<AstNode<Expr>>,
}

pub enum ComponentKind {
    Active,
    Passive,
    Queued,
}

/** Component definition */
pub struct DefComponent {
    pub kind: ComponentKind,
    pub name: Ident,
    pub members: Vec<ComponentMember>,
}

/** Component instance definition */
pub struct DefComponentInstance {
    pub name: Ident,
    pub component: AstNode<QualIdent>,
    pub base_id: AstNode<Expr>,
    pub impl_type: Option<AstNode<String>>,
    pub file: Option<AstNode<String>>,
    pub queue_size: Option<Expr>,
    pub stack_size: Option<Expr>,
    pub priority: Option<Expr>,
    pub cpu: Option<Expr>,
    pub init_specs: Vec<Annotated<SpecInit>>,
}

/** Init specifier */
pub struct SpecInit {
    pub phase: AstNode<Expr>,
    pub code: AstNode<String>,
}

/** Constant definition */
pub struct DefConstant {
    pub name: Ident,
    pub value: AstNode<Expr>,
}

/** Enum definition */
pub struct DefEnum {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
    pub constants: Vec<Annotated<AstNode<DefEnumConstant>>>,
    pub default: Option<AstNode<Expr>>
}

/** Enum constant definition */
pub struct DefEnumConstant {
    pub name: Ident,
    pub value: Option<AstNode<Expr>>,
}

/** Module definition */
pub struct DefModule {
    pub name: Ident,
    pub members: Vec<ModuleMember>,
}

pub enum ModuleMember {
    DefAbsType(AnnotatedNode<DefAbsType>),
    DefAliasType(AnnotatedNode<DefAliasType>),
    DefArray(AnnotatedNode<DefArray>),
    DefComponent(AnnotatedNode<DefComponent>),
    DefComponentInstance(AnnotatedNode<DefComponentInstance>),
    DefConstant(AnnotatedNode<DefConstant>),
    DefEnum(AnnotatedNode<DefEnum>),
    DefInterface(AnnotatedNode<DefInterface>),
    DefModule(AnnotatedNode<DefModule>),
    DefPort(AnnotatedNode<DefPort>),
    DefStateMachine(AnnotatedNode<DefStateMachine>),
    DefStruct(AnnotatedNode<DefStruct>),
    DefTopology(AnnotatedNode<DefTopology>),
    SpecInclude(AnnotatedNode<SpecInclude>),
    SpecLoc(AnnotatedNode<SpecLoc>),
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
    pub kind: SpecLocKind,
    pub symbol: QualIdent,
    pub file: AstNode<String>,
}

pub enum SpecPortInstance {
    General{
        kind: GeneralPortInstanceKind,
        name: Ident,
        size: Option<AstNode<Expr>>,
        port: Option<AstNode<QualIdent>>,
        priority: Option<AstNode<Expr>>,
        queue_full: Option<QueueFull>,
    },
    Special{
        input_kind: Option<InputPortKind>,
        kind: SpecialPortInstanceKind,
        name: Ident,
        priority: Option<AstNode<Expr>>,
        queue_full: Option<AstNode<QueueFull>>,
    }
}

/** Interface member */
pub enum InterfaceMember {
    SpecPortInstance(AnnotatedNode<SpecPortInstance>),
    SpecImport(AnnotatedNode<SpecImport>),
}

/** Interface definition */
pub struct DefInterface {
    pub name: Ident,
    pub members: Vec<InterfaceMember>,
}

pub struct StructTypeMember {
    pub name: Ident,
    pub size: Option<AstNode<Expr>>,
    pub type_name: AstNode<TypeName>,
    pub format: Option<AstNode<String>>,
}

/** Struct definition */
pub struct DefStruct {
    name: Ident,
    members: Vec<Annotated<StructTypeMember>>,
    default: Option<AstNode<Expr>>,
}

pub struct DefPort {
    name: Ident,
    params: FormalParamList,
}

/** Include specifier */
pub struct SpecInclude {
    file: AstNode<String>,
}

/** Import specifier */
pub struct SpecImport {
    sym: AstNode<QualIdent>,
}
