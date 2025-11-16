use fpp_core::{Error, FileReader, SourceFile};
use std::fs;

pub struct FsReader {}

impl FileReader for FsReader {
    fn include(&self, current: SourceFile, include: &str) -> Result<SourceFile, Error> {
        let current_path = if current.uri() == "<stdin>" {
            // Read from relative to the current directory
            return self.read(include);
        } else {
            current.uri()
        };

        let parent_file_path = std::path::Path::new(&current_path).canonicalize()?;
        match parent_file_path.parent() {
            None => Err(format!("Cannot resolve parent directory of {}", &current_path).into()),
            Some(parent_dir) => {
                let final_path = parent_dir.join(include);
                match final_path.as_path().to_str() {
                    None => Err(format!(
                        "Failed to resolve path {} relative to {:?}",
                        include, parent_dir
                    )
                    .into()),
                    Some(file_path) => self.read(file_path),
                }
            }
        }
    }

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
