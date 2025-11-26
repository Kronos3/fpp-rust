//! Abstract Syntax Tree, layered on top of untyped `SyntaxNode`s

use std::marker::PhantomData;

use crate::{
    syntax_node::{SyntaxNode, SyntaxNodeChildren, SyntaxToken},
    SyntaxKind,
};

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// The main trait to go from untyped `SyntaxNode`  to a typed ast. The
/// conversion itself has zero runtime cost: ast and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait AstNode {
    /// This panics if the `SyntaxKind` is not statically known.
    fn kind() -> SyntaxKind
    where
        Self: Sized,
    {
        panic!("dynamic `SyntaxKind` for `AstNode::kind()`")
    }

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }
    fn clone_subtree(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_subtree()).unwrap()
    }
}

/// Like `AstNode`, but wraps tokens rather than interior nodes.
pub trait AstToken {
    fn can_cast(token: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

impl<L, R> AstNode for Either<L, R>
where
    L: AstNode,
    R: AstNode,
{
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        L::can_cast(kind) || R::can_cast(kind)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if L::can_cast(syntax.kind()) {
            L::cast(syntax).map(Either::Left)
        } else {
            R::cast(syntax).map(Either::Right)
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        self.as_ref().either(L::syntax, R::syntax)
    }
}

impl<L, R> Either<L, R> {
    /// Apply one of two functions depending on contents, unifying their result. If the value is
    /// `Left(L)` then the first function `f` is applied; if it is `Right(R)` then the second
    /// function `g` is applied.
    ///
    /// ```
    /// use either::*;
    ///
    /// fn square(n: u32) -> i32 { (n * n) as i32 }
    /// fn negate(n: i32) -> i32 { -n }
    ///
    /// let left: Either<u32, i32> = Left(4);
    /// assert_eq!(left.either(square, negate), 16);
    ///
    /// let right: Either<u32, i32> = Right(-4);
    /// assert_eq!(right.either(square, negate), 4);
    /// ```
    pub fn either<F, G, T>(self, f: F, g: G) -> T
    where
        F: FnOnce(L) -> T,
        G: FnOnce(R) -> T,
    {
        match self {
            Either::Left(l) => f(l),
            Either::Right(r) => g(r),
        }
    }

    /// Convert `&Either<L, R>` to `Either<&L, &R>`.
    ///
    /// ```
    /// use either::*;
    ///
    /// let left: Either<_, ()> = Left("some value");
    /// assert_eq!(left.as_ref(), Left(&"some value"));
    ///
    /// let right: Either<(), _> = Right("some value");
    /// assert_eq!(right.as_ref(), Right(&"some value"));
    /// ```
    pub fn as_ref(&self) -> Either<&L, &R> {
        match self {
            Either::Left(left) => Either::Left(left),
            Either::Right(right) => Either::Right(right),
        }
    }
}

/// Trait to describe operations common to both `RangeExpr` and `RangePat`.
pub trait RangeItem {
    type Bound;

    fn start(&self) -> Option<Self::Bound>;
    fn end(&self) -> Option<Self::Bound>;
}

mod support {
    use super::{AstChildren, AstNode, SyntaxKind, SyntaxNode, SyntaxToken};

    #[inline]
    pub(super) fn child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
        parent.children().find_map(N::cast)
    }

    #[inline]
    pub(super) fn children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
        AstChildren::new(parent)
    }

    #[inline]
    pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
        parent
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find(|it| it.kind() == kind)
    }
}
