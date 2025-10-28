use crate::span::Span;

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

impl Diagnostic {
    pub fn spanned<T: Into<String>>(span: Span, level: Level, message: T) -> Diagnostic {
        Diagnostic {
            span,
            level,
            message: message.into(),
        }
    }
}
