use std::ops::Range;

use crate::{
    error_handler::Errors,
    tokens::{LiteralType, Span, Token, TokenType},
};

pub struct Lexer {
    source: String,
    pub token_list: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
    start_column: usize,
    // Indentation tracking
    indent_stack: Vec<usize>,
    at_line_start: bool,
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
            indent_stack: vec![0], // Base indentation level
            at_line_start: true,
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
        
        // Emit DEDENT tokens for remaining indentation levels
        while self.indent_stack.len() > 1 {
            self.emit_dedent();
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
    
    fn emit_newline(&mut self) {
        let span = Span::new(self.line, self.column, self.line, self.column);
        self.token_list.push(Token::with_span(
            TokenType::NEWLINE,
            "\n".to_string(),
            None,
            span,
        ));
    }
    
    fn emit_indent(&mut self, indent_level: usize) {
        let span = Span::new(self.line, self.column, self.line, self.column);
        self.token_list.push(Token::with_span(
            TokenType::INDENT,
            " ".repeat(indent_level),
            None,
            span,
        ));
        self.indent_stack.push(indent_level);
    }
    
    fn emit_dedent(&mut self) {
        let span = Span::new(self.line, self.column, self.line, self.column);
        self.token_list.push(Token::with_span(
            TokenType::DEDENT,
            String::default(),
            None,
            span,
        ));
        self.indent_stack.pop();
    }
    fn scan_token(&mut self) -> Result<(), Errors> {
        let character = self.advance();
        
        // Handle indentation at line start
        if self.at_line_start {
            self.at_line_start = false;
            if character == '\n' {
                self.emit_newline();
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
                return Ok(());
            } else if character == ' ' || character == '\t' {
                // Count leading whitespace
                let mut indent = 0;
                if character == ' ' {
                    indent = 1;
                } else if character == '\t' {
                    indent = 4; // Treat tab as 4 spaces
                }
                
                while !self.is_at_end() {
                    let ch = self.peek();
                    match ch {
                        Some(' ') => { indent += 1; self.advance(); }
                        Some('\t') => { indent += 4; self.advance(); }
                        _ => break,
                    }
                }
                
                // Check if next char is newline (empty line) or comment
                let next = self.peek();
                if next == Some('\n') || next == Some('/') {
                    // Empty line or comment line - don't change indentation
                    if next == Some('\n') {
                        self.emit_newline();
                        self.line += 1;
                        self.column = 1;
                        self.at_line_start = true;
                    }
                    return Ok(());
                }
                
                // Handle indentation changes
                let current_indent = *self.indent_stack.last().unwrap();
                if indent > current_indent {
                    self.emit_indent(indent);
                } else if indent < current_indent {
                    while indent < *self.indent_stack.last().unwrap() {
                        self.emit_dedent();
                    }
                    // If indent doesn't match any level, it's an error
                    if indent != *self.indent_stack.last().unwrap() {
                        return Err(Errors::UnexpectedChar);
                    }
                }
                // indent == current_indent: no change needed
                return Ok(());
            } else {
                // Non-whitespace at line start - check for dedent
                let new_indent = 0; // Non-indented line (column 1)
                while new_indent < *self.indent_stack.last().unwrap() {
                    self.emit_dedent();
                }
            }
        }
        
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
            '\n' => {
                self.emit_newline();
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }
            ';' => {
                // Treat semicolon as newline for backward compatibility
                self.emit_newline();
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            }
            ' ' | '\r' | '\t' => {
                // Whitespace within line - ignore
            }
            '"' => self.string()?,
            char if char.is_ascii_digit() => self.number()?,
            char if char.is_alphabetic() || char == '_' => self.identifier(),
            _char => {
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
        if self.source.chars().nth(self.current).unwrap() == pattern {
            self.current += 1;
            self.column += 1;
            true
        } else {
            false
        }
    }
    
    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, None)
    }
    
    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<LiteralType>) {
        let text = &self.source[self.start..self.current];
        let span = Span::new(self.line, self.start_column, self.line, self.column);
        self.token_list.push(Token::with_span(
            token_type,
            text.to_string(),
            literal,
            span,
        ));
    }
    
    fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }
        self.source.chars().nth(self.current)
    }
    
    fn string(&mut self) -> Result<(), Errors> {
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if ch == '"' {
                break;
            }
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            }
            value.push(self.advance());
        }
        
        if self.is_at_end() {
            return Err(Errors::UnterminatedString);
        }
        
        // Consume closing quote
        self.advance();
        
        let literal = LiteralType::String(value.clone());
        self.add_token_with_literal(TokenType::STRING, Some(literal));
        Ok(())
    }
    
    fn block_comment(&mut self) -> Result<(), Errors> {
        let mut depth = 1;
        while depth > 0 && !self.is_at_end() {
            let ch = self.advance();
            if ch == '*' && self.peek() == Some('/') {
                self.advance();
                depth -= 1;
            } else if ch == '/' && self.peek() == Some('*') {
                self.advance();
                depth += 1;
            }
        }
        if depth > 0 {
            return Err(Errors::UnterminatedGroup);
        }
        Ok(())
    }
    
    fn number(&mut self) -> Result<(), Errors> {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        
        // Fractional part
        if self.peek() == Some('.') {
            self.advance();
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        let text = &self.source[self.start..self.current];
        if let Ok(value) = text.parse::<f32>() {
            let literal = LiteralType::Number(value);
            self.add_token_with_literal(TokenType::NUMBER, Some(literal));
        }
        Ok(())
    }
    
    fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            return None;
        }
        self.source.chars().nth(self.current + 1)
    }
    
    fn identifier(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
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
            "super" => TokenType::SUPER,
            "this" => TokenType::THIS,
            "true" => TokenType::TRUE,
            "let" => TokenType::LET,
            "while" => TokenType::WHILE,
            _ => TokenType::IDENTIFIER,
        };
        
        let literal = if token_type == TokenType::IDENTIFIER {
            None
        } else if matches!(token_type, TokenType::TRUE | TokenType::FALSE) {
            Some(LiteralType::Bool(matches!(token_type, TokenType::TRUE)))
        } else if token_type == TokenType::NIL {
            Some(LiteralType::Null)
        } else {
            None
        };
        
        self.add_token_with_literal(token_type, literal);
    }
}