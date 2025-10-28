use std::cell::Cell;
use std::collections::HashMap;
use crate::file::SourceFile;
use crate::span::Span;

struct SpanData {
    file: SourceFile,
    start: u32,
    length: u32,
}

struct CompilerContext {
    spans: Vec<SpanData>,
}

impl CompilerContext {
    pub fn add_span(
        self: &mut CompilerContext,
        file: SourceFile,
        start: u32,
        length: u32,
    ) -> Span {
        let handle = self.spans.len();
        self.spans.push(SpanData{
            file, start, length
        });

        Span::internal_new(handle)
    }
}

pub(crate) trait CompilerInterface {
    fn add_span(
        &self,
        file: SourceFile,
        start: u32,
        length: u32
    ) -> Span;

    fn span_line(&self, s: Span) -> u32;
    fn span_column(&self, s: Span) -> u32;
    fn span_file(&self, s: Span) -> SourceFile;
}

// A thread local variable that stores a pointer to [`CompilerInterface`].
scoped_tls::scoped_thread_local!(static TLV: Cell<*const ()>);

/// Execute the given function with access the [`CompilerInterface`].
///
/// I.e., This function will load the current interface and calls a function with it.
/// Do not nest these, as that will ICE.
pub(crate) fn with<R>(f: impl FnOnce(&dyn CompilerInterface) -> R) -> R {
    assert!(TLV.is_set());
    TLV.with(|tlv| {
        let ptr = tlv.get();
        assert!(!ptr.is_null());
        f(unsafe { *(ptr as *const &dyn CompilerInterface) })
    })
}
