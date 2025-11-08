use std::collections::VecDeque;
use std::fmt::{Debug, Formatter, Write};

pub struct QualifiedName {
    qualifier: VecDeque<String>,
    base: String,
}

impl QualifiedName {
    pub fn to_ident_list(&self) -> VecDeque<String> {
        let mut out = self.qualifier.clone();
        out.push_back(self.base.clone());
        out
    }
}

impl From<String> for QualifiedName {
    fn from(value: String) -> Self {
        QualifiedName {
            qualifier: VecDeque::new(),
            base: value,
        }
    }
}

impl From<Vec<String>> for QualifiedName {
    fn from(mut value: Vec<String>) -> Self {
        let base = value
            .pop()
            .expect("qualified name must have at least one token");
        value.reverse();
        QualifiedName {
            base,
            qualifier: value.into(),
        }
    }
}

impl From<&fpp_ast::QualIdent> for QualifiedName {
    fn from(value: &fpp_ast::QualIdent) -> Self {
        todo!()
    }
}

impl Debug for QualifiedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let v: Vec<String> = self.qualifier.clone().into();
        f.write_str(&v.join("."))?;
        f.write_char('.')?;
        f.write_str(&self.base)
    }
}
