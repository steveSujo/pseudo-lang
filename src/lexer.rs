use std::{
    borrow::Borrow, collections::TryReserveError, iter::Peekable, ops::Range, vec::IntoIter,
};

use crate::{
    error_handler::Errors,
    tokens::{LiteralType, Token, TokenType},
};

pub struct Lexer {
    // source: String,
    source: Peekable<IntoIter<char>>,
    //TODO: Tokens
    pub token_list: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Lexer {
            source: source.chars().collect::<Vec<char>>().into_iter().peekable(),
            token_list: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Errors> {
        while self.source.peek().is_some() {
            self.start = self.current;
            self.current += 1;
            self.scan_token()?;
            // println!("scT {} {} ", self.start, self.current,)
        }

        self.token_list
            .push(Token::new(TokenType::EOF, None, 0, 0, self.line));

        return Ok(self.token_list.clone());
    }

    fn scan_token(&mut self) -> Result<(), Errors> {
        self.source.next().map(|character| {
            Ok(match character {
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
                        while self.source.peek() != Some(&'\n') {
                            self.next();
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
                char if char.is_digit(10) => self.number(char)?,
                char if char.is_alphabetic() || char == '_' => self.identifier(char),
                _char => {
                    //error unexpeted char
                    // self.error_set
                    //     .error_where(self.line, char.to_string(), "Unexpeted char".to_owned())
                    return Err(Errors::UnexpectedChar);
                }
            })
        });
        Ok(())
    }

    // fn is_at_end(&self) -> bool {
    //     self.current >= self.source.len()
    // }

    fn next(&mut self) -> Option<char> {
        self.current += 1;
        self.source.next()
    }

    fn match_next(&mut self, pattern: char) -> bool {
        // if self.is_at_end() {
        //     return false;
        // }
        // if self
        //     .source
        //     .chars()
        //     .nth(self.current)
        //     .is_some_and(|char| char != pattern)
        // {
        //     return false;
        // } else {
        //     self.current += 1;
        //     return true;
        // }
        if self.source.peek() == Some(&pattern) {
            self.next();
            return true;
        } else {
            return false;
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        return self.add_token_with_literal(token_type, None);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<LiteralType>) {
        // let mut text: String = String::new();
        // eprintln!("token_type: {token_type:#?}");
        // for index in 0..=self.current {
        //     eprintln!("index {index}");
        //     text.push(self.source.nth(index).unwrap());
        // }
        self.token_list.push(Token::new(
            token_type,
            literal,
            self.start,
            self.current,
            self.line,
        ))
    }

    // fn peek(&self) -> Option<char> {
    //     // if self.is_at_end() {
    //     //     '\0'
    //     // } else {
    //     //     self.source.chars().nth(self.current).is_some()
    //     // }

    //     self.source.chars().nth(self.current)
    // }

    fn string(&mut self) -> Result<(), Errors> {
        let mut string_value: String = String::new();
        while self.source.peek() != Some(&'"') && self.source.peek().is_some() {
            if self.source.peek() == Some(&'\n') {
                self.line += 1;
            }
            string_value.push(self.next().unwrap());
        }
        if self.source.peek().is_none() {
            return Err(Errors::UntermitedString);
        }
        self.next();
        self.add_token_with_literal(TokenType::STRING, Some(LiteralType::String(string_value)));
        Ok(())
    }

    fn number(&mut self, char: char) -> Result<(), Errors> {
        let mut num: String = String::from(char);

        while self.source.peek().is_some_and(|char| char.is_digit(10)) {
            num.push(self.next().unwrap());
        }

        if self.source.peek() == Some(&'.') {
            self.next();
            if !self.source.peek().is_some_and(|char| char.is_digit(10)) {
                self.add_token_with_literal(
                    TokenType::NUMBER,
                    Some(LiteralType::Number(num.parse::<f32>()?)),
                );
                self.add_token(TokenType::DOT);
                return Ok(());
            }

            num.push('.');
            while self.source.peek().is_some_and(|char| char.is_digit(10)) {
                num.push(self.next().unwrap());
            }
        }

        self.add_token_with_literal(
            TokenType::NUMBER,
            Some(LiteralType::Number(num.parse::<f32>()?)),
        );
        Ok(())
    }

    // fn peek_next(&self) -> Option<char> {
    //     // if self.current + 1 >= self.source.len() {
    //     //     return None;
    //     // }
    //     self.next_if().peek()
    // }

    fn identifier(&mut self, char: char) {
        let mut text: String = String::from(char);
        while self
            .source
            .peek()
            .is_some_and(|char| char.is_alphanumeric())
        {
            text.push(self.next().unwrap());
        }
        let token_type = match text.as_str() {
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
