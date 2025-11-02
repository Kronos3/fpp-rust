use crate::interface::with;
use crate::{Span, Spanned};

/// An enum representing a diagnostic level.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Level {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// A note.
    Note,
    /// A help message.
    Help,
}

#[derive(Clone, Debug)]
pub(crate) struct DiagnosticMessage {
    pub level: Level,
    pub message: String,
    pub span: Option<Span>,
}

/// A structure representing a diagnostic message and associated children
/// messages.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub(crate) msg: DiagnosticMessage,
    pub(crate) children: Vec<DiagnosticMessage>,
}

macro_rules! diagnostic_child_methods {
    ($spanned:ident, $regular:ident, $level:expr) => {
        #[doc = concat!("Adds a new child diagnostics message to `self` with the [`",
                        stringify!($level), "`] level, and the given `span` and `message`.")]
        pub fn $spanned<S, T>(mut self, span: S, message: T) -> Diagnostic
        where
            S: Spanned,
            T: Into<String>,
        {
            self.children.push(DiagnosticMessage {
                level: $level,
                message: message.into(),
                span: Some(span.span()),
            });
            self
        }

        #[doc = concat!("Adds a new child diagnostic message to `self` with the [`",
                        stringify!($level), "`] level, and the given `message`.")]
        pub fn $regular<T: Into<String>>(mut self, message: T) -> Diagnostic {
            self.children.push(DiagnosticMessage {
                level: $level,
                message: message.into(),
                span: None
            });
            self
        }
    };
}

impl Diagnostic {
    /// Creates a new diagnostic with the given `level` and `message`
    pub fn new<T>(level: Level, message: T) -> Diagnostic
    where
        T: Into<String>,
    {
        Diagnostic {
            msg: DiagnosticMessage {
                level,
                message: message.into(),
                span: None
            },
            children: vec![],
        }
    }

    /// Creates a new diagnostic with the given `level` and `message` pointing to
    /// the given set of `spans`.
    pub fn spanned<S, T>(span: S, level: Level, message: T) -> Diagnostic
    where
        S: Spanned,
        T: Into<String>,
    {
        Diagnostic {
            msg: DiagnosticMessage {
                level,
                message: message.into(),
                span: Some(span.span()),
            },
            children: vec![],
        }
    }

    diagnostic_child_methods!(span_error, error, Level::Error);
    diagnostic_child_methods!(span_warning, warning, Level::Warning);
    diagnostic_child_methods!(span_note, note, Level::Note);
    diagnostic_child_methods!(span_help, help, Level::Help);

    /// Returns the diagnostic `level` for `self`.
    pub fn level(&self) -> Level {
        self.msg.level
    }

    /// Sets the level in `self` to `level`.
    pub fn set_level(&mut self, level: Level) {
        self.msg.level = level;
    }

    /// Returns the message in `self`.
    pub fn message(&self) -> &str {
        &self.msg.message
    }

    /// Sets the message in `self` to `message`.
    pub fn set_message<T: Into<String>>(&mut self, message: T) {
        self.msg.message = message.into();
    }

    /// Returns the `Span` in `self`.
    pub fn span(&self) -> Option<Span> {
        self.msg.span
    }

    /// Sets the `Span`s in `self` to `spans`.
    pub fn set_span<S: Spanned>(&mut self, span: S) {
        self.msg.span = Some(span.span());
    }

    /// Emit the diagnostic.
    pub fn emit(self) {
        with(|w| w.diagnostic_emit(self));
    }
}
