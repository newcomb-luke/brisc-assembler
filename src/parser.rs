use std::{collections::HashMap, iter::Peekable, slice::Iter};

use crate::{
    instructions::{rules::*, OperandType},
    lexer::{Token, TokenType},
    sources::SourceManager,
    Instruction, LabelId, LabelManager, Opcode, Operand, Register,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Item {
    Label(LabelId),
    Instruction(Instruction),
}

pub enum ParseError {
    UnexpectedToken(TokenType, Token),
    MissingToken(TokenType),
    ExpectedInstructionOrLabel(Token),
    InvalidInstruction(Token),
    ExpectedInstructionBeforeLabel(Token),
    DuplicateLabel(Token),
    ExpectedInstructionOrNewline(Token),
    ExpectedInstruction(Token),
    ExpectedNoOperands(Token),
    ExpectedOperandFoundEOF(Token),
    ExpectedOperand(Token, String),
    ExpectedRegister(Token),
    IntegerOutOfRange(Token),
}

pub(crate) struct Parser<'a, 'b, 'c> {
    tokens: &'a Vec<Token>,
    tokens_iter: Peekable<Iter<'a, Token>>,
    source_manager: &'b SourceManager<'c>,
    parse_rules: HashMap<Opcode, &'static [&'static [OperandType]]>,
    label_manager: LabelManager,
    just_saw_label: bool,
}

impl<'a, 'b, 'c> Parser<'a, 'b, 'c> {
    pub fn new(tokens: &'a Vec<Token>, source_manager: &'b SourceManager<'c>) -> Self {
        let mut parse_rules = HashMap::new();

        parse_rules.insert(Opcode::Nop, NOP_RULES);
        parse_rules.insert(Opcode::Add, ADD_RULES);
        parse_rules.insert(Opcode::Ldi, LDI_RULES);
        parse_rules.insert(Opcode::Sub, SUB_RULES);
        parse_rules.insert(Opcode::And, AND_RULES);
        parse_rules.insert(Opcode::Or, OR_RULES);
        parse_rules.insert(Opcode::Inv, INV_RULES);
        parse_rules.insert(Opcode::Xor, XOR_RULES);
        parse_rules.insert(Opcode::Sr, SR_RULES);
        parse_rules.insert(Opcode::Sl, SL_RULES);
        parse_rules.insert(Opcode::In, IN_RULES);
        parse_rules.insert(Opcode::Out, OUT_RULES);
        parse_rules.insert(Opcode::Jz, JZ_RULES);
        parse_rules.insert(Opcode::Jlt, JLT_RULES);
        parse_rules.insert(Opcode::J, J_RULES);

        Self {
            tokens,
            tokens_iter: tokens.iter().peekable(),
            source_manager,
            parse_rules,
            label_manager: LabelManager::new(),
            just_saw_label: false,
        }
    }

    pub fn parse(mut self) -> Result<(Vec<Item>, LabelManager), ParseError> {
        let mut items = Vec::new();

        while self.tokens_iter.peek().is_some() {
            items.extend(self.parse_line()?);
        }

        Ok((items, self.label_manager))
    }

    fn parse_line(&mut self) -> Result<Vec<Item>, ParseError> {
        let mut items = Vec::new();

        if let Some(&&next_token) = self.tokens_iter.peek() {
            let mut should_parse_instruction = true;

            if next_token.tt == TokenType::Label {
                if self.just_saw_label {
                    return Err(ParseError::ExpectedInstructionBeforeLabel(next_token));
                }

                self.just_saw_label = true;

                let label_text_with_colon = self.source_manager.get_span(next_token.span).unwrap();
                let label_text = &label_text_with_colon[..label_text_with_colon.len() - 1];

                if let Ok(label_id) = self.label_manager.insert_unique(label_text) {
                    items.push(Item::Label(label_id));
                } else {
                    return Err(ParseError::DuplicateLabel(next_token));
                }

                // Consume the label token, we don't need it anymore
                self.tokens_iter.next().unwrap();
                should_parse_instruction =
                    self.tokens_iter.peek().is_some() && !self.is_peek_token(TokenType::Newline);
            }

            if should_parse_instruction {
                items.push(Item::Instruction(self.parse_instruction()?));
            }

            self.consume_or_eof(TokenType::Newline)?;

            Ok(items)
        } else {
            panic!("Internal Assembler Error: Should not have reached here");
        }
    }

