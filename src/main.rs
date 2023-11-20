use lexer::Lexer;
use parser::{ParseError, Parser};
use sources::SourceManager;

use crate::lexer::TokenType;

mod instructions;
mod lexer;
mod parser;
mod sources;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Register {
    pub fn encode(self) -> u8 {
        match self {
            Self::R0 => 0,
            Self::R1 => 1,
            Self::R2 => 2,
            Self::R3 => 3,
            Self::R4 => 4,
            Self::R5 => 5,
            Self::R6 => 6,
            Self::R7 => 7,
            Self::R8 => 8,
            Self::R9 => 9,
            Self::R10 => 10,
            Self::R11 => 11,
            Self::R12 => 12,
            Self::R13 => 13,
            Self::R14 => 14,
            Self::R15 => 15,
        }
    }
}

impl TryFrom<&str> for Register {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "r0" => Self::R0,
            "r1" => Self::R1,
            "r2" => Self::R2,
            "r3" => Self::R3,
            "r4" => Self::R4,
            "r5" => Self::R5,
            "r6" => Self::R6,
            "r7" => Self::R7,
            "r8" => Self::R8,
            "r9" => Self::R9,
            "r10" => Self::R10,
            "r11" => Self::R11,
            "r12" => Self::R12,
            "r13" => Self::R13,
            "r14" => Self::R14,
            "r15" => Self::R15,
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Opcode {
    Nop,
    Add,
    Ldi,
    Sub,
    And,
    Or,
    Inv,
    Xor,
    Sr,
    Sl,
    In,
    Out,
    Jz,
    Jlt,
    J,
}

impl Opcode {
    pub fn encode(self) -> u8 {
        match self {
            Self::Nop => 0,
            Self::Add => 1,
            Self::Ldi => 2,
            Self::Sub => 3,
            Self::And => 5,
            Self::Or => 6,
            Self::Inv => 7,
            Self::Xor => 8,
            Self::Sr => 9,
            Self::Sl => 10,
            Self::In => 11,
            Self::Out => 12,
            Self::Jz => 13,
            Self::Jlt => 14,
            Self::J => 15,
        }
    }
}

impl TryFrom<&str> for Opcode {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "nop" => Self::Nop,
            "add" => Self::Add,
            "ldi" => Self::Ldi,
            "sub" => Self::Sub,
            "and" => Self::And,
            "or" => Self::Or,
            "inv" => Self::Inv,
            "xor" => Self::Xor,
            "sr" => Self::Sr,
            "sl" => Self::Sl,
            "in" => Self::In,
            "out" => Self::Out,
            "jz" => Self::Jz,
            "jlt" => Self::Jlt,
            "j" => Self::J,
            _ => {
                return Err(());
            }
        })
    }
}

pub type LabelId = usize;

#[derive(Debug, Clone, Copy)]
enum Operand {
    Register(Register),
    Integer(i16),
    Label(LabelId),
}

#[derive(Debug, Clone, Copy)]
enum Instruction {
    NoOperand(Opcode),
    SingleOperand(Opcode, Operand),
    DoubleOperand(Opcode, Operand, Operand),
}

pub struct LabelManager {
    map: Vec<(String, Option<u8>)>,
}

impl LabelManager {
    pub fn new() -> Self {
        Self { map: Vec::new() }
    }

    pub fn get_id_of(&self, label: &str) -> Option<LabelId> {
        self.map.iter().position(|l| l.0 == label)
    }

    pub fn insert_unique(&mut self, label: &str) -> Result<LabelId, ()> {
        let exists = self.map.iter().any(|l| l.0 == label);

        if exists {
            Err(())
        } else {
            self.map.push((String::from(label), None));
            Ok(self.map.len() - 1)
        }
    }

    pub fn get_or_insert_reference(&mut self, label: &str) -> LabelId {
        let exists = self.map.iter().any(|l| l.0 == label);

        if exists {
            self.get_id_of(label).unwrap()
        } else {
            self.map.push((String::from(label), None));
            self.map.len() - 1
        }
    }

    pub fn get_value_of(&self, id: LabelId) -> Option<u8> {
        self.map.get(id).map(|l| l.1).flatten()
    }
}

fn main() {
    let source = "nop ; Do something cool\nnop\ninv r2\ninfinite: j infinite";

    let source_manager = SourceManager::new(source);
    let mut lexer = Lexer::new(source);

    let tokens = lexer.lex();
    let mut valid_tokens = Vec::with_capacity(tokens.capacity());

    for token in tokens {
        if token.tt == TokenType::InvalidTokenError {
            let text = source_manager.get_span(token.span).unwrap();
            eprintln!("Invalid token found `{}`", text);
        } else if token.tt == TokenType::InvalidIntegerError {
            let text = source_manager.get_span(token.span).unwrap();
            eprintln!("Invalid integer value `{}`", text);
        } else if token.tt != TokenType::Comment {
            valid_tokens.push(token);
        }
    }

    let parser = Parser::new(&valid_tokens, &source_manager);

    match parser.parse() {
        Ok((items, label_manager)) => {
            for item in items {
                println!("{:?}", item);
            }
        }
        Err(e) => {
            display_parse_error(&source_manager, e);
        }
    }
}

fn display_parse_error(source_manager: &SourceManager, error: ParseError) {
    match error {
        ParseError::MissingToken(tt) => {
            eprintln!("Expected `{:?}`, found the end of file", tt);
        }
        ParseError::UnexpectedToken(tt, t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!("Expected `{:?}`, found `{}`", tt, text);
        }
        ParseError::InvalidInstruction(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!("`{}` is not a valid instruction", text);
        }
        ParseError::ExpectedInstructionOrLabel(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!("Expected instruction or label, found `{}`", text);
        }
        ParseError::ExpectedInstructionBeforeLabel(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!(
                "Expected instruction after label, found second label `{}`",
                text
            );
        }
        ParseError::DuplicateLabel(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!("Duplicate label `{}`", text);
        }
        ParseError::ExpectedInstructionOrNewline(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!(
                "Expected instruction or newline after label, found `{}`",
                text
            );
        }
        ParseError::ExpectedNoOperands(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!("Instruction takes no operands, found `{}`", text);
        }
        ParseError::ExpectedInstruction(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!("Expected an instruction, found `{}`", text);
        }
        ParseError::ExpectedOperand(t, expected) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!(
                "Expected instruction operand (one of {}), found `{}`",
                expected, text
            );
        }
        ParseError::ExpectedOperandFoundEOF(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!(
                "Expected instruction operand for `{}`, found end of file",
                text
            );
        }
        ParseError::ExpectedRegister(t) => {
            let text = source_manager.get_span(t.span).unwrap();

            eprintln!(
                "Expected register for instruction operand, found `{}`",
                text
            );
        }
        ParseError::IntegerOutOfRange(t) => {
            eprintln!("Value is out of range for 16-bit signed integer values")
        }
    }
}
