use crate::diagnostic::DiagnosticMessage;
use crate::file::SourceFile;
use crate::map::IdMap;
use crate::span::Span;
use crate::{BytePos, Diagnostic, DiagnosticMessageKind, Level, Node, Position};
use rustc_hash::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Clone, Debug)]
pub struct SpanData {
    pub handle: usize,
    pub file: Weak<SourceFileData>,
    pub start: BytePos,
    pub length: BytePos,
    pub include_span: Option<Box<SpanData>>,

    /// Exposed outside the weak reference for performance
    file_handle: usize,
}

impl SpanData {
    pub fn snippet(&self) -> DiagnosticDataSnippet {
        self.file.upgrade().unwrap().snippet(self)
    }
}

#[derive(Default, Debug)]
pub struct RawFilePosition {
    pub pos: usize,
    pub line: u32,
    pub column: u32,
}

pub struct RawFileLines(Vec<BytePos>);

impl RawFileLines {
    pub fn new(file: &str) -> RawFileLines {
        let lines = {
            let mut out = vec![0];
            for (i, c) in file.chars().enumerate() {
                if c == '\n' {
                    out.push(BytePos::from(i) + 1)
                }
            }

            out
        };

        RawFileLines(lines)
    }

    pub fn position(&self, offset: usize) -> RawFilePosition {
        let line = match self.0.binary_search(&offset) {
            // End of the line, it's actually on the line before
            Ok(line_idx) => match line_idx {
                0 => 0,
                _ => line_idx - 1,
            },
            // Somewhere in the middle of the last
            Err(line_insertion_point) => line_insertion_point - 1,
        };
        let line_offset = *self.0.get(line).unwrap();

        RawFilePosition {
            pos: offset,
            line: line as u32,
            column: (offset - line_offset) as u32,
        }
    }
}

pub struct SourceFileData {
    pub handle: usize,
    pub uri: String,
    pub content: String,
    pub lines: RawFileLines,
}

impl SourceFileData {
    fn new(handle: usize, uri: String, content: String) -> SourceFileData {
        let lines = RawFileLines::new(&content);

        SourceFileData {
            handle,
            uri,
            content,
            lines,
        }
    }

    pub fn position(&self, offset: BytePos) -> Position {
        let raw_position = self.lines.position(offset);
        Position {
            pos: raw_position.pos,
            line: raw_position.line,
            column: raw_position.column,
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
            uri: self.uri.clone(),
        }
    }

    pub fn snippet(&self, span: &SpanData) -> DiagnosticDataSnippet {
        // Find the line start of the start/end
        let first_line = match self.lines.0.binary_search(&span.start) {
            // The span start is a newline
            Ok(newline_idx) => match newline_idx {
                0 => 0,
                _ => newline_idx - 1,
            },
            Err(insert_position) => insert_position - 1,
        };

        let last_line = self
            .lines
            .0
            .binary_search(&(span.start + span.length))
            .unwrap_or_else(|line_insert| line_insert);

        let first = *self.lines.0.get(first_line).unwrap();
        let last = match self.lines.0.get(last_line) {
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
                    out.push(loc.file.upgrade().unwrap().include_loc(loc));
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
            uri: self.uri.clone(),
            file_content: self.content[first..last].into(),
            include_spans,
        }
    }
}

pub(crate) struct NodeData {
    pub span_handle: usize,
    pub pre_annotation: Vec<String>,
    pub post_annotation: Vec<String>,
}

#[derive(Debug)]
pub struct DiagnosticDataIncludeLocation {
    pub line: u32,
    pub column: u32,
    pub uri: String,
}

#[derive(Debug)]
pub struct DiagnosticDataSnippet {
    pub start: BytePos,
    pub end: BytePos,
    pub line_offset: usize,
    pub uri: String,
    pub file_content: String,
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
    spans: IdMap<SpanData>,
    files: IdMap<Rc<SourceFileData>>,
    file_uris: FxHashMap<String, usize>,
    nodes: IdMap<NodeData>,
    emitter: Rc<RefCell<E>>,
}

impl<E: DiagnosticEmitter> CompilerContext<E> {
    pub fn new(emitter: Rc<RefCell<E>>) -> CompilerContext<E> {
        CompilerContext {
            spans: Default::default(),
            files: Default::default(),
            file_uris: Default::default(),
            nodes: Default::default(),
            emitter,
        }
    }

    pub(crate) fn file_new(&mut self, uri: &str, content: String) -> SourceFile {
        // If this file uri is overlapping, remove it and refresh the file contents
        if let Some(old) = self.file_uris.get(uri) {
            self.file_drop(SourceFile {
                handle: old.clone(),
            });
        }

        let handle = self
            .files
            .push_with(|handle| Rc::new(SourceFileData::new(handle, uri.to_string(), content)));

        SourceFile { handle }
    }

    pub(crate) fn file_drop(&mut self, file: SourceFile) {
        // Clean up anything pointing to this file
        let old = self.files.remove(file.handle);
        let removed_spans = FxHashSet::from_iter(self.spans.retain(|_, v| {
            if file.handle == v.file_handle {
                // Span is in the file
                return false;
            } else {
                // Check if file includes this span
                let mut included_loc = &v.include_span;
                while included_loc.is_some() {
                    let l = included_loc.as_ref().unwrap();
                    if l.file_handle == file.handle {
                        return false;
                    }

                    included_loc = &l.include_span
                }

                true
            }
        }));

        self.nodes
            .retain(|_, v| !removed_spans.contains(&v.span_handle));

        self.file_uris.remove(&old.uri);
    }

    pub(crate) fn file_get_from_uri(&self, uri: &str) -> Option<SourceFile> {
        self.file_uris
            .get(uri)
            .map(|s| SourceFile { handle: s.clone() })
    }

    pub(crate) fn node_add(&mut self, span: &Span) -> Node {
        let handle = self.nodes.push(NodeData {
            span_handle: span.handle,
            pre_annotation: vec![],
            post_annotation: vec![],
        });

        Node { handle }
    }

    pub(crate) fn span_add(
        &mut self,
        file: SourceFile,
        start: BytePos,
        length: BytePos,
        include_span: Option<Span>,
    ) -> Span {
        let file = self.files.get(file.handle);
        let include_span = include_span.map(|s| Box::new(self.span_get(&s).clone()));
        let handle = self.spans.push_with(|handle| SpanData {
            handle,
            file: Rc::downgrade(file),
            start,
            length,
            include_span,
            file_handle: file.handle,
        });

        Span { handle }
    }

    pub(crate) fn node_get(&self, node: &Node) -> &NodeData {
        self.nodes.get(node.handle)
    }

    pub(crate) fn node_get_mut(&mut self, node: &Node) -> &mut NodeData {
        self.nodes.get_mut(node.handle)
    }

    pub(crate) fn node_get_span(&self, node: &Node) -> Span {
        Span {
            handle: self.nodes.get(node.handle).span_handle,
        }
    }

    pub(crate) fn span_get(&self, span: &Span) -> &SpanData {
        self.spans.get(span.handle)
    }

    pub(crate) fn file_get(&self, file: &SourceFile) -> &SourceFileData {
        self.files.get(file.handle)
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
}
