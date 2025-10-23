pub struct Annotated<T> {
    pre_annotation: String,
    data: T,
    post_annotation: String,
}

pub struct AstNode<T> {
    id: i64,
    data: T,
}

pub type StringNode = AstNode<String>;

pub type NodeList<T> = AstNode<Vec<T>>;

pub type Ident = AstNode<String>;

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
    Array(NodeList<Expr>),
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
