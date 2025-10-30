use crate::context::with;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::Level;
use crate::file::SourceFile;

#[derive(Clone, Copy)]
pub struct Span {
    handle: usize,
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

#[derive(Debug)]
pub struct Position {
    pos: u32,
    source_file: SourceFile,
}

impl Span {
    pub(crate) fn internal_new(handle: usize) -> Span {
        Span { handle }
    }

    pub fn eof(source_file: SourceFile) -> Span {
        todo!()
    }

    pub fn new(file: SourceFile, start: u32, length: u32) -> Span {
        with(|w| w.add_span(file, start, length))
    }

    /// Gets the start position of the span
    pub fn start(&self) -> Position {
        let pos = with(|w| w.span_start(self));
        let source_file = with(|w| w.span_file(self));

        Position { pos, source_file }
    }

    /// Creates an empty span pointing to directly after this span.
    pub fn end(&self) -> Position {
        let pos = with(|w| w.span_end(self));
        let source_file = with(|w| w.span_file(self));

        Position { pos, source_file }
    }

    /// The path to the source file in which this span occurs, for display purposes.
    pub fn file(&self) -> SourceFile {
        with(|w| w.span_file(self))
    }

    diagnostic_method!(error, Level::Error);
    diagnostic_method!(warning, Level::Warning);
    diagnostic_method!(note, Level::Note);
    diagnostic_method!(help, Level::Help);
}

impl Position {
    pub fn start(source_file: SourceFile) -> Position {
        Position {
            pos: 0,
            source_file,
        }
    }

    pub fn pos(&self) -> u32 {
        self.pos
    }

    /// Get the zero indexed line number at this position in the source file
    pub fn line(&self) -> u32 {
        0
    }

    /// Get the zero indexed column number at this position in the source file
    pub fn column(&self) -> u32 {
        0
    }
}
