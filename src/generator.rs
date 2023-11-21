use crate::{
    ast::{Instruction, Item, Opcode, Operand, Register},
    lexer::Span, parser::LabelManager,
};

pub(crate) const INSTRUCTION_MEMORY_SIZE_BYTES: i8 = 64;
const INSTRUCTION_SIZE_BYTES: i8 = 2;
pub(crate) const MAX_NUM_INSTRUCTIONS: i8 = INSTRUCTION_MEMORY_SIZE_BYTES / INSTRUCTION_SIZE_BYTES;

#[derive(Debug, Clone)]
pub(crate) enum GeneratorError {
    SourceOrSinkRangeError(Span),
    DanglingLabelError(Span),
    MaximumInstructionsError,
    UndefinedLabelError(Span),
    JumpDestinationRangeError(Span),
}

pub(crate) struct Generator {
    items: Vec<Item>,
    label_manager: LabelManager,
}

impl Generator {
    pub fn new(items: Vec<Item>, label_manager: LabelManager) -> Self {
        Self {
            items,
            label_manager,
        }
    }

    pub fn generate(&mut self) -> Result<Vec<u8>, GeneratorError> {
        let mut output = Vec::new();

        let mut instr_counter = 0;
        let mut ended_on_label = None;

        for item in self.items.iter() {
            match *item {
                Item::Label(label_id) => {
                    ended_on_label = Some(label_id);
                    self.label_manager
                        .set_value_of(label_id, instr_counter as i8)
                        .unwrap();
                }
                Item::Instruction(_) => {
                    ended_on_label = None;
                    instr_counter += 1;

                    if instr_counter > MAX_NUM_INSTRUCTIONS {
                        return Err(GeneratorError::MaximumInstructionsError);
                    }
                }
            }
        }

        // If we ended on a label, this will be Some()
        if let Some(label_id) = ended_on_label {
            let span = self.label_manager.get_span_of(label_id).unwrap();
            return Err(GeneratorError::DanglingLabelError(span));
        }

        for item in self.items.iter() {
            match item {
                Item::Label(_) => {}
                Item::Instruction(instruction) => {
                    match instruction {
                        Instruction::NoOperand(opcode) => {
                            if *opcode != Opcode::Nop {
                                panic!("Internal Assembler Error");
                            }

                            Self::generate_no_operand(&mut output, *opcode)
                        }
                        Instruction::SingleOperand(opcode, operand) => {
                            if *opcode == Opcode::Inv {
                                if let Operand::Register {
                                    value: register,
                                    span: _,
                                } = operand
                                {
                                    Self::generate_single_register(&mut output, *opcode, *register);
                                } else {
                                    panic!("Internal Assembler Error");
                                }
                            } else if *opcode == Opcode::J {
                                match *operand {
                                    Operand::Integer { value, span: _ } => {
                                        // R0 here is arbitrary, the value is never looked at
                                        Self::generate_immediate(
                                            &mut output,
                                            *opcode,
                                            Register::R0,
                                            value,
                                        );
                                    }
                                    Operand::Label {
                                        value: label_id,
                                        span,
                                    } => {
                                        if let Some(value) =
                                            self.label_manager.get_value_of(label_id)
                                        {
                                            // R0 here is arbitrary, the value is never looked at
                                            Self::generate_immediate(
                                                &mut output,
                                                *opcode,
                                                Register::R0,
                                                value,
                                            );
                                        } else {
                                            return Err(GeneratorError::UndefinedLabelError(span));
                                        }
                                    }
                                    _ => {
                                        panic!("Internal Assembler Error");
                                    }
                                }
                            } else {
                                panic!("Internal Assembler Error");
                            }
                        }
                        Instruction::DoubleOperand(opcode, operand1, operand2) => match opcode {
                            Opcode::Add
                            | Opcode::Sub
                            | Opcode::And
                            | Opcode::Or
                            | Opcode::Xor
                            | Opcode::Sr
                            | Opcode::Sl => {
                                if let Operand::Register {
                                    value: register1,
                                    span: _,
                                } = *operand1
                                {
                                    if let Operand::Register {
                                        value: register2,
                                        span: _,
                                    } = *operand2
                                    {
                                        Self::generate_double_register(
                                            &mut output,
                                            *opcode,
                                            register1,
                                            register2,
                                        );
                                    } else {
                                        panic!("Internal Assembler Error");
                                    }
                                } else {
                                    panic!("Internal Assembler Error");
                                }
                            }
                            Opcode::Jz | Opcode::Jlt => {
                                if let Operand::Register {
                                    value: register,
                                    span: _,
                                } = *operand1
                                {
                                    match *operand2 {
                                        Operand::Label {
                                            value: label_id,
                                            span,
                                        } => {
                                            if let Some(value) =
                                                self.label_manager.get_value_of(label_id)
                                            {
                                                Self::generate_immediate(
                                                    &mut output,
                                                    *opcode,
                                                    register,
                                                    value,
                                                );
                                            } else {
                                                return Err(GeneratorError::UndefinedLabelError(
                                                    span,
                                                ));
                                            }
                                        }
                                        Operand::Integer { value, span } => {
                                            if value < MAX_NUM_INSTRUCTIONS {
                                                Self::generate_immediate(
                                                    &mut output,
                                                    *opcode,
                                                    register,
                                                    value,
                                                );
                                            } else {
                                                return Err(
                                                    GeneratorError::JumpDestinationRangeError(span),
                                                );
                                            }
                                        }
                                        _ => {
                                            panic!("Internal Assembler Error");
                                        }
                                    }
                                } else {
                                    panic!("Internal Assembler Error");
                                }
                            }
                            Opcode::Ldi => {
                                if let Operand::Register {
                                    value: register,
                                    span: _,
                                } = *operand1
                                {
                                    if let Operand::Integer { value, span: _ } = *operand2 {
                                        Self::generate_immediate(
                                            &mut output,
                                            *opcode,
                                            register,
                                            value,
                                        );
                                    } else {
                                        panic!("Internal Assembler Error");
                                    }
                                } else {
                                    panic!("Internal Assembler Error");
                                }
                            }
                            Opcode::In | Opcode::Out => {
                                if let Operand::Register {
                                    value: register,
                                    span: _,
                                } = *operand1
                                {
                                    if let Operand::Integer { value, span } = *operand2 {
                                        Self::generate_io(
                                            &mut output,
                                            *opcode,
                                            register,
                                            value as u8,
                                        )
                                        .map_err(|_| {
                                            GeneratorError::SourceOrSinkRangeError(span)
                                        })?;
                                    } else {
                                        panic!("Internal Assembler Error");
                                    }
                                } else {
                                    panic!("Internal Assembler Error");
                                }
                            }
                            _ => {
                                panic!("Internal Assembler Error");
                            }
                        },
                    }
                }
            }
        }

        Ok(output)
    }

