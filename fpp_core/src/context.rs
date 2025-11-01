use crate::error::Error;
use crate::file::SourceFile;
use crate::span::Span;
use crate::{BytePos, Node, Position};
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
}

pub(crate) struct NodeData {
    pub span_id: usize,
    pub pre_annotation: Vec<String>,
    pub post_annotation: Vec<String>,
}

pub struct CompilerContext {
    spans: Vec<SpanData>,
    files: Vec<SourceFileData>,

    current_node_id: Node,
    nodes: HashMap<Node, NodeData>,
}

impl CompilerContext {
    pub fn new() -> CompilerContext {
        CompilerContext {
            spans: vec![],
            files: vec![],
            current_node_id: Node { handle: 1 },
            nodes: HashMap::new(),
        }
    }

    fn compute_lines(src: &str) -> Vec<BytePos> {
        let mut out = vec![0];
        for (i, c) in src.chars().enumerate() {
            if c == '\n' {
                out.push(BytePos::from(i))
            }
        }

        out
    }

    pub(crate) fn file_open(&mut self, path: &str) -> Result<SourceFile, Error> {
        let handle = self.files.len();
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => return Err(Error(format!("failed to read file: {}", err))),
        };
        let lines = Self::compute_lines(content.as_str());

        self.files.push(SourceFileData {
            handle,
            path: path.to_string(),
            content,
            lines,
        });

        Ok(SourceFile { handle })
    }

    pub(crate) fn file_from(&mut self, src: &str) -> SourceFile {
        let handle = self.files.len();

        self.files.push(SourceFileData {
            handle,
            path: "<input>".to_string(),
            content: src.to_string(),
            lines: CompilerContext::compute_lines(src),
        });

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
}
