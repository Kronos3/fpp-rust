pub type BytePos = u32;

#[derive(Clone, Debug)]
pub struct SourceFile {
    file_id: u16,
}

impl SourceFile {
    pub fn read(&self) -> String {
        &self.contents
    }
}

pub struct SourceMap {
    files: Box<SourceFile>
}
