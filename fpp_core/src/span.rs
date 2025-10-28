use crate::diagnostic::Diagnostic;
use crate::context::with;
use crate::diagnostic::Level;
use crate::file::SourceFile;

pub struct Span {
    handle: usize
}

macro_rules! diagnostic_method {
    ($name:ident, $level:expr) => {
        /// Creates a new `Diagnostic` with the given `message` at the span
        /// `self`.
        pub fn $name<T: Into<String>>(self, message: T) -> Diagnostic {
            Diagnostic::spanned(self, $level, message)
        }
    };
}

pub struct Position {
    pos: u32,
    source_file: SourceFile
}

impl Span {
    pub(crate) fn internal_new(handle: usize) -> Span {
        Span{handle}
    }

    pub fn new(
        file: SourceFile,
        start: u32,
        length: u32,
    ) -> Span {
        with(|w| w.add_span(file, start, length))
    }

    /// Gets the start position of the span
    pub fn start(&self) -> Position {
        with(|w| w)
    }

    /// Creates an empty span pointing to directly after this span.
    pub fn end(&self) -> Position {}

    /// The path to the source file in which this span occurs, for display purposes.
    pub fn file(&self) -> SourceFile {}

    diagnostic_method!(error, Level::Error);
    diagnostic_method!(warning, Level::Warning);
    diagnostic_method!(note, Level::Note);
    diagnostic_method!(help, Level::Help);
}

pub impl Position {
    /// Get the zero indexed line number at this position in the source file
    pub fn line(&self) -> u32 {

    }

    /// Get the zero indexed column number at this position in the source file
    pub fn column(&self) -> u32 {

    }
}
