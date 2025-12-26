use fpp_core::{Error, FileReader, SourceFile};
use std::fs;

pub struct FsReader {}

impl FileReader for FsReader {
    fn read(&self, path: &str) -> Result<SourceFile, Error> {
        let fs_path = std::path::Path::new(&path).canonicalize()?;

        let content = match fs::read_to_string(&fs_path) {
            Ok(c) => c,
            Err(err) => return Err(format!("failed to read file {}: {}", path, err).into()),
        };

        let path_str = match fs_path.to_str() {
            None => {
                return Err(format!("failed to convert path to string: {:?}", fs_path).into());
            }
            Some(p) => p,
        };

        Ok(SourceFile::new(path_str, content))
    }
}