    fn parse_instruction(&mut self) -> Result<Instruction, ParseError> {
        if let Some(&next_token) = self.tokens_iter.next() {
            if next_token.tt != TokenType::Identifier {
                return Err(ParseError::ExpectedInstruction(next_token));
            }

            let text = self.source_manager.get_span(next_token.span).unwrap();

            if let Ok(opcode) = Opcode::try_from(text) {
                let rules = *self.parse_rules.get(&opcode).unwrap();

                if rules.is_empty() {
                    if self.tokens_iter.peek().is_none() || self.is_peek_token(TokenType::Newline) {
                        // All good
                        Ok(Instruction::NoOperand(opcode))
                    } else {
                        Err(ParseError::ExpectedNoOperands(
                            *self.tokens_iter.next().unwrap(),
                        ))
                    }
                } else if rules.len() == 1 {
                    let operand = self.parse_operand(next_token, rules[0])?;

                    Ok(Instruction::SingleOperand(opcode, operand))
                } else if rules.len() == 2 {
                    let operand1 = self.parse_operand(next_token, rules[0])?;

                    self.expect_token(TokenType::Comma)?;

                    let operand2 = self.parse_operand(next_token, rules[1])?;

                    Ok(Instruction::DoubleOperand(opcode, operand1, operand2))
                } else {
                    panic!("Internal Assembler Error: Instructions with more than 2 operands are currently not supported");
                }
            } else {
                Err(ParseError::InvalidInstruction(next_token))
            }
        } else {
            panic!(
                "Internal Assembler Error: Attempted to parse instruction from empty token stream"
            );
        }
    }

    fn parse_operand(
        &mut self,
        instruction_token: Token,
        operand_rule: &[OperandType],
    ) -> Result<Operand, ParseError> {
        let expected_token_types: Vec<TokenType> = operand_rule
            .iter()
            .map(|ot| match ot {
                OperandType::Integer => TokenType::Integer,
                OperandType::Label | OperandType::Register => TokenType::Identifier,
            })
            .collect();

        if let Some(&next_token) = self.tokens_iter.next() {
            if expected_token_types.iter().any(|&t| t == next_token.tt) {
                // The token was the one that was expected
                let text = self.source_manager.get_span(next_token.span).unwrap();

                if next_token.tt == TokenType::Identifier {
                    if operand_rule.contains(&OperandType::Register) {
                        // See if it is is a register
                        if let Ok(register) = Register::try_from(text) {
                            return Ok(Operand::Register(register));
                        }
                    }

                    if operand_rule.contains(&OperandType::Label) {
                        // It's a label, we can't do much about checking it's validity until later
                        let label_id = self.label_manager.get_or_insert_reference(text);

                        Ok(Operand::Label(label_id))
                    } else if operand_rule.contains(&OperandType::Register) {
                        // It should have been a register, it just wasn't a valid one
                        Err(ParseError::ExpectedRegister(next_token))
                    } else {
                        panic!("Internal Assembler Error");
                    }
                } else if next_token.tt == TokenType::Integer {
                    if let Ok(parsed_value) = text.parse::<i16>() {
                        Ok(Operand::Integer(parsed_value))
                    } else {
                        Err(ParseError::IntegerOutOfRange(next_token))
                    }
                } else {
                    panic!("Internal Assembler Error");
                }
            } else {
                let expected = match operand_rule.len() {
                    1 => format!("{}", operand_rule[0].as_str()),
                    2 => format!(
                        "{} or {}",
                        operand_rule[0].as_str(),
                        operand_rule[0].as_str()
                    ),
                    3 => format!(
                        "{}, {} or {}",
                        operand_rule[0].as_str(),
                        operand_rule[1].as_str(),
                        operand_rule[2].as_str()
                    ),
                    _ => panic!("Internal Assembler Error"),
                };

                Err(ParseError::ExpectedOperand(next_token, expected))
            }
        } else {
            Err(ParseError::ExpectedOperandFoundEOF(instruction_token))
        }
    }

    fn is_peek_token(&mut self, tt: TokenType) -> bool {
        self.tokens_iter.peek().filter(|t| t.tt == tt).is_some()
    }

    fn consume_or_eof(&mut self, tt: TokenType) -> Result<(), ParseError> {
        if let Some(next_token) = self.tokens_iter.next() {
            if next_token.tt != tt {
                Err(ParseError::UnexpectedToken(tt, *next_token))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn expect_token(&mut self, tt: TokenType) -> Result<(), ParseError> {
        if let Some(next_token) = self.tokens_iter.next() {
            if next_token.tt != tt {
                Err(ParseError::UnexpectedToken(tt, *next_token))
            } else {
                Ok(())
            }
        } else {
            Err(ParseError::MissingToken(tt))
        }
    }
}
