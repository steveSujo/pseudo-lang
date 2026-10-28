// pIR (Pseudo Intermediate Representation) Parser
// Hyper-compressed LLM-optimized bytecode-style text format

use crate::parser::{Expression, Statment};
use crate::tokens::{LiteralType, Span, Token, TokenType};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum PirTokenType {
    // Opcodes (single-char)
    VarDecl,      // v
    Assign,       // =
    Print,        // p
    If,           // ?
    While,        // w
    Function,     // f
    Return,       // r
    Class,        // c
    MethodCall,   // m
    
    // Builtin calls (@ + two chars)
    BuiltinRead,    // @rd
    BuiltinWrite,   // @wt
    BuiltinList,    // @ls
    BuiltinExists,  // @ex
    BuiltinIsFile,  // @if
    BuiltinIsDir,   // @id
    BuiltinMkdir,   // @mk
    
    // Delimiters
    LParen,       // (
    RParen,       // )
    LBracket,     // [
    RBracket,     // ]
    
    // Literals/identifiers
    Identifier,   // variable/function/class names
    Number,       // f32
    String,       // "string"
    Bool,         // true/false
    
    // End
    EOF,
}

#[derive(Debug, Clone)]
pub struct PirToken {
    pub token_type: PirTokenType,
    pub lexeme: String,
    pub literal: Option<LiteralType>,
    pub span: Span,
}

impl PirToken {
    fn new(token_type: PirTokenType, lexeme: String, literal: Option<LiteralType>, span: Span) -> Self {
        PirToken { token_type, lexeme, literal, span }
    }
}

#[derive(Debug, Clone)]
pub struct PirError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for PirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}, col {}] {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for PirError {}

pub struct PirLexer {
    source: String,
    tokens: Vec<PirToken>,
    start: usize,
    current: usize,
    line: usize,
    column: usize,
}

impl PirLexer {
    pub fn new(source: String) -> Self {
        PirLexer {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 1,
        }
    }
    
    pub fn scan_tokens(&mut self) -> Result<Vec<PirToken>, PirError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        
        let eof_span = Span::new(self.line, self.column, self.line, self.column);
        self.tokens.push(PirToken::new(
            PirTokenType::EOF,
            String::new(),
            None,
            eof_span,
        ));
        
