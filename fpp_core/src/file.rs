pub type BytePos = u32;

#[derive(Clone, Copy, Debug)]
pub struct SourceFile {
    file_id: u16,
}

impl SourceFile {
    pub fn read(&self) -> String {
        "".to_string()
    }

    pub fn len(&self) -> usize {
        0
    }
}

pub struct SourceMap {
    files: Box<SourceFile>
}
