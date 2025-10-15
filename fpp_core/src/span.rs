use crate::file::{BytePos, SourceFile};

pub struct SpanData<'a> {
    pub file: &'a SourceFile,
    pub start: BytePos,
    pub length: u32,
}

impl<'a> Into<Span> for SpanData<'a> {
    fn into(self) -> Span {
        todo!()
    }
}

pub struct Span {
    hi: u32,
    lo: u32,
}

// TODO(tumbar) Implement span
impl Span {
    pub fn merge(left: Span, right: Span) -> Span {
        Span{
            hi: 0,
            lo: 0,
        }
    }

    pub fn clone(&self) -> Span {
        Span{
            hi: self.hi,
            lo: self.lo,
        }
    }
}