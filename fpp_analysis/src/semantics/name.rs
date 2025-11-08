use std::fmt::{Debug, Formatter, Write};

pub struct QualifiedName {
    qualifier: Vec<String>,
    base: String,
}

impl From<String> for QualifiedName {
    fn from(value: String) -> Self {
        QualifiedName {
            qualifier: vec![],
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
            qualifier: value,
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
        f.write_str(&self.qualifier.join("."))?;
        f.write_char('.')?;
        f.write_str(&self.base)
    }
}
