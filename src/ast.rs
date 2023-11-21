use crate::lexer::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Register {
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
pub(crate) enum Opcode {
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

pub(crate) type LabelId = usize;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Operand {
    Register { value: Register, span: Span },
    Integer { value: i8, span: Span },
    Label { value: LabelId, span: Span },
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Instruction {
    NoOperand(Opcode),
    SingleOperand(Opcode, Operand),
    DoubleOperand(Opcode, Operand, Operand),
}

impl Instruction {
    #[allow(dead_code)]
    pub fn opcode(&self) -> Opcode {
        match self {
            Self::NoOperand(op) => *op,
            Self::SingleOperand(op, _) => *op,
            Self::DoubleOperand(op, _, _) => *op,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Item {
    Label(LabelId),
    Instruction(Instruction),
}