        Ok(self.tokens.clone())
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }
    
    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap_or('\0');
        self.current += c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }
    
    fn peek(&self) -> char {
        self.source.chars().nth(self.current).unwrap_or('\0')
    }
    
    fn make_span(&self) -> Span {
        Span::new(self.line, self.start_column(), self.line, self.column)
    }
    
    fn start_column(&self) -> usize {
        self.column - (self.current - self.start)
    }
    
    fn add_token(&mut self, token_type: PirTokenType, literal: Option<LiteralType>) {
        let lexeme = self.source[self.start..self.current].to_string();
        let span = self.make_span();
        self.tokens.push(PirToken::new(token_type, lexeme, literal, span));
    }
    
    fn scan_token(&mut self) -> Result<(), PirError> {
        let c = self.advance();
        
        match c {
            // Skip whitespace
            ' ' | '\t' | '\r' | '\n' => Ok(()),
            
            // Comments or operator
            '/' => {
                if self.peek() == '/' {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    return Ok(());
                }
                self.scan_identifier('/')
            }
            // Opcodes
            'v' => {
                if self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                    self.scan_identifier('v')
                } else {
                    self.add_token(PirTokenType::VarDecl, None);
                    Ok(())
                }
            }
            '=' => { self.add_token(PirTokenType::Assign, None); Ok(()) }
            'p' => {
                if self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                    self.scan_identifier('p')
                } else {
                    self.add_token(PirTokenType::Print, None);
                    Ok(())
                }
            }
            '?' => { self.add_token(PirTokenType::If, None); Ok(()) }
            'w' => {
                if self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                    self.scan_identifier('w')
                } else {
                    self.add_token(PirTokenType::While, None);
                    Ok(())
                }
            }
            'f' => {
                if self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                    self.scan_identifier('f')
                } else {
                    self.add_token(PirTokenType::Function, None);
                    Ok(())
                }
            }
            'r' => {
                if self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                    self.scan_identifier('r')
                } else {
                    self.add_token(PirTokenType::Return, None);
                    Ok(())
                }
            }
            'c' => {
                if self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                    self.scan_identifier('c')
                } else {
                    self.add_token(PirTokenType::Class, None);
                    Ok(())
                }
            }
            'm' => {
                if self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                    self.scan_identifier('m')
                } else {
                    self.add_token(PirTokenType::MethodCall, None);
                    Ok(())
                }
            }
            // Delimiters
            '(' => { self.add_token(PirTokenType::LParen, None); Ok(()) }
            ')' => { self.add_token(PirTokenType::RParen, None); Ok(()) }
            '[' => { self.add_token(PirTokenType::LBracket, None); Ok(()) }
            ']' => { self.add_token(PirTokenType::RBracket, None); Ok(()) }
            
            // Builtin calls (@ + code)
            '@' => self.scan_builtin(),
            
            // String literals
            '"' => self.scan_string(),
            
            // Numbers
            '0'..='9' => self.scan_number(c),
            '-' => {
                // Check if it's a negative number or an operator
                let next = self.peek();
                if next.is_ascii_digit() {
                    self.scan_number(c)
                } else {
                    // It's an operator, scan as identifier (will match operator in parse_prefix_expr)
                    self.scan_identifier(c)
                }
            }
            
            // Identifiers and operators
            'a'..='z' | 'A'..='Z' | '_' | '+' | '*' | '>' | '<' | '!' => self.scan_identifier(c),
            
            _ => Err(self.error(&format!("Unexpected character: '{}'", c))),
        }
    }
    
    fn scan_builtin(&mut self) -> Result<(), PirError> {
        let code = format!("{}{}", self.advance(), self.advance());
        let token_type = match code.as_str() {
            "rd" => PirTokenType::BuiltinRead,
            "wt" => PirTokenType::BuiltinWrite,
            "ls" => PirTokenType::BuiltinList,
            "ex" => PirTokenType::BuiltinExists,
            "if" => PirTokenType::BuiltinIsFile,
            "id" => PirTokenType::BuiltinIsDir,
            "mk" => PirTokenType::BuiltinMkdir,
            _ => return Err(self.error(&format!("Unknown builtin code: @{}", code))),
        };
        let span = Span::new(self.line, self.start_column() - 1, self.line, self.column);
        self.tokens.push(PirToken::new(token_type, format!("@{}", code), None, span));
        Ok(())
    }
    
    fn scan_string(&mut self) -> Result<(), PirError> {
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek() != '"' {
            let c = self.advance();
            if c == '\\' && !self.is_at_end() {
                let escaped = self.advance();
                value.push(match escaped {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '"' => '"',
                    '\\' => '\\',
                    _ => escaped,
                });
            } else {
                value.push(c);
            }
        }
        
        if self.is_at_end() {
            return Err(self.error("Unterminated string"));
        }
        
        // Consume closing quote
        self.advance();
        
        self.add_token(PirTokenType::String, Some(LiteralType::String(value)));
        Ok(())
    }
    
    fn scan_number(&mut self, first_char: char) -> Result<(), PirError> {
        let mut num_str = String::new();
        num_str.push(first_char);
        
        while !self.is_at_end() {
            let c = self.peek();
            if c.is_ascii_digit() || c == '.' {
                num_str.push(self.advance());
            } else {
                break;
            }
        }
        
        let value: f32 = num_str.parse().map_err(|_| self.error("Invalid number"))?;
        self.add_token(PirTokenType::Number, Some(LiteralType::Number(value)));
        Ok(())
    }
    
    fn scan_identifier(&mut self, first_char: char) -> Result<(), PirError> {
        let mut ident = String::new();
        ident.push(first_char);
        
        while !self.is_at_end() {
            let c = self.peek();
            if c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-' || c == '*' || c == '/' || c == '>' || c == '<' || c == '=' || c == '!' {
                ident.push(self.advance());
            } else {
                break;
            }
        }
        
        // Check for boolean literals
        let (token_type, literal) = match ident.as_str() {
            "true" => (PirTokenType::Bool, Some(LiteralType::Bool(true))),
            "false" => (PirTokenType::Bool, Some(LiteralType::Bool(false))),
            _ => (PirTokenType::Identifier, None),
        };
        
        self.add_token(token_type, literal);
        Ok(())
    }
    
    fn error(&self, message: &str) -> PirError {
        PirError {
            message: message.to_string(),
            line: self.line,
            column: self.start_column(),
        }
    }
}

