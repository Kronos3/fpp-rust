//! See [`Input`].

use crate::syntax::SyntaxKind;

#[allow(non_camel_case_types)]
type bits = u64;

/// Input for the parser -- a sequence of tokens.
///
/// As of now, parser doesn't have access to the *text* of the tokens, and makes
/// decisions based solely on their classification. Unlike `LexerToken`, the
/// `Tokens` doesn't include whitespace and comments. Main input to the parser.
///
/// Struct of arrays internally, but this shouldn't really matter.
pub struct Input {
    kind: Vec<SyntaxKind>,
    joint: Vec<bits>,
}

/// `pub` impl used by callers to create `Tokens`.
impl Input {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            kind: Vec::with_capacity(capacity),
            joint: Vec::with_capacity(capacity / size_of::<bits>()),
        }
    }
    #[inline]
    pub fn push(&mut self, kind: SyntaxKind) {
        self.push_impl(kind)
    }
    #[inline]
    pub fn push_ident(&mut self) {
        self.push_impl(SyntaxKind::IDENT)
    }
    #[inline]
    fn push_impl(&mut self, kind: SyntaxKind) {
        let idx = self.len();
        if idx.is_multiple_of(bits::BITS as usize) {
            self.joint.push(0);
        }
        self.kind.push(kind);
    }
}

/// pub(crate) impl used by the parser to consume `Tokens`.
impl Input {
    pub(crate) fn kind(&self, idx: usize) -> SyntaxKind {
        self.kind.get(idx).copied().unwrap_or(SyntaxKind::EOF)
    }
    pub(crate) fn is_joint(&self, n: usize) -> bool {
        let (idx, b_idx) = self.bit_index(n);
        self.joint[idx] & (1 << b_idx) != 0
    }
}

impl Input {
    fn bit_index(&self, n: usize) -> (usize, usize) {
        let idx = n / (bits::BITS as usize);
        let b_idx = n % (bits::BITS as usize);
        (idx, b_idx)
    }
    fn len(&self) -> usize {
        self.kind.len()
    }
}
