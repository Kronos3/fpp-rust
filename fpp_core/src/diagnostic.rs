use crate::span::Span;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

pub struct Diagnostic {
    span: Span,
    level: Level,
    message: String,
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{:?}] {:?}\n", self.level, self.span))?;
        f.write_str(self.message.as_str())
    }
}

impl Diagnostic {
    pub fn spanned<T: Into<String>>(span: Span, level: Level, message: T) -> Diagnostic {
        Diagnostic {
            span,
            level,
            message: message.into(),
        }
    }
}
