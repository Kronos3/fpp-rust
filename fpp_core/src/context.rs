use crate::diagnostic::DiagnosticMessage;
use crate::file::SourceFile;
use crate::map::IdMap;
use crate::span::Span;
use crate::{BytePos, Diagnostic, DiagnosticMessageKind, Level, Node, Position};
use line_index::LineIndex;
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::{Arc, Mutex, Weak};

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

pub struct SourceFileData {
    pub handle: usize,
    pub uri: String,
    pub content: String,
    pub lines: LineIndex,
    pub parent: Option<usize>,
}

impl SourceFileData {
    fn new(
        handle: usize,
        uri: String,
        content: String,
        parent: Option<SourceFile>,
    ) -> SourceFileData {
        let lines = LineIndex::new(&content);

        SourceFileData {
            handle,
            uri,
            content,
            lines,
            parent: parent.map(|p| p.handle),
        }
    }

    pub fn position(&self, offset: BytePos) -> Position {
        let raw_position = self.lines.line_col(offset.into());
        Position {
            pos: offset,
            line: raw_position.line,
            column: raw_position.col,
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
        let first_line_i = self.lines.line_col(span.start.into()).line;
        let last_line_i = self.lines.line_col((span.start + span.length).into()).line;

        let first_line = self.lines.line(first_line_i).unwrap();
        let last_line = self.lines.line(last_line_i).unwrap();
        let full_range = first_line.cover(last_line);

        let first: BytePos = first_line.start().into();

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
            line_offset: first_line_i as usize,
            uri: self.uri.clone(),
            file_content: self.content[full_range.start().into()..full_range.end().into()].into(),
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
    files: IdMap<Arc<SourceFileData>>,
    file_uris: FxHashMap<String, usize>,
    nodes: IdMap<NodeData>,
    emitter: Arc<Mutex<E>>,
}

impl<E: DiagnosticEmitter> CompilerContext<E> {
    pub fn new(emitter: Arc<Mutex<E>>) -> CompilerContext<E> {
        CompilerContext {
            spans: Default::default(),
            files: Default::default(),
            file_uris: Default::default(),
            nodes: Default::default(),
            emitter,
        }
    }

    pub(crate) fn file_new(
        &mut self,
        uri: &str,
        content: String,
        parent: Option<SourceFile>,
    ) -> SourceFile {
        if let Some(old) = self.file_uris.get(uri).cloned() {
            self.file_drop(SourceFile { handle: old });
            let handle = self.files.push_with(|handle| {
                Arc::new(SourceFileData::new(
                    handle,
                    uri.to_string(),
                    content,
                    parent,
                ))
            });

            // Handles should stay identical
            assert_eq!(handle, old);
            self.file_uris.insert(uri.to_string(), handle);
            SourceFile { handle }
        } else {
            let handle = self.files.push_with(|handle| {
                Arc::new(SourceFileData::new(
                    handle,
                    uri.to_string(),
                    content,
                    parent,
                ))
            });

            self.file_uris.insert(uri.to_string(), handle);

            SourceFile { handle }
        }
    }

    pub(crate) fn file_drop(&mut self, file: SourceFile) {
        // Clean up anything pointing to this file
        let removed_spans =
            FxHashSet::from_iter(self.spans.retain(|_, v| file.handle != v.file_handle));

        self.nodes
            .retain(|_, v| !removed_spans.contains(&v.span_handle));

        // Remove all files that are parented by this file
        let files_to_remove: Vec<usize> = self
            .files
            .iter()
            .filter(|(_, v)| {
                v.parent
                    .map_or_else(|| false, |parent| parent == file.handle)
            })
            .map(|(k, _)| *k)
            .collect();

        for handle in files_to_remove {
            self.file_drop(SourceFile { handle })
        }

        // Clean up the file itself
        // This should be done last so that the final retained source file id
        // is sitting at the of the 'reuse' stack.
        let old = self.files.remove(file.handle);
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
            file: Arc::downgrade(file),
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
        self.emitter.lock().unwrap().emit(self.diagnostic_get(diag));
    }
}
