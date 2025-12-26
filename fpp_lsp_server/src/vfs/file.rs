use lsp_types::{
    DidChangeTextDocumentParams, DidOpenTextDocumentParams, Uri,
};
use std::path::PathBuf;

#[derive(Debug)]
pub struct FsFile {
    /// Path to the file on the filesystem
    pub path: PathBuf,
    /// Buffered file contents
    pub text: String,
}

#[derive(Debug)]
pub struct LspFile {
    /// LSP document version number to handle interleaved updated
    pub version: i32,
    /// LSP file URI
    pub uri: Uri,
    /// LSP managed file contents, may not have been committed to disk yet
    pub text: String,
}

#[derive(Debug)]
pub enum File {
    Fs(FsFile),
    Lsp(LspFile),
}

impl File {
    pub(crate) fn open_new(open: DidOpenTextDocumentParams) -> File {
        let key = open.text_document.uri.as_str();
        tracing::debug!(
            uri = key,
            version = open.text_document.version,
            "opening new lsp file"
        );

        File::Lsp(LspFile {
            version: open.text_document.version,
            uri: open.text_document.uri,
            text: open.text_document.text,
        })
    }

    pub(crate) fn open_over(self, open: DidOpenTextDocumentParams) -> File {
        match self {
            File::Fs(_) => {
                tracing::debug!(
                    uri = open.text_document.uri.to_string(),
                    version = open.text_document.version,
                    "opening lsp file over fs-watched file, dropping watch"
                );

                let file = File::Lsp(LspFile {
                    version: open.text_document.version,
                    uri: open.text_document.uri,
                    text: open.text_document.text,
                });

                file
            }
            File::Lsp(_) => {
                tracing::warn!(
                    uri = open.text_document.uri.to_string(),
                    version = open.text_document.version,
                    "opening lsp file over an already opened lsp file"
                );

                File::Lsp(LspFile {
                    version: open.text_document.version,
                    uri: open.text_document.uri,
                    text: open.text_document.text,
                })
            }
        }
    }

    pub(crate) fn update(self, mut change: DidChangeTextDocumentParams) -> File {
        match self {
            File::Fs(fs) => {
                tracing::warn!(
                    uri = change.text_document.uri.to_string(),
                    "received a change event to a file not being traced by the LSP, dropping event"
                );
                File::Fs(fs)
            }
            File::Lsp(_) => {
                // TODO(tumbar) Handle incremental changes
                // Currently the server only handles full file synchronization
                // Look at the final event and apply the full text to the Vfs
                File::Lsp(LspFile {
                    version: change.text_document.version,
                    uri: change.text_document.uri.clone(),
                    text: change.content_changes.pop().unwrap().text,
                })
            }
        }
    }
}
