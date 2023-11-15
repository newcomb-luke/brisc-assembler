use crate::lexer::Span;

pub struct SourceManager<'a> {
    source: &'a str,
}

impl<'a> SourceManager<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn get_span(&'a self, span: Span) -> Result<&'a str, ()> {
        let index = span.index as usize;
        let len = span.len as usize;

        if (index + len) <= self.source.len() {
            Ok(&self.source[index..(index + len)])
        } else {
            Err(())
        }
    }
}
