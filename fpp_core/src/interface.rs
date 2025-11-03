use crate::context::CompilerContext;
use crate::error::Error;
use crate::{BytePos, Diagnostic, DiagnosticEmitter, Node, Position, SourceFile, Span};
use std::cell::{Cell, Ref, RefCell};

pub struct Container<'ctx, E: DiagnosticEmitter> {
    ctx: RefCell<&'ctx mut CompilerContext<E>>,
}

impl<'ctx, E: DiagnosticEmitter> Container<'ctx, E> {
    pub fn new(ctx: &'ctx mut CompilerContext<E>) -> Container<'ctx, E> {
        Container {
            ctx: RefCell::new(ctx),
        }
    }
}

impl<'ctx, E: DiagnosticEmitter> CompilerInterface for Container<'ctx, E> {
    fn node_add(&self, span: &Span) -> Node {
        self.ctx.borrow_mut().node_add(span)
    }

    fn node_span(&self, node: &Node) -> Span {
        self.ctx.borrow().node_get_span(node)
    }

    fn node_pre_annotation(&self, node: &Node) -> Vec<String> {
        self.ctx.borrow().node_get(node).pre_annotation.clone()
    }

    fn node_post_annotation(&self, node: &Node) -> Vec<String> {
        self.ctx.borrow().node_get(node).post_annotation.clone()
    }

    fn node_add_annotation(&self, node: &Node, pre: Vec<String>, post: Vec<String>) {
        let mut ctx = self.ctx.borrow_mut();
        let node = ctx.node_get_mut(node);
        node.pre_annotation = pre;
        node.post_annotation = post;
    }

    fn file_open(&self, path: &str) -> Result<SourceFile, Error> {
        self.ctx.borrow_mut().file_open(path)
    }

    fn file_from(&self, content: &str) -> SourceFile {
        self.ctx.borrow_mut().file_from(content)
    }

    fn file_path(&self, file: &SourceFile) -> String {
        self.ctx.borrow().file_get(file).path.to_string()
    }

    fn file_content(&self, file: &SourceFile) -> Ref<'_, String> {
        // self.ctx.borrow().file_get(file).content.clone()
        let ctx = self.ctx.borrow();
        Ref::map(ctx, |c| &c.file_get(file).content)
    }

    fn file_lines(&self, file: &SourceFile) -> Ref<'_, Vec<BytePos>> {
        let ctx = self.ctx.borrow();
        Ref::map(ctx, |c| &c.file_get(file).lines)
    }

    fn file_len(&self, file: &SourceFile) -> usize {
        self.ctx.borrow().file_get(file).content.len()
    }

    fn span_add(&self, file: SourceFile, start: BytePos, length: BytePos) -> Span {
        self.ctx.borrow_mut().span_add(file, start, length)
    }

    fn span_start(&self, s: &Span) -> Position {
        let ctx = self.ctx.borrow();
        let data = ctx.span_get(s);
        ctx.file_get(&data.file).position(data.start)
    }

    fn span_end(&self, s: &Span) -> Position {
        let ctx = self.ctx.borrow();
        let data = ctx.span_get(s);
        ctx.file_get(&data.file)
            .position(data.start + (data.length as BytePos))
    }

    fn span_file(&self, s: &Span) -> SourceFile {
        let ctx = self.ctx.borrow();
        ctx.span_get(s).file
    }

    fn diagnostic_emit(&self, diag: Diagnostic) {
        self.ctx.borrow_mut().diagnostic_emit(diag)
    }
}

pub(crate) trait CompilerInterface {
    /** Ast Node related functions */
    fn node_add(&self, span: &Span) -> Node;
    fn node_span(&self, node: &Node) -> Span;
    fn node_pre_annotation(&self, node: &Node) -> Vec<String>;
    fn node_post_annotation(&self, node: &Node) -> Vec<String>;
    fn node_add_annotation(&self, node: &Node, pre: Vec<String>, post: Vec<String>);

    /** Source file related functions */
    fn file_open(&self, path: &str) -> Result<SourceFile, Error>;
    fn file_from(&self, content: &str) -> SourceFile;
    fn file_path(&self, file: &SourceFile) -> String;
    fn file_content(&self, file: &SourceFile) -> Ref<'_, String>;
    fn file_lines(&self, file: &SourceFile) -> Ref<'_, Vec<BytePos>>;
    fn file_len(&self, file: &SourceFile) -> usize;

    /** Span related functions */
    fn span_add(&self, file: SourceFile, start: BytePos, length: BytePos) -> Span;
    fn span_start(&self, s: &Span) -> Position;
    fn span_end(&self, s: &Span) -> Position;
    fn span_file(&self, s: &Span) -> SourceFile;

    /** Diagnostic related functions */
    fn diagnostic_emit(&self, diag: Diagnostic);
}

// A thread local variable that stores a pointer to [`CompilerInterface`].
scoped_tls::scoped_thread_local!(static TLV: Cell<*const ()>);

/// Run the compiler under a closure with a compiler context
///
/// # Arguments
///
/// * `ctx`: Context to attach to the core compiler
/// * `f`: Function closure to run
pub fn run<F, T, E>(ctx: &mut CompilerContext<E>, f: F) -> Result<T, Error>
where
    F: FnOnce() -> T,
    E: DiagnosticEmitter,
{
    let container = Container::new(ctx);
    run1(&container, f)
}

pub(crate) fn run1<F, T>(interface: &dyn CompilerInterface, f: F) -> Result<T, Error>
where
    F: FnOnce() -> T,
{
    if TLV.is_set() {
        Err(Error::from("fpp_core already running"))
    } else {
        let ptr: *const () = (&raw const interface) as _;
        TLV.set(&Cell::new(ptr), || Ok(f()))
    }
}

/// Execute the given function with access the [`CompilerInterface`].
///
/// I.e., This function will load the current interface and calls a function with it.
/// Do not nest these, as that will ICE.
pub(crate) fn with<R>(f: impl FnOnce(&'static dyn CompilerInterface) -> R) -> R {
    assert!(TLV.is_set());
    TLV.with(|tlv| {
        let ptr = tlv.get();
        assert!(!ptr.is_null());
        f(unsafe { *(ptr as *const &dyn CompilerInterface) })
    })
}
