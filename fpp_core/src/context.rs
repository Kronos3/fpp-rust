use crate::diagnostic::DiagnosticMessage;
use crate::error::Error;
use crate::file::SourceFile;
use crate::span::Span;
use crate::{BytePos, Diagnostic, DiagnosticMessageKind, Level, Node, Position};
use rustc_hash::FxHashMap as HashMap;
use std::cell::{Ref, RefCell};
use std::fs;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct SpanData {
    pub handle: usize,
    pub file: Rc<SourceFileData>,
    pub start: BytePos,
    pub length: BytePos,
    pub include_span: Option<Box<SpanData>>,
}

impl SpanData {
    pub fn snippet(&'_ self) -> DiagnosticDataSnippet<'_> {
        self.file.snippet(self)
    }
}

#[derive(Debug)]
pub struct SourceFileData {
    pub handle: usize,
    pub path: Option<String>,
    pub content: String,
    pub lines: Vec<BytePos>,
}

impl SourceFileData {
    fn new(handle: usize, path: Option<String>, content: String) -> SourceFileData {
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

    pub fn path(&self) -> Option<String> {
        match &self.path {
            None => None,
            Some(path) => Some(path.clone()),
        }
    }

    pub fn position(&self, offset: BytePos) -> Position {
        let line = match self.lines.binary_search(&offset) {
            // End of the line, it's actually on the line before
            Ok(line_idx) => match line_idx {
                0 => 0,
                _ => line_idx - 1,
            },
            // Somewhere in the middle of the last
            Err(line_insertion_point) => line_insertion_point - 1,
        };
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

    pub fn include_loc(&self, span: &SpanData) -> DiagnosticDataIncludeLocation {
        let pos = self.position(span.start);

        DiagnosticDataIncludeLocation {
            line: pos.line,
            column: pos.column,
            file_path: self.path().unwrap_or_else(|| "<stdin>".to_string()),
        }
    }

    pub fn snippet(&'_ self, span: &SpanData) -> DiagnosticDataSnippet<'_> {
        // Find the line start of the start/end
        let first_line = match self.lines.binary_search(&span.start) {
            // The span start is a newline
            Ok(newline_idx) => match newline_idx {
                0 => 0,
                _ => newline_idx - 1,
            },
            Err(insert_position) => insert_position - 1,
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

        // Collect all the include locations
        fn collect_include_locs(
            loc: &Option<Box<SpanData>>,
            out: &mut Vec<DiagnosticDataIncludeLocation>,
        ) {
            match loc {
                None => {}
                Some(loc) => {
                    out.push(loc.file.include_loc(loc));
                    collect_include_locs(&loc.include_span, out)
                }
            }
        }

        let mut include_spans = vec![];
        collect_include_locs(&span.include_span, &mut include_spans);

        DiagnosticDataSnippet {
            start: span.start - first,
            end: span.start + span.length - first,
            line_offset: first_line,
            file_path: match &self.path {
                None => "<stdin>",
                Some(path) => &path,
            },
            file_content: &self.content[first..last],
            include_spans,
        }
    }
}

pub(crate) struct NodeData {
    pub span_id: usize,
    pub pre_annotation: Vec<String>,
    pub post_annotation: Vec<String>,
}

#[derive(Debug)]
pub struct DiagnosticDataIncludeLocation {
    pub line: u32,
    pub column: u32,
    pub file_path: String,
}

#[derive(Debug)]
pub struct DiagnosticDataSnippet<'a> {
    pub start: BytePos,
    pub end: BytePos,
    pub line_offset: usize,
    pub file_path: &'a str,
    pub file_content: &'a str,
    pub include_spans: Vec<DiagnosticDataIncludeLocation>,
}

#[derive(Debug)]
pub struct DiagnosticMessageData {
    pub kind: DiagnosticMessageKind,
    pub message: String,
    pub span: Option<SpanData>,
}

#[derive(Debug)]
pub struct DiagnosticData {
    pub level: Level,
    pub message: String,
    pub span: SpanData,
    pub children: Vec<DiagnosticMessageData>,
}

pub trait DiagnosticEmitter {
    fn emit(&mut self, diagnostic: DiagnosticData);
}

pub struct CompilerContext<E: DiagnosticEmitter> {
    spans: Vec<SpanData>,
    files: Vec<Rc<SourceFileData>>,

    current_node_id: Node,
    nodes: HashMap<Node, NodeData>,
    emitter: RefCell<E>,
}

impl<'e, E: DiagnosticEmitter> CompilerContext<E> {
    pub fn new(emitter: E) -> CompilerContext<E> {
        CompilerContext {
            spans: vec![],
            files: vec![],
            current_node_id: Node { handle: 1 },
            nodes: HashMap::default(),
            emitter: RefCell::new(emitter),
        }
    }

    pub(crate) fn file_open(&mut self, path: &str) -> Result<SourceFile, Error> {
        let fs_path = std::path::Path::new(path).canonicalize()?;

        let handle = self.files.len();
        let content = match fs::read_to_string(&fs_path) {
            Ok(c) => c,
            Err(err) => return Err(Error(format!("failed to read file {}: {}", path, err))),
        };

        let path_str = match fs_path.to_str() {
            None => {
                return Err(Error(format!(
                    "failed to convert path to string: {:?}",
                    fs_path
                )));
            }
            Some(p) => p,
        };

        self.files.push(Rc::new(SourceFileData::new(
            handle,
            Some(path_str.to_string()),
            content,
        )));
        Ok(SourceFile { handle })
    }

    pub(crate) fn file_from(&mut self, src: &str) -> SourceFile {
        let handle = self.files.len();
        self.files
            .push(Rc::new(SourceFileData::new(handle, None, src.to_string())));

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

    pub(crate) fn span_add(
        &mut self,
        file: SourceFile,
        start: BytePos,
        length: BytePos,
        include_span: Option<Span>,
    ) -> Span {
        let handle = self.spans.len();
        self.spans.push(SpanData {
            handle,
            file: self.files.get(file.handle).unwrap().clone(),
            start,
            length,
            include_span: include_span.map(|s| Box::new(self.span_get(&s).clone())),
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
        DiagnosticMessageData {
            message: diagnostic.message,
            kind: diagnostic.kind,
            span: diagnostic.span.map(|s| self.span_get(&s).clone()),
        }
    }

    fn diagnostic_get(&self, diagnostic: Diagnostic) -> DiagnosticData {
        DiagnosticData {
            level: diagnostic.level,
            message: diagnostic.msg,
            span: self.span_get(&diagnostic.span).clone(),
            children: diagnostic
                .children
                .into_iter()
                .map(|child| self.diagnostic_message_get(child))
                .collect(),
        }
    }

    pub(crate) fn diagnostic_emit(&mut self, diag: Diagnostic) {
        // Convert a standard diagnostic to a flattened diagnostic
        // Send to the emitter
        self.emitter.borrow_mut().emit(self.diagnostic_get(diag));
    }

    pub fn diagnostics(&'_ self) -> Ref<'_, E> {
        self.emitter.borrow()
    }
}
