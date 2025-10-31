pub mod component;
pub mod state_machine;
pub mod topology;
pub mod visit;

pub use component::{*};
use fpp_core::Positioned;
pub use state_machine::{*};
pub use topology::{*};
pub use visit::{*};

#[derive(Debug)]
pub struct AstNode<T> {
    pub id: fpp_core::NodeId,
    pub data: T,
}

impl<T> Positioned for AstNode<T> {
    fn span(&self) -> fpp_core::Span {
        fpp_core::Positioned::span(&self.id)
    }
}

#[derive(Debug)]
pub struct Annotated<T> {
    pub pre_annotation: Vec<String>,
    pub data: T,
    pub post_annotation: Vec<String>,
}

/** Identifier */
pub type Ident = AstNode<String>;

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
pub enum TypeName {
    Floating(FloatType),
    Integer(IntegerType),
    QualIdent(AstNode<QualIdent>),
    Bool(),
    String(Option<AstNode<Expr>>),
}

#[derive(Debug)]
pub enum QualIdent {
    Unqualified(Ident),
    Qualified {
        qualifier: Box<AstNode<QualIdent>>,
        name: Ident,
    },
}

#[derive(Debug)]
pub struct StructMember {
    pub name: Ident,
    pub value: AstNode<Expr>
}

#[derive(Debug)]
pub enum Expr {
    Array(Vec<AstNode<Expr>>),
    ArraySubscript {
        e1: Box<AstNode<Expr>>,
        e2: Box<AstNode<Expr>>
    },
    Binop {
        left: Box<AstNode<Expr>>,
        op: Binop,
        right: Box<AstNode<Expr>>,
    },
    Dot {
        e: Box<AstNode<Expr>>,
        id: AstNode<Ident>,
    },
    Ident(Ident),
    LiteralBool(bool),
    LiteralInt(String),
    LiteralFloat(String),
    LiteralString(String),
    Paren(Box<Expr>),
    Struct(Vec<StructMember>),
    Unop {
        op: AstNode<Unop>,
        e: Box<AstNode<Expr>>,
    },
}

#[derive(Debug)]
pub enum FormalParamKind {
    Ref,
    Value,
}

#[derive(Debug)]
pub struct FormalParam {
    pub kind: FormalParamKind,
    pub name: Ident,
    pub type_name: AstNode<TypeName>,
}

pub type FormalParamList = Vec<Annotated<AstNode<FormalParam>>>;


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
#[derive(Debug)]
pub struct DefAbsType {
    pub name: Ident,
}

/** Aliased type definition */
#[derive(Debug)]
pub struct DefAliasType {
    pub name: Ident,
    pub type_name: AstNode<TypeName>,
}

/** Array definition */
#[derive(Debug)]
pub struct DefArray {
    pub name: Ident,
    pub size: AstNode<Expr>,
    pub elt_type: AstNode<TypeName>,
    pub default: Option<AstNode<Expr>>,
    pub format: Option<AstNode<String>>,
}

#[derive(Debug)]
pub enum ComponentKind {
    Active,
    Passive,
    Queued,
}

/** Component definition */
#[derive(Debug)]
pub struct DefComponent {
    pub kind: ComponentKind,
    pub name: Ident,
    pub members: Vec<Annotated<ComponentMember>>,
}

/** Component instance definition */
#[derive(Debug)]
pub struct DefComponentInstance {
    pub name: Ident,
    pub component: AstNode<QualIdent>,
    pub base_id: AstNode<Expr>,
    pub impl_type: Option<AstNode<String>>,
    pub file: Option<AstNode<String>>,
    pub queue_size: Option<AstNode<Expr>>,
    pub stack_size: Option<AstNode<Expr>>,
    pub priority: Option<AstNode<Expr>>,
    pub cpu: Option<AstNode<Expr>>,
    pub init_specs: Vec<Annotated<AstNode<SpecInit>>>,
}

/** Init specifier */
#[derive(Debug)]
pub struct SpecInit {
    pub phase: AstNode<Expr>,
    pub code: AstNode<String>,
}

/** Constant definition */
#[derive(Debug)]
pub struct DefConstant {
    pub name: Ident,
    pub value: AstNode<Expr>,
}

/** Enum definition */
#[derive(Debug)]
pub struct DefEnum {
    pub name: Ident,
    pub type_name: Option<AstNode<TypeName>>,
    pub constants: Vec<Annotated<AstNode<DefEnumConstant>>>,
    pub default: Option<AstNode<Expr>>
}

/** Enum constant definition */
#[derive(Debug)]
pub struct DefEnumConstant {
    pub name: Ident,
    pub value: Option<AstNode<Expr>>,
}

/** Module definition */
#[derive(Debug)]
pub struct DefModule {
    pub name: Ident,
    pub members: Vec<Annotated<ModuleMember>>,
}

#[derive(Debug)]
pub enum ModuleMember {
    DefAbsType(AstNode<DefAbsType>),
    DefAliasType(AstNode<DefAliasType>),
    DefArray(AstNode<DefArray>),
    DefComponent(AstNode<DefComponent>),
    DefComponentInstance(AstNode<DefComponentInstance>),
    DefConstant(AstNode<DefConstant>),
    DefEnum(AstNode<DefEnum>),
    DefInterface(AstNode<DefInterface>),
    DefModule(AstNode<DefModule>),
    DefPort(AstNode<DefPort>),
    DefStateMachine(AstNode<DefStateMachine>),
    DefStruct(AstNode<DefStruct>),
    DefTopology(AstNode<DefTopology>),
    SpecInclude(AstNode<SpecInclude>),
    SpecLoc(AstNode<SpecLoc>),
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
#[derive(Debug)]
pub struct SpecLoc {
    pub kind: SpecLocKind,
    pub symbol: QualIdent,
    pub file: AstNode<String>,
}

#[derive(Debug)]
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
        queue_full: Option<QueueFull>,
    }
}

/** Interface member */
#[derive(Debug)]
pub enum InterfaceMember {
    SpecPortInstance(AstNode<SpecPortInstance>),
    SpecImport(AstNode<SpecImport>),
}

/** Interface definition */
#[derive(Debug)]
pub struct DefInterface {
    pub name: Ident,
    pub members: Vec<Annotated<InterfaceMember>>,
}

#[derive(Debug)]
pub struct StructTypeMember {
    pub name: Ident,
    pub size: Option<AstNode<Expr>>,
    pub type_name: AstNode<TypeName>,
    pub format: Option<AstNode<String>>,
}

/** Struct definition */
#[derive(Debug)]
pub struct DefStruct {
    pub name: Ident,
    pub members: Vec<Annotated<AstNode<StructTypeMember>>>,
    pub default: Option<AstNode<Expr>>,
}

#[derive(Debug)]
pub struct DefPort {
    pub name: Ident,
    pub params: FormalParamList,
}

/** Include specifier */
#[derive(Debug)]
pub struct SpecInclude {
    pub file: AstNode<String>,
}

/** Import specifier */
#[derive(Debug)]
pub struct SpecImport {
    pub sym: AstNode<QualIdent>,
}
