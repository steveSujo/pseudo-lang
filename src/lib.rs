pub mod lexer {
    use std::ops::Index;

    use crate::tokens::{Token, TokenType};

    pub struct Lexer {
        source: String,
        //TODO: Tokens
        token_list: Vec<Token>,
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
        pub fn scan_tokens(&mut self) -> &Vec<Token> {
            while !self.is_at_end() {
                self.start = self.current;
                self.scan_token();
            }

            self.token_list.push(Token::new(
                TokenType::EOF,
                String::default(),
                None,
                self.line,
            ));

            return &self.token_list;
        }

        fn scan_token(&mut self) {
            let character = self.advance().expect("advace to next char return none");
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
                '/' => self.add_token(TokenType::SLASH),
                '*' => self.add_token(TokenType::STAR),
                _ => {}
            }
        }

        fn is_at_end(&self) -> bool {
            self.current >= self.source.len()
        }

        fn advance(&mut self) -> Option<char> {
            let character = self.source.chars().nth(self.current);

            self.current += 1;

            character
        }

        fn add_token(&mut self, token_type: TokenType) {
            return self.add_token_with_literal(token_type, None);
        }

        fn add_token_with_literal(&mut self, token_type: TokenType, literal: Option<String>) {
            let sub_string = &self.source[self.start..self.current];
            self.token_list.push(Token::new(
                token_type,
                sub_string.to_string(),
                literal,
                self.line,
            ))
        }
    }
}

pub mod error_handler {
    pub fn error(line: String, message: String) {
        report(line, String::new(), message)
    }
    fn report(line: String, place: String, message: String) {
        println!("[Line {line}] Error{place}: {message}")
        // set had error
    }
}

pub mod tokens {
    use std::fmt::{self, Display};

    #[derive(Debug)]
    pub enum TokenType {
        //single char
        LeftPara,
        RightPara,
        LeftBrace,
        RightBrace,
        COMMA,
        DOT,
        MINUS,
        PLUS,
        SEMICOLON,
        SLASH,
        STAR,

        BANG,
        BangEqual,
        EQUAL,
        EqualEqual,
        GREATER,
        GreaterEqual,
        LESS,
        LessEqual,

        IDENTIFIER,
        STRING,
        NUMBER,

        AND,
        CLASS,
        ELSE,
        FALSE,
        FUN,
        FOR,
        IF,
        NIL,
        OR,
        PRINT,
        RETURN,
        SUPER,
        THIS,
        TRUE,
        VAR,
        WHILE,

        EOF,
    }

    // impl Display for TokenType {
    //     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         write!(f, "{}", self)
    //     }
    // }
    #[derive(Debug)]
    pub struct Token {
        token_type: TokenType,
        text: String,
        literal: Option<String>,
        line: usize,
    }

    impl Token {
        pub fn new(
            token_type: TokenType,
            text: String,
            literal: Option<String>,
            line: usize,
        ) -> Self {
            Token {
                token_type,
                text,
                literal,
                line,
            }
        }
    }

    impl Display for Token {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "{:#?} {} {:#?}",
                self.token_type, self.text, self.literal
            )
        }
    }
}
