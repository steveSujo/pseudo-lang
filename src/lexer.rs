use std::ops::Range;

use crate::{
    error_handler::Errors,
    tokens::{LiteralType, Span, Token, TokenType},
};

pub struct Lexer {
    source: String,
    //TODO: Tokens
    pub token_list: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    start_column: usize,
}
impl Lexer {
    pub fn new(source: String) -> Self {
        Lexer {
            source,
            token_list: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
            start_column: 1,
        }
    }
    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Errors> {
        let mut had_error = false;
        while !self.is_at_end() {
            self.start = self.current;
            self.start_column = self.column;
            if let Err(e) = self.scan_token() {
                println!("{}", e);
                had_error = true;
            }
        }

        let eof_span = Span::new(self.line, self.column, self.line, self.column);
        self.token_list.push(Token::with_span(
            TokenType::EOF,
            String::default(),
            None,
            eof_span,
        ));

        if had_error {
            Err(Errors::UnexpectedChar)
        } else {
            Ok(self.token_list.clone())
        }
    }

    pub fn get_tokens(&self) -> Vec<Token> {
        self.token_list.clone()
    }

    fn scan_token(&mut self) -> Result<(), Errors> {
        let character = self.advance();
        match character {
            '(' => self.add_token(TokenType::LeftPara),
            ')' => self.add_token(TokenType::RightPara),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::COMMA),
            ':' => self.add_token(TokenType::COLON),
            '.' => self.add_token(TokenType::DOT),
            '-' => {
                if self.match_next('>') {
                    self.add_token(TokenType::ARROW);
                } else {
                    self.add_token(TokenType::MINUS);
                }
            }
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
                if self.match_next('>') {
                    self.add_token(TokenType::ARROW);
                } else {
                    let token_type = if self.match_next('=') {
                        TokenType::EqualEqual
                    } else {
                        TokenType::EQUAL
                    };
                    self.add_token(token_type);
                }
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
                } else if self.match_next('*') {
                    self.block_comment()?;
                } else {
                    self.add_token(TokenType::SLASH)
                }
            }
            '\n' => self.line += 1,
            ' ' => {}
            '\r' => {}
            '\t' => {}
            '"' => self.string()?,
            char if char.is_ascii_digit() => self.number()?,
            char if char.is_alphabetic() || char == '_' => self.identifier(),
            _char => {
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
        if character == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

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
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None)
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<LiteralType>) {
        let sub_string = &self.source[self.start..self.current];
        let span = Span::new(self.line, self.start_column, self.line, self.column);
        self.token_list.push(Token::with_span(
            token_type,
            sub_string.to_string(),
            literal,
            span,
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
            return Err(Errors::UnterminatedString);
        }
        self.advance();
        let string_value = &self.source[self.start + 1..self.current - 1];
        self.add_token_with_literal(
            TokenType::STRING,
            Some(LiteralType::String(string_value.to_string())),
        );
        Ok(())
    }

    fn block_comment(&mut self) -> Result<(), Errors> {
        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            match self.advance() {
                '*' if self.peek() == Some('/') => {
                    self.advance();
                    depth -= 1;
                }
                '/' if self.peek() == Some('*') => {
                    self.advance();
                    depth += 1;
                }
                '\n' => {
                    // line already tracked in advance()
                }
                _ => {}
            }
        }
        if depth > 0 {
            return Err(Errors::UnterminatedString); // reuse for now
        }
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
        while self
            .peek()
            .is_some_and(|char| char.is_alphanumeric() || char == '_')
        {
            self.advance();
        }
        let text = &self.source[self.start..self.current];
        let token_type = match text {
            "and" => TokenType::AND,
            "class" => TokenType::CLASS,
            "else" => TokenType::ELSE,
            "false" => TokenType::FALSE,
            "fn" => TokenType::FN,
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
