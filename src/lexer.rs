#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub tt: TokenType,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub index: u32,
    pub len: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Identifier,
    Label,
    Comma,
    Integer,
    Newline,
    Comment,

    InvalidTokenError,
    InvalidIntegerError,
}

pub struct Lexer<'a> {
    input: &'a str,
    current_index: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            input: source,
            current_index: 0,
        }
    }

    pub fn lex(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        while let Some(c) = self.peek_char() {
            let token = match c {
                '\r' | '\t' | ' ' => {
                    // Skip
                    self.current_index += 1;
                    continue;
                }
                '\n' => self.consume_current_single_char_token(TokenType::Newline),
                ',' => self.consume_current_single_char_token(TokenType::Comma),
                ';' => self.lex_comment(),
                _ => {
                    if c.is_digit(10) {
                        self.lex_integer()
                    } else if c.is_alphabetic() {
                        self.lex_identifier()
                    } else {
                        self.consume_current_single_char_token(TokenType::InvalidTokenError)
                    }
                }
            };

            tokens.push(token);
        }

        tokens
    }

    fn lex_comment(&mut self) -> Token {
        let start_index = self.current_index;
        let mut len = 1;

        self.current_index += 1;

        while let Some(c) = self.peek_char() {
            if c == '\n' {
                break;
            }

            len += 1;
            self.current_index += 1;
        }

        Token {
            tt: TokenType::Comment,
            span: Span {
                index: start_index as u32,
                len,
            },
        }
    }

    fn lex_integer(&mut self) -> Token {
        let start_index = self.current_index;
        let mut len = 1;
        let mut is_valid_int = true;

        self.current_index += 1;

        while let Some(c) = self.peek_char() {
            if c.is_digit(10) {
                len += 1;
                self.current_index += 1;
            } else if c.is_alphabetic() {
                is_valid_int = false;
                len += 1;
                self.current_index += 1;
            } else {
                break;
            }
        }

        if is_valid_int {
            Token {
                tt: TokenType::Integer,
                span: Span {
                    index: start_index as u32,
                    len,
                },
            }
        } else {
            Token {
                tt: TokenType::InvalidIntegerError,
                span: Span {
                    index: start_index as u32,
                    len,
                },
            }
        }
    }

    fn lex_identifier(&mut self) -> Token {
        let start_index = self.current_index;
        let mut len = 1;
        let mut is_label = false;

        self.current_index += 1;

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                len += 1;
                self.current_index += 1;
            } else if c == ':' {
                len += 1;
                self.current_index += 1;
                is_label = true;
                break;
            } else {
                break;
            }
        }

        if is_label {
            Token {
                tt: TokenType::Label,
                span: Span {
                    index: start_index as u32,
                    len,
                },
            }
        } else {
            Token {
                tt: TokenType::Identifier,
                span: Span {
                    index: start_index as u32,
                    len,
                },
            }
        }
    }

    fn consume_current_single_char_token(&mut self, tt: TokenType) -> Token {
        let token = Token {
            tt,
            span: Span {
                index: self.current_index as u32,
                len: 1,
            },
        };

        self.current_index += 1;

        token
    }

    fn peek_char(&mut self) -> Option<char> {
        self.input.chars().skip(self.current_index).next()
    }

    #[allow(dead_code)]
    fn next_char(&mut self) -> Option<char> {
        let c = self.input.chars().skip(self.current_index).next();

        self.current_index += 1;

        c
    }
}
