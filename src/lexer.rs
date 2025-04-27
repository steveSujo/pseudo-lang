use std::{collections::TryReserveError, ops::Range};

use crate::{
    error_handler::Errors,
    tokens::{LiteralType, Token, TokenType},
};

pub struct Lexer {
    source: String,
    //TODO: Tokens
    pub token_list: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Lexer {
            source,
            token_list: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Errors> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.token_list.push(Token::new(
            TokenType::EOF,
            String::default(),
            None,
            self.line,
        ));

        return Ok(self.token_list.clone());
    }

    fn scan_token(&mut self) -> Result<(), Errors> {
        let character = self.advance();
        match character {
            '(' => self.add_token(TokenType::LeftPara),
            ')' => self.add_token(TokenType::RightPara),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::COMMA),
            '.' => self.add_token(TokenType::DOT),
            '-' => self.add_token(TokenType::MINUS),
            '+' => self.add_token(TokenType::PLUS),
            ';' => self.add_token(TokenType::SEMICOLON),
            '*' => self.add_token(TokenType::STAR),
            '!' => {
                let token_type = if self.match_next('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::BANG
                };
                self.add_token(token_type);
            }
            '=' => {
                let token_type = if self.match_next('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::EQUAL
                };
                self.add_token(token_type);
            }
            '<' => {
                let token_type = if self.match_next('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::LESS
                };
                self.add_token(token_type);
            }
            '>' => {
                let token_type = if self.match_next('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::GREATER
                };
                self.add_token(token_type);
            }
            '/' => {
                if self.match_next('/') {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH)
                }
            }
            '\n' => self.line += 1,
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '"' => self.string()?,
            char if char.is_digit(10) => self.number()?,
            char if char.is_alphabetic() || char == '_' => self.identifier(),
            char => {
                //error unexpeted char
                // self.error_set
                //     .error_where(self.line, char.to_string(), "Unexpeted char".to_owned())
                return Err(Errors::UnexpectedChar);
            }
        }
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let character = self.source.chars().nth(self.current).unwrap();

        self.current += 1;

        character
    }

    fn match_next(&mut self, pattern: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self
            .source
            .chars()
            .nth(self.current)
            .is_some_and(|char| char != pattern)
        {
            return false;
        } else {
            self.current += 1;
            return true;
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        return self.add_token_with_literal(token_type, None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<LiteralType>) {
        let sub_string = &self.source[self.start..self.current];
        self.token_list.push(Token::new(
            token_type,
            sub_string.to_string(),
            literal,
            self.line,
        ))
    }

    fn peek(&self) -> Option<char> {
        // if self.is_at_end() {
        //     '\0'
        // } else {
        //     self.source.chars().nth(self.current).is_some()
        // }

        self.source.chars().nth(self.current)
    }

    fn string(&mut self) -> Result<(), Errors> {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(Errors::UntermitedString);
        }
        self.advance();
        let string_value = &self.source[self.start + 1..self.current - 1];
        self.add_token_with_literal(
            TokenType::STRING,
            Some(LiteralType::String(string_value.to_string())),
        );
        Ok(())
    }

    fn number(&mut self) -> Result<(), Errors> {
        let digits: Range<Option<char>> = Some('0')..Some('9');

        while digits.contains(&self.peek()) {
            self.advance();
        }
        if self.peek() == Some('.') && digits.contains(&self.peek_next()) {
            self.advance();
            while digits.contains(&self.peek()) {
                self.advance();
            }
        }
        self.add_token_with_literal(
            TokenType::NUMBER,
            Some(LiteralType::Number(
                self.source[self.start..self.current].parse::<f32>()?,
            )),
        );
        Ok(())
    }

    fn peek_next(&self) -> Option<char> {
        // if self.current + 1 >= self.source.len() {
        //     return None;
        // }
        self.source.chars().nth(self.current + 1)
    }

    fn identifier(&mut self) {
        while self.peek().is_some_and(|char| char.is_alphanumeric()) {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let token_type = match text {
            "and" => TokenType::AND,
            "class" => TokenType::CLASS,
            "else" => TokenType::ELSE,
            "false" => TokenType::FALSE,
            "fun" => TokenType::FUN,
            "for" => TokenType::FOR,
            "if" => TokenType::IF,
            "nil" => TokenType::NIL,
            "or" => TokenType::OR,
            "print" => TokenType::PRINT,
            "return" => TokenType::RETURN,
            "this" => TokenType::THIS,
            "true" => TokenType::TRUE,
            "let" => TokenType::LET,
            "while" => TokenType::WHILE,
            _ => TokenType::IDENTIFIER,
        };
        self.add_token(token_type);
    }
}