    fn generate_immediate(buffer: &mut Vec<u8>, opcode: Opcode, register: Register, value: i8) {
        let first_byte = (opcode.encode() << 4) | (register.encode());

        buffer.push(first_byte);
        buffer.push(value as u8);
    }

    fn generate_single_register(buffer: &mut Vec<u8>, opcode: Opcode, register: Register) {
        // The 0 could be anything, but this is just the same as an immediate instruction, but the data field is ignored
        Self::generate_immediate(buffer, opcode, register, 0);
    }

    fn generate_double_register(
        buffer: &mut Vec<u8>,
        opcode: Opcode,
        register1: Register,
        register2: Register,
    ) {
        let first_byte = (opcode.encode() << 4) | (register1.encode());
        let second_byte = register2.encode() << 4;

        buffer.push(first_byte);
        buffer.push(second_byte);
    }

    fn generate_io(
        buffer: &mut Vec<u8>,
        opcode: Opcode,
        register: Register,
        source_or_sink: u8,
    ) -> Result<(), ()> {
        if source_or_sink > 0b1111 {
            return Err(());
        }

        Self::generate_immediate(buffer, opcode, register, (source_or_sink as i8) << 4);

        Ok(())
    }

    fn generate_no_operand(buffer: &mut Vec<u8>, opcode: Opcode) {
        // The 0 could be anything, but this is just the same as an immediate instruction, but everything but the opcode is ignored
        Self::generate_immediate(buffer, opcode, Register::R0, 0);
    }
}
