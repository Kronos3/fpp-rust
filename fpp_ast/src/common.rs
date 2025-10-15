use fpp_core::span::Span;

/** An AST node is an object that represents some span inside the source code */
pub trait Node {
    /// Get the node range across the parent source file
    /// Used creating diagnostic purposes and interfacing with LSP
    fn span(&self) -> Span;
}

pub struct Annotated<T: Node> {
    pre_annotation: String,
    data: T,
    post_annotation: String,
}

impl<T: Node> Node for Annotated<T> {
    fn span(&self) -> Span {
        self.data.span()
    }
}

struct AstNode<T> {
    s: Span,
    data: T,
}

impl<T> Node for AstNode<T> {
    fn span(&self) -> Span {
        self.s.clone()
    }
}

pub type StringNode = AstNode<String>;

pub struct NodeList<T: Node> {
    items: Vec<T>,
    span: Span
}

impl<T: Node> Node for NodeList<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

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

#[derive(fpp_derive::Ast)]
pub enum TypeName {
    Floating(AstNode<FloatType>),
    Integer(AstNode<IntegerType>),
    QualIdent(AstNode<QualIdent>),
    Bool(AstNode<>),
    String(AstNode<Option<Expr>>),
}

impl Node for TypeName {
    fn span(&self) -> Span {
        match self {
            TypeName::Floating(f) => f.span(),
            TypeName::Integer(i) => i.span(),
            TypeName::QualIdent(qi) => qi.span(),
            TypeName::Bool(b) => b.clone(),
            TypeName::String(s) => s.span(),
        }
    }
}

// #[derive(fpp_derive::Ast)]
pub enum QualIdent {
    Unqualified(Ident),
    Qualified {
        qualifier: Box<QualIdent>,
        name: Ident,
    },
}

impl Node for QualIdent {
    fn span(&self) -> Span {
        match self {
            QualIdent::Unqualified(i) => i.span(),
            QualIdent::Qualified { qualifier, name } => Span::merge(qualifier.span(), name.span()),
        }
    }
}

pub type Ident = AstNode<String>;

pub enum Expr {
    Array(AstNode<Vec<Expr>>),
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

impl Node for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Array(a) => a.span(),
            Expr::Binop { left, op: _, right } => Span::merge(left.span(), right.span()),
            Expr::Dot { left, right } => Span::merge(left.span(), right.span()),
            Expr::Ident(id) => id.span(),
            Expr::LiteralBool(b) => b.span(),
            Expr::LiteralInt(i) => i.span(),
            Expr::LiteralFloat(f) => f.span(),
            Expr::Paren(e) => e.span(),
            Expr::Unop { op, e } => Span::merge(op.span(), e.span()),
        }
    }
}

pub enum FormalParamKind {
    Ref,
    Value,
}

pub struct FormalParam {
    kind: FormalParamKind,
    name: Ident,
    type_name: TypeName,
}

impl Node for FormalParam {
    fn span(&self) -> Span {
        Span::merge(self.kind.span(), self.type_name.span())
    }
}

pub type FormalParamList = Vec<Annotated<FormalParam>>;
