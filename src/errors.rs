use std::fmt::Display;

use crate::{generator::{GeneratorError, MAX_NUM_INSTRUCTIONS}, lexer::Span, parser::ParseError, sources::SourceManager};

#[derive(Debug, Clone)]
pub(crate) struct Diagnostic {
    kind: DiagnosticKind,
    label: String,
    label_span: Option<Span>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagnosticKind {
    Error,
}

impl Display for DiagnosticKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Error => "error",
        })
    }
}

impl Diagnostic {
    pub fn new(kind: DiagnosticKind, label: impl Into<String>) -> Self {
        Self {
            kind,
            label: label.into(),
            label_span: None,
        }
    }

    pub fn new_with_span(kind: DiagnosticKind, label: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            label: label.into(),
            label_span: Some(span),
        }
    }

    pub fn error(label: impl Into<String>) -> Self {
        Self::new(DiagnosticKind::Error, label)
    }

    pub fn error_with_span(label: impl Into<String>, span: Span) -> Self {
        Self::new_with_span(DiagnosticKind::Error, label, span)
    }

    pub fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    pub fn label(&self) -> &String {
        &self.label
    }

    pub fn label_span(&self) -> Option<Span> {
        self.label_span
    }
}

pub(crate) struct TerminalEmitter {}

impl TerminalEmitter {
    pub(crate) fn emit(diagnostic: Diagnostic, source_manager: &SourceManager) {
        eprintln!("{}: {}", diagnostic.kind(), diagnostic.label());

        if let Some(label_span) = diagnostic.label_span() {
            let (line, line_number, column) = source_manager.get_span_line(label_span).unwrap();

            let line_number_width = format!("{}", line_number).len();
            let line_number_padding: String =
                std::iter::repeat(' ').take(line_number_width).collect();

            eprintln!(
                " {} --> {}:{}:{}",
                line_number_padding,
                source_manager.file_name(),
                line_number,
                column
            );

            // Fixes tab rendering to be what we define
            let line_fixed = line.replace('\t', "    ");

            eprintln!(" {} | {}", line_number, line_fixed);

            let mut pointer = line_number_padding.clone();

            for _ in 0..(column + 4) {
                pointer.push(' ');
            }

            for _ in 0..label_span.len {
                pointer.push('^');
            }

            eprintln!("{}", pointer);
        }
    }
}

pub(crate) fn parse_error_into_diagnostic(
    error: ParseError,
    source_manager: &SourceManager,
) -> Diagnostic {
    match error {
        ParseError::MissingToken(tt) => {
            Diagnostic::error(format!("Expected `{:?}`, found the end of file", tt))
        }
        ParseError::UnexpectedToken(tt, t) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!("Expected `{:?}`, found `{}`", tt, text);

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::InvalidInstruction(t) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!("`{}` is not a valid instruction", text);

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::ExpectedInstructionBeforeLabel(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            let label = format!(
                "Expected instruction after label, found second label `{}`",
                text
            );
            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::DuplicateLabel(t) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!("Duplicate label `{}`", text);

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::ExpectedNoOperands(t) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!("Instruction takes no operands, found `{}`", text);

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::ExpectedInstruction(t) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!("Expected an instruction, found `{}`", text);

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::ExpectedOperand(t, expected) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!(
                "Expected instruction operand (one of {}), found `{}`",
                expected, text
            );

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::ExpectedOperandFoundEOF(t) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!(
                "Expected instruction operand for `{}`, found end of file",
                text
            );

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::ExpectedRegister(t) => {
            let text = source_manager.get_span(t.span).unwrap();
            let label = format!(
                "Expected register for instruction operand, found `{}`",
                text
            );

            Diagnostic::error_with_span(label, t.span)
        }
        ParseError::IntegerOutOfRange(t) => {
            let label = format!("Value is out of range for an 8-bit signed integer value");
            Diagnostic::error_with_span(label, t.span)
        }
    }
}

pub(crate) fn generator_error_into_diagnostic(
    error: GeneratorError,
    source_manager: &SourceManager,
) -> Diagnostic {
    match error {
        GeneratorError::DanglingLabelError(span) => {
            let text = source_manager.get_span(span).unwrap();
            let label = format!("Dangling label `{text}`");

            Diagnostic::error_with_span(label, span)
        }
        GeneratorError::SourceOrSinkRangeError(span) => {
            let text = source_manager.get_span(span).unwrap();
            let label = format!("Source or sink must be in the range of 0-15, found `{text}`");

            Diagnostic::error_with_span(label, span)
        }
        GeneratorError::JumpDestinationRangeError(span) => {
            let text = source_manager.get_span(span).unwrap();
            let max_destination = MAX_NUM_INSTRUCTIONS - 1;
            let label = format!("Jump destination must be in the range of 0-{max_destination}, found `{text}`");

            Diagnostic::error_with_span(label, span)
        }
        GeneratorError::MaximumInstructionsError => {
            let label = format!("Maximum number of instructions reached ({MAX_NUM_INSTRUCTIONS})");

            Diagnostic::error(label)
        }
        GeneratorError::UndefinedLabelError(span) => {
            let text = source_manager.get_span(span).unwrap();
            let label = format!("Label `{text}` is undefined");

            Diagnostic::error_with_span(label, span)
        }
    }
}
