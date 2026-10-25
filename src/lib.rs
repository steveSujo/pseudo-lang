pub mod error_handler;
pub mod lexer;
pub mod parser;
pub mod fs;
 pub mod lsp;
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
        LeftBracket,
        RightBracket,
        COMMA,
        COLON,
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
        FN,
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

        ARROW,

        EOF,
    }
    // impl Display for TokenType {
    //     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //         write!(f, "{}", self)
    //     }
    // }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Span {
        pub start_line: usize,
        pub start_col: usize,
        pub end_line: usize,
        pub end_col: usize,
    }

    impl Span {
        pub fn new(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Self {
            Span { start_line, start_col, end_line, end_col }
        }
        pub fn single(line: usize, col: usize) -> Self {
            Span { start_line: line, start_col: col, end_line: line, end_col: col }
        }
    }

    impl Default for Span {
        fn default() -> Self {
            Span { start_line: 1, start_col: 1, end_line: 1, end_col: 1 }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Token {
        pub token_type: TokenType,
        pub text: String,
        pub literal: Option<LiteralType>,
        pub line: usize,
        pub span: Span,
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
                span: Span::default(),
            }
        }

        pub fn with_span(
            token_type: TokenType,
            text: String,
            literal: Option<LiteralType>,
            span: Span,
        ) -> Self {
            Token {
                token_type,
                text,
                literal,
                line: span.start_line,
                span,
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
