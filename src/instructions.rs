#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandType {
    Register,
    Integer,
    Label,
}

impl OperandType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Integer => "integer",
            Self::Label => "label",
        }
    }
}

pub mod rules {
    use super::OperandType;

    pub static NOP_RULES: &[&[OperandType]] = &[];
    pub static ADD_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Register]];
    pub static LDI_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Integer]];
    pub static SUB_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Register]];
    pub static AND_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Register]];
    pub static OR_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Register]];
    pub static INV_RULES: &[&[OperandType]] = &[&[OperandType::Register]];
    pub static XOR_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Register]];
    pub static SR_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Register]];
    pub static SL_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Register]];
    pub static IN_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Integer]];
    pub static OUT_RULES: &[&[OperandType]] = &[&[OperandType::Register], &[OperandType::Integer]];
    pub static JZ_RULES: &[&[OperandType]] = &[
        &[OperandType::Register],
        &[OperandType::Integer, OperandType::Label],
    ];
    pub static JLT_RULES: &[&[OperandType]] = &[
        &[OperandType::Register],
        &[OperandType::Integer, OperandType::Label],
    ];
    pub static J_RULES: &[&[OperandType]] = &[&[OperandType::Integer, OperandType::Label]];
}
