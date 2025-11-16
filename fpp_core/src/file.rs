use crate::interface::with;
use crate::{Error, Span};
use std::cell::Ref;
use std::fmt::{Debug, Display, Formatter};

pub trait FileReader {
    /// Read a file given the current FPP source file and the included FPP source file
    fn include(&self, current: SourceFile, include: &str) -> Result<SourceFile, Error>;

    /// Read a file given its path
    fn read(&self, path: &str) -> Result<SourceFile, Error>;
}

pub type BytePos = usize;

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct SourceFile {
    pub(crate) handle: usize,
}

pub struct SourceFileContent<'a> {
    data: Ref<'a, String>,
}

impl<'a> AsRef<str> for SourceFileContent<'a> {
    fn as_ref(&self) -> &str {
        self.data.as_str()
    }
}

impl Debug for SourceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.uri())
    }
}

impl Display for SourceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.uri())
    }
}

impl SourceFile {
    /// Create a new source file given a unique Universal-Resource-Identifier (URI) and it's contents.
    /// The URI is used by a [FileReader] to identify the source location.
    /// If there already is a file with this URI in the compiler, that file will be dropped and all
    /// its spans and AST nodes will be dropped.
    ///
    /// # Arguments
    ///
    /// * `uri`: File's unique URI
    /// * `content`: File content
    ///
    /// returns: SourceFile
    pub fn new(uri: &str, content: String) -> SourceFile {
        with(|w| w.file_new(uri, content))
    }

    pub fn get(uri: &str) -> Option<SourceFile> {
        with(|w| w.file_get(uri))
    }

    pub fn uri(&self) -> String {
        with(|w| w.file_uri(self))
    }

    pub fn drop(self) {
        with(|w| w.file_drop(self))
    }

    pub fn read(&self) -> SourceFileContent<'_> {
        with(|w| SourceFileContent {
            data: w.file_content(self),
        })
    }

    pub fn lines(&self) -> Ref<'_, Vec<BytePos>> {
        with(|w| w.file_lines(self))
    }

    pub fn read_snippet(&self, span: &Span) -> String {
        let start = span.start();
        let end = span.end();

        with(|w| {
            let content = &w.file_content(self);
            content.as_str()[(start.pos - start.column as usize)..end.pos].to_string()
        })
    }

    pub fn len(&self) -> usize {
        with(|w| w.file_len(self))
    }
}
