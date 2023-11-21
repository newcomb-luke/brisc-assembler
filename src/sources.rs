use std::ops::Range;

use crate::lexer::Span;

pub struct SourceManager<'a> {
    source: &'a str,
    lines: Vec<Range<usize>>,
    file_name: String,
}

impl<'a> SourceManager<'a> {
    pub fn new(source: &'a str, file_name: String) -> Self {
        let mut lines = Vec::new();
        let mut last_newline = 0;
        let mut current_index = 0;
        let mut saw_carriage_return = false;

        for c in source.chars() {
            match c {
                '\r' => {
                    saw_carriage_return = true;
                }
                '\n' => {
                    if saw_carriage_return {
                        lines.push(last_newline..(current_index - 1));
                        saw_carriage_return = false;
                    } else {
                        lines.push(last_newline..current_index);
                    }

                    last_newline = current_index + 1;
                }
                _ => {}
            }

            current_index += 1;
        }

        if last_newline != current_index {
            lines.push(last_newline..current_index);
        }

        Self {
            source,
            lines,
            file_name,
        }
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

    /// Gets the line containing the span from the source, also returning the line number and column index
    /// of the span in the line
    ///
    /// The span must not cross multiple lines
    pub fn get_span_line(&'a self, span: Span) -> Result<(&'a str, u32, u32), ()> {
        let (line_range, line_number) = self
            .find_line_containing_char(span.index as usize)
            .ok_or(())?;
        let line = &self.source[line_range.clone()];
        let span_line_index = span.index as usize - line_range.start;

        let mut col = 0;

        for (i, c) in line.chars().enumerate() {
            if i == span_line_index {
                break;
            }

            if c != '\t' {
                col += 1;
            } else {
                col += 4;
            }
        }

        Ok((line, line_number + 1, col))
    }

    pub fn file_name(&self) -> &String {
        &self.file_name
    }

    fn find_line_containing_char(&'a self, index: usize) -> Option<(Range<usize>, u32)> {
        for (line_number, line) in self.lines.iter().enumerate() {
            if line.contains(&index) {
                return Some((line.clone(), line_number as u32));
            }
        }

        None
    }
}
