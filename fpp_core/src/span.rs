use crate::BytePos;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::Level;
use crate::file::SourceFile;
use crate::interface::with;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Copy)]
pub struct Span {
    pub(crate) handle: usize,
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let file = self.file();
        let start = self.start();
        let end = self.end();
        f.write_str("Span\n")?;
        f.write_fmt(format_args!(
            "start {}:{}:{}\n",
            file.path(),
            start.line(),
            start.column()
        ))?;
        f.write_fmt(format_args!(
            "end {}:{}:{}\n",
            file.path(),
            end.line(),
            end.column()
        ))?;

        Ok(())
    }
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

impl Span {
    pub fn new(file: SourceFile, start: BytePos, length: BytePos) -> Span {
        with(|w| w.span_add(file, start, length))
    }

    /// Gets the start position of the span
    pub fn start(&self) -> Position {
        with(|w| w.span_start(self))
    }

    /// Creates an empty span pointing to directly after this span.
    pub fn end(&self) -> Position {
        with(|w| w.span_end(self))
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

#[derive(Debug)]
pub struct Position {
    pub(crate) pos: BytePos,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) source_file: SourceFile,
}

impl Position {
    pub fn start(source_file: SourceFile) -> Position {
        Position {
            pos: 0,
            line: 0,
            column: 0,
            source_file,
        }
    }

    pub fn pos(&self) -> BytePos {
        self.pos
    }

    /// Get the zero indexed line number at this position in the source file
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Get the zero indexed column number at this position in the source file
    pub fn column(&self) -> u32 {
        self.column
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{}:{}",
            self.source_file,
            self.line + 1,
            self.column + 1
        ))
    }
}
