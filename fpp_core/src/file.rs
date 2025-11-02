use crate::interface::with;
use crate::{Error, Span};
use std::cell::Ref;
use std::fmt::{Debug, Display, Formatter};

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
        f.write_str(self.path().as_str())
    }
}

impl Display for SourceFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.path().as_str())
    }
}

impl SourceFile {
    pub fn open(path: &str) -> Result<SourceFile, Error> {
        with(|w| w.file_open(path))
    }

    pub fn path(&self) -> String {
        with(|w| w.file_path(self))
    }

    pub fn read(&self) -> SourceFileContent {
        with(|w| SourceFileContent {
            data: w.file_content(self),
        })
    }

    pub fn lines(&self) -> Ref<Vec<BytePos>> {
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

impl From<&str> for SourceFile {
    fn from(value: &str) -> Self {
        with(|w| w.file_from(value))
    }
}
