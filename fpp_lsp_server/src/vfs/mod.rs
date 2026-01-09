mod file;
pub use file::*;

use fpp_core::{Error, SourceFile};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};
use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::lsp::capabilities::PositionEncoding;

#[derive(Clone)]
pub struct Vfs {
    files: Arc<RwLock<FxHashMap<String, File>>>,
}

impl Vfs {
    pub fn new() -> Vfs {
        Vfs {
            files: Default::default(),
        }
    }

    pub(crate) fn read_sync(&self, path: &PathBuf) -> anyhow::Result<String> {
        let key = path.to_string_lossy().to_string();
        match self.files.read().unwrap().get(&key) {
            None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("file not in vfs: {}", key),
            )
            .into()),
            Some(file) => Ok(file.content.text().to_string()),
        }
    }

    pub(crate) fn read(&mut self, path: &PathBuf) -> anyhow::Result<String> {
        let key = path.to_string_lossy().to_string();
        match self.files.read().unwrap().get(&key) {
            None => {}
            Some(file) => return Ok(file.content.text().to_string()),
        }

        let text = std::fs::read_to_string(&path)?;

        self.files.write().unwrap().insert(
            key,
            File::new(FileContent::Fs(FsFile {
                path: path.clone(),
                text: text.clone(),
            })),
        );

        Ok(text)
    }

    pub fn clear(&mut self) {
        let _ = self
            .files
            .write()
            .unwrap()
            .extract_if(|_, f| match &f.content {
                FileContent::Fs(_) => true,
                FileContent::Lsp(_) => false,
            });
    }

    pub fn did_open(&mut self, open: DidOpenTextDocumentParams) {
        let key = open.text_document.uri.path().to_string();
        let mut files = self.files.write().unwrap();
        let new_file = match files.remove(&key) {
            None => File::open_new(open),
            Some(old) => old.open_over(open),
        };

        files.insert(key, new_file);
    }

    pub fn did_change(&mut self, change: DidChangeTextDocumentParams, encoding: PositionEncoding) {
        let key = change.text_document.uri.path().to_string();
        let mut files = self.files.write().unwrap();

        let new_file = match files.remove(&key) {
            None => {
                tracing::warn!(
                    uri = key,
                    version = change.text_document.version,
                    "received didChange event for a file that hasn't been opened yet"
                );
                return;
            }
            Some(old) => old.update(change, encoding),
        };

        files.insert(key, new_file);
    }

    pub fn did_close(&mut self, close: DidCloseTextDocumentParams) {
        let uri = close.text_document.uri.clone();
        let key = close.text_document.uri.path().to_string();
        let mut files = self.files.write().unwrap();

        match files.remove(&key) {
            None => {
                tracing::warn!(
                    uri = key,
                    "received didClose event for a file that hasn't been opened yet"
                );
            }
            Some(file) => match file.content {
                FileContent::Fs(fs_file) => {
                    tracing::warn!(
                        uri = close.text_document.uri.to_string(),
                        "received a close event to a file not being traced by the LSP, dropping event"
                    );

                    files.insert(
                        key,
                        File {
                            content: FileContent::Fs(fs_file),
                            lines: file.lines,
                            parse: file.parse,
                        },
                    );
                }
                FileContent::Lsp(_) => {
                    tracing::info!(
                        uri = key,
                        "received didClose on LSP file, falling back to filesystem tracking"
                    );

                    // Read the file asynchronously
                    let mut this = self.clone();
                    tokio::task::spawn_blocking(move || {
                        let path_buf: PathBuf = uri.path().to_string().into();
                        match this.read(&path_buf) {
                            Ok(_) => {}
                            Err(err) => {
                                tracing::error!(
                                    uri = key,
                                    err = %err,
                                    "failed to read file {} into vfs",
                                    uri.path().to_string(),
                                );
                            }
                        }
                    });
                }
            },
        };
    }
}

impl fpp_core::FileReader for &Vfs {
    fn read(&self, path: &str) -> Result<SourceFile, Error> {
        let mut this = (*self).clone();
        match Vfs::read(&mut this, &path.into()) {
            Ok(text) => Ok(SourceFile::new(path, text)),
            Err(e) => Err(e.to_string().into()),
        }
    }
}
