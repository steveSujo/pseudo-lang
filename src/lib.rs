pub mod lexer {
    use std::{
        collections::HashMap,
        ops::{Index, Range},
    };

    use crate::{
        error_handler::ErrorSet,
        tokens::{LiteralType, Token, TokenType},
    };

    pub struct Lexer {
        source: String,
        //TODO: Tokens
        token_list: Vec<Token>,
        start: usize,
        current: usize,
        error_set: ErrorSet,
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
                error_set: ErrorSet::default(),
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
                '"' => self.string(),
                char if char.is_digit(10) => self.number(),
                char if char.is_alphabetic() || char == '_' => self.identifier(),
                char => {
                    //error unexpeted char
                    self.error_set.error_where(
                        self.line,
                        char.to_string(),
                        "Unexpeted char".to_owned(),
                    )
                }
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

        fn string(&mut self) {
            while self.peek() != Some('"') && !self.is_at_end() {
                if self.peek() == Some('\n') {
                    self.line += 1;
                }
                self.advance();
            }
            if self.is_at_end() {
                self.error_set
                    .error(self.line, String::from("Untermited String"));
            }
            self.advance();
            let string_value = &self.source[self.start + 1..self.current - 1];
            self.add_token_with_literal(
                TokenType::STRING,
                Some(LiteralType::String(string_value.to_string())),
            );
        }

        fn number(&mut self) {
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
                    self.source[self.start..self.current]
                        .parse::<f32>()
                        .expect("INTERNAl: Number parse error"),
                )),
            )
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
}

pub mod error_handler {
    //TODO: add more types
    // enum ErrorType {
    //     Error,
    //     Warning,
    // }
    #[derive(Default)]
    struct Error {
        line: usize,
        place: String,
        message: String,
    }

    impl Error {
        fn report(&self) {
            println!(
                "[Line {}] Error {}: {}",
                self.line, self.place, self.message
            );
        }
    }
    #[derive(Default)]
    pub struct ErrorSet {
        error_list: Vec<Error>,
        had_error: bool,
    }

    impl ErrorSet {
        pub fn error(&mut self, line: usize, message: String) {
            let error = Error {
                line,
                place: String::new(),
                message,
            };
            error.report();
            self.error_list.push(error);
            self.had_error = true;
        }

        pub fn error_where(&mut self, line: usize, place: String, message: String) {
            let error = Error {
                line,
                place,
                message,
            };
            error.report();
            self.error_list.push(error);
            self.had_error = true;
        }
    }
}

pub mod tokens {
    use std::{
        collections::HashMap,
        fmt::{self, Display},
    };

    #[derive(Debug)]
    pub enum LiteralType {
        String(String),
        Number(f32),
    }

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
        THIS,
        TRUE,
        LET,
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
        literal: Option<LiteralType>,
        line: usize,
    }

    impl Token {
        pub fn new(
            token_type: TokenType,
            text: String,
            literal: Option<LiteralType>,
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
