use crate::interface::with;
use std::fmt::{Debug, Display, Formatter};
use crate::Error;

pub type BytePos = usize;

#[derive(Clone, Copy)]
pub struct SourceFile {
    pub(crate) handle: usize,
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

    pub fn read(&self) -> String {
        with(|w| w.file_content(self))
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
