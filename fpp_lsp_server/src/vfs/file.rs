use fpp_core::RawFileLines;
use fpp_lsp_parser::Parse;
use lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams, Uri};
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
pub enum FileContent {
    Fs(FsFile),
    Lsp(LspFile),
}

pub struct File {
    pub content: FileContent,
    pub lines: fpp_core::RawFileLines,
    pub parse: Parse,
    // semantic_tokens: lsp_types::SemanticTokens,
}

impl FileContent {
    pub(crate) fn text(&self) -> &str {
        match self {
            FileContent::Fs(fs_file) => &fs_file.text,
            FileContent::Lsp(lsp_file) => &lsp_file.text,
        }
    }

    pub(crate) fn open_new(open: DidOpenTextDocumentParams) -> FileContent {
        let key = open.text_document.uri.as_str();
        tracing::debug!(
            uri = key,
            version = open.text_document.version,
            "opening new lsp file"
        );

        FileContent::Lsp(LspFile {
            version: open.text_document.version,
            uri: open.text_document.uri,
            text: open.text_document.text,
        })
    }

    pub(crate) fn open_over(self, open: DidOpenTextDocumentParams) -> FileContent {
        match self {
            FileContent::Fs(_) => {
                tracing::debug!(
                    uri = open.text_document.uri.to_string(),
                    version = open.text_document.version,
                    "opening lsp file over fs-watched file, dropping watch"
                );

                let file = FileContent::Lsp(LspFile {
                    version: open.text_document.version,
                    uri: open.text_document.uri,
                    text: open.text_document.text,
                });

                file
            }
            FileContent::Lsp(_) => {
                tracing::warn!(
                    uri = open.text_document.uri.to_string(),
                    version = open.text_document.version,
                    "opening lsp file over an already opened lsp file"
                );

                FileContent::Lsp(LspFile {
                    version: open.text_document.version,
                    uri: open.text_document.uri,
                    text: open.text_document.text,
                })
            }
        }
    }
}

impl File {
    pub(crate) fn new(content: FileContent) -> File {
        let lines = RawFileLines::new(content.text());
        let parse = fpp_lsp_parser::parse(content.text());
        File {
            content,
            lines,
            parse,
        }
    }

    pub(crate) fn open_new(open: DidOpenTextDocumentParams) -> File {
        File::new(FileContent::open_new(open))
    }

    pub(crate) fn open_over(self, open: DidOpenTextDocumentParams) -> File {
        File::new(FileContent::open_over(self.content, open))
    }

    pub(crate) fn update(self, change: DidChangeTextDocumentParams) -> File {
        match self.content {
            FileContent::Fs(fs) => {
                tracing::warn!(
                    uri = change.text_document.uri.to_string(),
                    "received a change event to a file not being traced by the LSP, dropping event"
                );

                File {
                    content: FileContent::Fs(fs),
                    lines: self.lines,
                    parse: self.parse,
                }
            }
            FileContent::Lsp(f) => {
                let mut text = f.text;
                for event in change.content_changes {
                    match event.range {
                        Some(delta_range) => {
                            let start = self
                                .lines
                                .position_of(delta_range.start.line, delta_range.start.character);
                            let end = self
                                .lines
                                .position_of(delta_range.end.line, delta_range.end.character);

                            text.replace_range(start as usize..end as usize, &event.text);
                        }
                        None => {
                            text = event.text;
                        }
                    }
                }

                File::new(FileContent::Lsp(LspFile {
                    version: change.text_document.version,
                    uri: change.text_document.uri.clone(),
                    text,
                }))
            }
        }
    }
}
