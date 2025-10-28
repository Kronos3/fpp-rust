pub type BytePos = u32;

pub struct SourceFile {
    file_id: u16,
}

impl SourceFile {
    pub fn read(&self) -> &str {
        &self.contents
    }
}

pub struct SourceMap {
    files: Box<SourceFile>
}
