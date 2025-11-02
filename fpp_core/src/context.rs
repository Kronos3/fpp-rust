use crate::diagnostic::DiagnosticMessage;
use crate::error::Error;
use crate::file::SourceFile;
use crate::span::Span;
use crate::{BytePos, Diagnostic, Level, Node, Position};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;

pub(crate) struct SpanData {
    pub file: SourceFile,
    pub start: BytePos,
    pub length: BytePos,
}

pub(crate) struct SourceFileData {
    pub handle: usize,
    pub path: String,
    pub content: String,
    pub lines: Vec<BytePos>,
}

impl SourceFileData {
    fn new(handle: usize, path: String, content: String) -> SourceFileData {
        // Compute the newline position
        let lines = {
            let mut out = vec![0];
            for (i, c) in content.chars().enumerate() {
                if c == '\n' {
                    out.push(BytePos::from(i))
                }
            }

            out
        };

        SourceFileData {
            handle,
            path,
            content,
            lines,
        }
    }

    pub fn position(&self, offset: BytePos) -> Position {
        let line = self
            .lines
            .binary_search(&offset)
            .unwrap_or_else(|line_insert| line_insert - 1);
        let line_offset = *self.lines.get(line).unwrap();

        Position {
            pos: offset,
            line: line as u32,
            column: (offset - line_offset) as u32,
            source_file: SourceFile {
                handle: self.handle,
            },
        }
    }

    pub fn snippet(&self, span: &SpanData) -> DiagnosticDataSnippet {
        // Find the line start of the start/end
        let first_line = match self.lines.binary_search(&span.start) {
            // The span start is a newline
            Ok(newline_idx) => {
                match newline_idx {
                    0 => 0,
                    _ => newline_idx - 1
                }
            }
            Err(insert_position) => {
                insert_position - 1
            }
        };

        let last_line = self
            .lines
            .binary_search(&(span.start + span.length))
            .unwrap_or_else(|line_insert| line_insert);

        let first = *self.lines.get(first_line).unwrap();
        let last = match self.lines.get(last_line) {
            None => self.content.len(),
            Some(last) => *last + 1,
        };

        DiagnosticDataSnippet {
            start: span.start - first,
            end: span.start + span.length - first,
            line_offset: first_line,
            file_path: self.path.as_str(),
            file_content: &self.content.as_str()[first..last],
        }
    }
}

pub(crate) struct NodeData {
    pub span_id: usize,
    pub pre_annotation: Vec<String>,
    pub post_annotation: Vec<String>,
}

#[derive(Debug)]
pub struct DiagnosticDataSnippet<'a> {
    pub start: BytePos,
    pub end: BytePos,
    pub line_offset: usize,
    pub file_path: &'a str,
    pub file_content: &'a str,
}

#[derive(Debug)]
pub struct DiagnosticMessageData<'a> {
    pub level: Level,
    pub message: String,
    pub snippet: Option<DiagnosticDataSnippet<'a>>,
}

#[derive(Debug)]
pub struct DiagnosticData<'a> {
    pub message: DiagnosticMessageData<'a>,
    pub children: Vec<DiagnosticMessageData<'a>>,
}

pub trait DiagnosticEmitter {
    fn emit<'d>(&'_ mut self, diagnostic: DiagnosticData<'d>);
}

pub struct CompilerContext<E: DiagnosticEmitter> {
    spans: Vec<SpanData>,
    files: Vec<SourceFileData>,

    current_node_id: Node,
    nodes: HashMap<Node, NodeData>,

    emitter: RefCell<E>,
    seen_errors: bool
}

impl<E: DiagnosticEmitter> CompilerContext<E> {
    pub fn new(emitter: E) -> CompilerContext<E> {
        CompilerContext {
            spans: vec![],
            files: vec![],
            current_node_id: Node { handle: 1 },
            nodes: HashMap::new(),
            emitter: RefCell::new(emitter),
            seen_errors: false,
        }
    }

    pub(crate) fn file_open(&mut self, path: &str) -> Result<SourceFile, Error> {
        let handle = self.files.len();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => return Err(Error(format!("failed to read file {}: {}", path, err))),
        };

        self.files
            .push(SourceFileData::new(handle, path.to_string(), content));
        Ok(SourceFile { handle })
    }

    pub(crate) fn file_from(&mut self, src: &str) -> SourceFile {
        let handle = self.files.len();
        self.files.push(SourceFileData::new(
            handle,
            "input".to_string(),
            src.to_string(),
        ));

        SourceFile { handle }
    }

    pub(crate) fn node_add(&mut self, span: &Span) -> Node {
        let node_id = self.current_node_id;
        self.current_node_id = self.current_node_id.next();
        self.nodes.insert(
            node_id,
            NodeData {
                span_id: span.handle,
                pre_annotation: vec![],
                post_annotation: vec![],
            },
        );
        node_id
    }

    pub(crate) fn span_add(&mut self, file: SourceFile, start: BytePos, length: BytePos) -> Span {
        let handle = self.spans.len();
        self.spans.push(SpanData {
            file,
            start,
            length,
        });

        Span { handle }
    }

    pub(crate) fn node_get(&self, node: &Node) -> &NodeData {
        self.nodes
            .get(node)
            .expect(&format!("invalid node: {}", node.handle))
    }

    pub(crate) fn node_get_mut(&mut self, node: &Node) -> &mut NodeData {
        self.nodes
            .get_mut(node)
            .expect(&format!("invalid node: {}", node.handle))
    }

    pub(crate) fn node_get_span(&self, node: &Node) -> Span {
        Span {
            handle: self
                .nodes
                .get(node)
                .expect(&format!("invalid node: {}", node.handle))
                .span_id,
        }
    }

    pub(crate) fn span_get(&self, span: &Span) -> &SpanData {
        self.spans
            .get(span.handle)
            .expect(&format!("invalid span: {}", span.handle))
    }

    pub(crate) fn file_get(&self, file: &SourceFile) -> &SourceFileData {
        self.files
            .get(file.handle)
            .expect(&format!("invalid file: {}", file.handle))
    }

    fn diagnostic_message_get(&self, diagnostic: DiagnosticMessage) -> DiagnosticMessageData {
        let snippet = match diagnostic.span {
            None => None,
            Some(span) => {
                let span = self.span_get(&span);
                let file = self.file_get(&span.file);
                Some(file.snippet(span))
            }
        };

        DiagnosticMessageData {
            snippet,
            message: diagnostic.message,
            level: diagnostic.level,
        }
    }

    fn diagnostic_get(&self, diagnostic: Diagnostic) -> DiagnosticData {
        DiagnosticData {
            message: self.diagnostic_message_get(diagnostic.msg),
            children: diagnostic
                .children
                .into_iter()
                .map(|child| self.diagnostic_message_get(child))
                .collect(),
        }
    }

    pub(crate) fn diagnostic_emit(&mut self, diag: Diagnostic) {
        match diag.msg.level {
            Level::Error => {
                self.seen_errors = true;
            }
            _ => {}
        }

        // Convert a standard diagnostic to a flattened diagnostic
        // Send to the emitter
        self.emitter.borrow_mut().emit(self.diagnostic_get(diag));
    }

    pub fn has_errors(&self) -> bool {
        self.seen_errors
    }
}