pub struct PirParser {
    tokens: Vec<PirToken>,
    current: usize,
}

impl PirParser {
    pub fn new(tokens: Vec<PirToken>) -> Self {
        PirParser { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Vec<Statment>, PirError> {
        let mut statements = Vec::new();
        
        while !self.check(PirTokenType::EOF) {
            statements.push(self.statement()?);
        }
        
        Ok(statements)
    }
    
    fn check(&self, token_type: PirTokenType) -> bool {
        if self.is_at_end() {
            // At EOF, only match if explicitly checking for EOF
            matches!(token_type, PirTokenType::EOF)
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(&token_type)
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || matches!(self.peek().token_type, PirTokenType::EOF)
    }
    
    fn peek(&self) -> &PirToken {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &PirToken {
        &self.tokens[self.current - 1]
    }
    
    fn advance(&mut self) -> &PirToken {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn consume(&mut self, token_type: PirTokenType, message: &str) -> Result<&PirToken, PirError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(PirError {
                message: message.to_string(),
                line: self.peek().span.start_line,
                column: self.peek().span.start_col,
            })
        }
    }
    
    fn is_identifier_token(&self, token_type: &PirTokenType) -> bool {
        matches!(token_type,
            PirTokenType::Identifier 
            | PirTokenType::VarDecl 
            | PirTokenType::Assign 
            | PirTokenType::Print 
            | PirTokenType::If 
            | PirTokenType::While 
            | PirTokenType::Function 
            | PirTokenType::Return 
            | PirTokenType::Class 
            | PirTokenType::MethodCall
            | PirTokenType::Bool
        )
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, PirError> {
        let token = self.peek();
        if self.is_identifier_token(&token.token_type) {
            let lexeme = token.lexeme.clone();
            self.advance();
            Ok(lexeme)
        } else {
            Err(PirError {
                message: message.to_string(),
                line: token.span.start_line,
                column: token.span.start_col,
            })
        }
    }
    
    fn statement(&mut self) -> Result<Statment, PirError> {
        let token = self.peek();
        
        match token.token_type {
            PirTokenType::VarDecl => self.var_decl(),
            PirTokenType::Assign => self.assign(),
            PirTokenType::Print => self.print_stmt(),
            PirTokenType::If => self.if_stmt(),
            PirTokenType::While => self.while_stmt(),
            PirTokenType::Function => self.function_stmt(),
            PirTokenType::Return => self.return_stmt(),
            PirTokenType::Class => self.class_stmt(),
            PirTokenType::MethodCall => self.method_call_stmt(),
            PirTokenType::LParen => self.expression_statement(),
            PirTokenType::BuiltinRead
            | PirTokenType::BuiltinWrite
            | PirTokenType::BuiltinList
            | PirTokenType::BuiltinExists
            | PirTokenType::BuiltinIsFile
            | PirTokenType::BuiltinIsDir
            | PirTokenType::BuiltinMkdir => self.builtin_call_stmt(),
            _ => Err(PirError {
                message: format!("Unexpected token in statement: {:?}", token.token_type),
                line: token.span.start_line,
                column: token.span.start_col,
            }),
        }
    }
    
    // v name expr
    fn var_decl(&mut self) -> Result<Statment, PirError> {
        let token_span = self.peek().span;
        self.advance(); // consume 'v'
        let name = self.consume_identifier("Expected variable name after 'v'")?;
        let expr = self.expression()?;
        
        let tok = Token::with_span(TokenType::IDENTIFIER, name.clone(), None, token_span);
        Ok(Statment::LetStatement(tok, expr))
    }
    
    // = name expr
    fn assign(&mut self) -> Result<Statment, PirError> {
        self.advance(); // consume '='
        let name = self.consume_identifier("Expected variable name after '='")?;
        let expr = self.expression()?;
        
        let assign_expr = Expression::Assign(name, Box::new(expr));
        Ok(Statment::ExpressionStatement(assign_expr))
    }
    
    
    // p expr
    fn print_stmt(&mut self) -> Result<Statment, PirError> {
        self.advance(); // consume 'p'
        let expr = self.expression()?;
        Ok(Statment::PrintStatement(expr))
    }
    
    // ? cond [then] [else]
    fn if_stmt(&mut self) -> Result<Statment, PirError> {
        self.advance(); // consume '?'
        let condition = self.expression()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.check(PirTokenType::LBracket) {
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };
        
        Ok(Statment::IfStatement(
            condition,
            Box::new(then_branch),
            else_branch,
        ))
    }
    
    // w cond [body]
    fn while_stmt(&mut self) -> Result<Statment, PirError> {
        self.advance(); // consume 'w'
        let condition = self.expression()?;
        let body = self.parse_block()?;
        
        Ok(Statment::WhileStatement(condition, Box::new(body)))
    }
    
    // f name (args) [body]
    fn function_stmt(&mut self) -> Result<Statment, PirError> {
        self.advance(); // consume 'f'
        let name = self.consume_identifier("Expected function name after 'f'")?;
        
        // Parse parameters: (arg1 arg2 ...)
        self.consume(PirTokenType::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        while !self.check(PirTokenType::RParen) && !self.is_at_end() {
            let param = self.consume_identifier("Expected parameter name")?;
            params.push(param);
        }
        self.consume(PirTokenType::RParen, "Expected ')' after parameters")?;
        
        let body = self.parse_block()?;
        
        let func_expr = Expression::Function(params, Box::new(body));
        let assign_expr = Expression::Assign(name, Box::new(func_expr));
        Ok(Statment::ExpressionStatement(assign_expr))
    }
    
    // r expr?
    fn return_stmt(&mut self) -> Result<Statment, PirError> {
        self.advance(); // consume 'r'
        let expr = if self.check(PirTokenType::LParen) || self.is_identifier_token(&self.peek().token_type) 
            || self.check(PirTokenType::Number) || self.check(PirTokenType::String)
            || self.check(PirTokenType::Bool) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        let return_expr = Expression::Return(expr);
        Ok(Statment::ExpressionStatement(return_expr))
    }
    
    fn class_stmt(&mut self) -> Result<Statment, PirError> {
        let token_span = self.peek().span;
        self.advance(); // consume 'c'
        let name = self.consume_identifier("Expected class name after 'c'")?;
        let methods_block = self.parse_block()?;
        let methods = match methods_block {
            Statment::BlockStatement(stmts) => stmts,
            _ => vec![],
        };
        
        let class_token = Token::with_span(TokenType::CLASS, name.clone(), None, token_span);
        Ok(Statment::ClassStatement(class_token, methods, None))
    }
    
    // m instance method (args...)
    fn method_call_stmt(&mut self) -> Result<Statment, PirError> {
        let expr = self.method_call_expr()?;
        Ok(Statment::ExpressionStatement(expr))
    }
    
    // @builtin args...
    fn builtin_call_stmt(&mut self) -> Result<Statment, PirError> {
        let expr = self.builtin_call_expr()?;
        Ok(Statment::ExpressionStatement(expr))
    }
    
    // (expr) - prefix expression as statement
    fn expression_statement(&mut self) -> Result<Statment, PirError> {
        let expr = self.expression()?;
        Ok(Statment::ExpressionStatement(expr))
    }
    
    fn parse_block(&mut self) -> Result<Statment, PirError> {
        self.consume(PirTokenType::LBracket, "Expected '[' to start block")?;
        let mut statements = Vec::new();
        
        while !self.check(PirTokenType::RBracket) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        
        self.consume(PirTokenType::RBracket, "Expected ']' to end block")?;
        Ok(Statment::BlockStatement(statements))
    }
    
    fn expression(&mut self) -> Result<Expression, PirError> {
        let token = self.peek();
        
        match token.token_type {
            PirTokenType::LParen => self.parse_prefix_expr(),
            _ if self.is_identifier_token(&token.token_type) => self.parse_identifier_expr(),
            PirTokenType::Number => self.parse_literal(),
            PirTokenType::String => self.parse_literal(),
            PirTokenType::Bool => self.parse_literal(),
            PirTokenType::BuiltinRead
            | PirTokenType::BuiltinWrite
            | PirTokenType::BuiltinList
            | PirTokenType::BuiltinExists
            | PirTokenType::BuiltinIsFile
            | PirTokenType::BuiltinIsDir
            | PirTokenType::BuiltinMkdir => self.builtin_call_expr(),
            _ => Err(PirError {
                message: format!("Unexpected token in expression: {:?}", token.token_type),
                line: token.span.start_line,
                column: token.span.start_col,
            }),
        }
    }
    
    // (op arg1 arg2 ...) - strict prefix notation
    // Also handles function calls: (name arg1 arg2 ...)
    fn parse_prefix_expr(&mut self) -> Result<Expression, PirError> {
        self.consume(PirTokenType::LParen, "Expected '('")?;
        
        let peek_type = self.peek().token_type.clone();
        match peek_type {
            PirTokenType::BuiltinRead
            | PirTokenType::BuiltinWrite
            | PirTokenType::BuiltinList
            | PirTokenType::BuiltinExists
            | PirTokenType::BuiltinIsFile
            | PirTokenType::BuiltinIsDir
            | PirTokenType::BuiltinMkdir => {
                let call = self.builtin_call_expr()?;
                self.consume(PirTokenType::RParen, "Expected ')' after builtin call")?;
                return Ok(call);
            }
            _ => {}
        }

        let op_type = self.peek().token_type.clone();
        if !self.is_identifier_token(&op_type) {
            return Err(PirError {
                message: "Expected operator, function name, or builtin after '('".to_string(),
                line: self.peek().span.start_line,
                column: self.peek().span.start_col,
            });
        }
        let op_token = self.advance();
        let op = op_token.lexeme.clone();
        let is_operator = matches!(op.as_str(), 
            "+" | "-" | "*" | "/" | ">" | "<" | ">=" | "<=" | "==" | "!=" | "!"
        );
        
        if is_operator {
            // Parse arguments for prefix operator
            let mut args = Vec::new();
            while !self.check(PirTokenType::RParen) && !self.is_at_end() {
                args.push(self.expression()?);
            }
            self.consume(PirTokenType::RParen, "Expected ')' after arguments")?;
            self.build_binary_expr(&op, args)
        } else {
            // Function call: (name arg1 arg2 ...)
            let mut args = Vec::new();
            while !self.check(PirTokenType::RParen) && !self.is_at_end() {
                args.push(self.expression()?);
            }
            self.consume(PirTokenType::RParen, "Expected ')' after arguments")?;
            Ok(Expression::Call(op, args))
        }
    }
    fn build_binary_expr(&self, op: &str, args: Vec<Expression>) -> Result<Expression, PirError> {
        let (left, right) = match args.len() {
            2 => (args[0].clone(), args[1].clone()),
            1 if op == "-" => (Expression::Literal(Token::with_span(
                TokenType::NUMBER, "0".to_string(), Some(LiteralType::Number(0.0)), Span::default()
            )), args[0].clone()),
            _ => return Err(PirError {
                message: format!("Operator '{}' expects 1 or 2 arguments, got {}", op, args.len()),
                line: 0,
                column: 0,
            }),
        };
        
        let token_type = match op {
            "+" => TokenType::PLUS,
            "-" => TokenType::MINUS,
            "*" => TokenType::STAR,
            "/" => TokenType::SLASH,
            ">" => TokenType::GREATER,
            "<" => TokenType::LESS,
            ">=" => TokenType::GreaterEqual,
            "<=" => TokenType::LessEqual,
            "==" => TokenType::EqualEqual,
            "!=" => TokenType::BangEqual,
            "!" => TokenType::BANG,
            _ => return Err(PirError {
                message: format!("Unknown operator: {}", op),
                line: 0,
                column: 0,
            }),
        };
        
        let token = Token::with_span(token_type, op.to_string(), None, Span::default());
        Ok(Expression::Binary(Box::new(left), token, Box::new(right)))
    }
    
    fn parse_identifier_expr(&mut self) -> Result<Expression, PirError> {
        let name = self.consume_identifier("Expected identifier")?;
        // Check if it's a function call: (name arg1 arg2 ...)
        if self.check(PirTokenType::LParen) {
            self.consume(PirTokenType::LParen, "Expected '('")?;
            let mut args = Vec::new();
            while !self.check(PirTokenType::RParen) && !self.is_at_end() {
                args.push(self.expression()?);
            }
            self.consume(PirTokenType::RParen, "Expected ')' after arguments")?;
            Ok(Expression::Call(name, args))
        } else {
            // Variable reference: Expression::Literal(Token { token_type: TokenType::IDENTIFIER, text: name, ... })
            Ok(Expression::Literal(Token::with_span(
                TokenType::IDENTIFIER, name, None, Span::default()
            )))
        }
    }
    
    fn method_call_expr(&mut self) -> Result<Expression, PirError> {
        self.advance(); // consume 'm'
        
        let instance = self.consume_identifier("Expected instance name after 'm'")?;
        let method = self.consume_identifier("Expected method name")?;
        // Parse arguments: (arg1 arg2 ...)
        let mut args = Vec::new();
        if self.check(PirTokenType::LParen) {
            self.consume(PirTokenType::LParen, "Expected '(' after method name")?;
            while !self.check(PirTokenType::RParen) && !self.is_at_end() {
                args.push(self.expression()?);
            }
            self.consume(PirTokenType::RParen, "Expected ')' after arguments")?;
        }
        
        // Build: Call(method, [instance, ...args])
        let mut call_args = vec![Expression::Assign(instance, Box::new(Expression::Literal(
            Token::with_span(TokenType::NIL, "nil".to_string(), None, Span::default())
        )))];
        call_args.extend(args);
        
        Ok(Expression::Call(method, call_args))
    }
    
    fn builtin_call_expr(&mut self) -> Result<Expression, PirError> {
        let builtin_token = self.advance();
        let builtin_name = match builtin_token.token_type {
            PirTokenType::BuiltinRead => "readFile",
            PirTokenType::BuiltinWrite => "writeFile",
            PirTokenType::BuiltinList => "listDir",
            PirTokenType::BuiltinExists => "exists",
            PirTokenType::BuiltinIsFile => "isFile",
            PirTokenType::BuiltinIsDir => "isDir",
            PirTokenType::BuiltinMkdir => "mkdir",
            _ => return Err(PirError {
                message: "Invalid builtin token".to_string(),
                line: builtin_token.span.start_line,
                column: builtin_token.span.start_col,
            }),
        };
        
        // Parse arguments
        let mut args = Vec::new();
        while !self.check(PirTokenType::RParen) 
            && !self.check(PirTokenType::RBracket)
            && !self.is_at_end() {
            if self.is_statement_start(self.peek()) {
                break;
            }
            args.push(self.expression()?);
        }
        
        Ok(Expression::Call(builtin_name.to_string(), args))
    }
    
    fn is_statement_start(&self, token: &PirToken) -> bool {
        matches!(token.token_type,
            PirTokenType::VarDecl | PirTokenType::Assign | PirTokenType::Print
            | PirTokenType::If | PirTokenType::While | PirTokenType::Function
            | PirTokenType::Return | PirTokenType::Class | PirTokenType::MethodCall
            | PirTokenType::EOF)
    }
    
    fn parse_literal(&mut self) -> Result<Expression, PirError> {
        let token = self.advance();
        let lit_token = match token.token_type {
            PirTokenType::Number => {
                if let Some(LiteralType::Number(n)) = &token.literal {
                    Token::with_span(TokenType::NUMBER, n.to_string(), Some(LiteralType::Number(*n)), token.span)
                } else {
                    return Err(PirError { message: "Invalid number literal".to_string(), line: token.span.start_line, column: token.span.start_col });
                }
            }
            PirTokenType::String => {
                if let Some(LiteralType::String(s)) = &token.literal {
                    Token::with_span(TokenType::STRING, s.clone(), Some(LiteralType::String(s.clone())), token.span)
                } else {
                    return Err(PirError { message: "Invalid string literal".to_string(), line: token.span.start_line, column: token.span.start_col });
                }
            }
            PirTokenType::Bool => {
                if let Some(LiteralType::Bool(b)) = &token.literal {
                    Token::with_span(if *b { TokenType::TRUE } else { TokenType::FALSE }, b.to_string(), Some(LiteralType::Bool(*b)), token.span)
                } else {
                    return Err(PirError { message: "Invalid bool literal".to_string(), line: token.span.start_line, column: token.span.start_col });
                }
            }
            _ => return Err(PirError { message: "Not a literal".to_string(), line: token.span.start_line, column: token.span.start_col }),
        };
        
        Ok(Expression::Literal(lit_token))
    }
}

/// Public entry point: parse pIR source into Stmt AST
pub fn parse_pir(source: &str) -> Result<Vec<Statment>, PirError> {
    let mut lexer = PirLexer::new(source.to_string());
    let tokens = lexer.scan_tokens()?;
    let mut parser = PirParser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_var_print() {
        let source = r#"
            v x 10
            p x
        "#;
        let stmts = parse_pir(source).unwrap();
        assert_eq!(stmts.len(), 2);
    }
    
    #[test]
    fn test_if_else() {
        let source = r#"
            v x 5
            ? (> x 0) [ p "positive" ] [ p "zero" ]
        "#;
        let stmts = parse_pir(source).unwrap();
        assert_eq!(stmts.len(), 2);
    }
    
    #[test]
    fn test_while_loop() {
        let source = r#"
            v i 3
            w (> i 0) [ p i = i (- i 1) ]
        "#;
        let stmts = parse_pir(source).unwrap();
        assert_eq!(stmts.len(), 2);
    }
    
    #[test]
    fn test_function() {
        let source = r#"
            f add (a b) [ r (+ a b) ]
            v result (add 2 3)
            p result
        "#;
        let stmts = parse_pir(source).unwrap();
        assert_eq!(stmts.len(), 3);
    }
    
    #[test]
    fn test_builtin() {
        let source = r#"
            @wt "test.txt" "hello"
            v content (@rd "test.txt")
            p content
        "#;
        let stmts = parse_pir(source).unwrap();
        assert_eq!(stmts.len(), 3);
    }
    
    #[test]
    fn test_class() {
        let source = r#"
            c Breakfast [
                f cook () [ p "Eggs!" ]
                f serve (who) [ p (+ "Enjoy, " who) ]
            ]
            v b (Breakfast)
            m b cook ()
            m b serve ("world")
        "#;
        let stmts = parse_pir(source).unwrap();
        assert_eq!(stmts.len(), 4);
    }
    
    #[test]
    fn test_prefix_ops() {
        let source = r#"
            v x (+ 1 2)
            v y (- 10 3)
            v z (* 4 5)
            v w (/ 20 4)
            v neg (- 5)
            v cmp (> x 0)
        "#;
        let _stmts = parse_pir(source).unwrap();
        let stmts = parse_pir(source).unwrap();
        eprintln!("STMTS (len {}): {:?}", stmts.len(), stmts);
        assert_eq!(stmts.len(), 6);
    }
}