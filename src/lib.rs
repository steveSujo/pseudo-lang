pub mod error_handler;
pub mod lexer;
pub mod parser;

pub mod tokens {
    use std::fmt::{self, Display};

    #[derive(Debug, Clone)]
    pub enum LiteralType {
        String(String),
        Number(f32),
    }

    impl LiteralType {
        pub fn print(&self) -> String {
            match self {
                LiteralType::String(x) => x.to_string(),
                LiteralType::Number(x) => x.to_string(),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone)]
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
    #[derive(Debug, Clone)]
    pub struct Token {
        pub token_type: TokenType,
        pub text: String,
        pub literal: Option<LiteralType>,
        pub line: usize,
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
