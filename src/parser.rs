use std::{fmt::Display, iter::Peekable, vec::IntoIter};

use crate::{
    error_handler::Errors,
    tokens::{LiteralType, Token, TokenType},
};

#[derive(Debug, Clone)]
pub enum DataTypes {
    Number(f32),
    Bool(bool),
    Str(String),
}

impl Display for DataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataTypes::Number(num) => write!(f, "{num}"),
            DataTypes::Bool(b) => write!(f, "{b}"),
            DataTypes::Str(s) => write!(f, "\"{s}\""),
        }
    }
}


impl From<DataTypes> for f32 {
    fn from(value: DataTypes) -> Self {
        match value {
            DataTypes::Number(num) => num,
            DataTypes::Bool(true) => 1.0,
            DataTypes::Bool(false) => 0.0,
            DataTypes::Str(_) => 0.0,
        }
    }
}
impl From<f32> for DataTypes {
    fn from(value: f32) -> Self {
        DataTypes::Number(value)
    }
}
impl From<DataTypes> for bool {
    fn from(value: DataTypes) -> Self {
        match value {
            DataTypes::Bool(b) => b,
            DataTypes::Number(0.0) => false,
            DataTypes::Number(_) => true,
            DataTypes::Str(_) => false,
        }
    }
}

impl From<bool> for DataTypes {
    fn from(value: bool) -> Self {
        DataTypes::Bool(value)
    }
}
impl From<Option<LiteralType>> for DataTypes {
    fn from(value: Option<LiteralType>) -> Self {
        match value {
            Some(LiteralType::Number(num)) => DataTypes::Number(num),
            Some(LiteralType::String(s)) => DataTypes::Str(s),
            None => DataTypes::Number(0.0), // nil/false default
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(Token),
    Grouping(Box<Expression>),
    Assign(String, Box<Expression>), // name, value
}

// let var:Option

// visitor pattern for better maintainability
pub trait ExpressionVisitor<T> {
    fn visit_binary(&mut self, left: &Expression, operator: &Token, right: &Expression) -> T;
    fn visit_unary(&mut self, operator: &Token, right: &Expression) -> T;
    fn visit_literal(&mut self, token: &Token) -> T;
    fn visit_grouping(&mut self, expression: &Expression) -> T;
    fn visit_assign(&mut self, name: &str, value: &Expression) -> T;
}
//operations as classes
use std::collections::HashMap;

pub struct Interpret {
    pub environment: HashMap<String, DataTypes>,
}

impl Default for Interpret {
    fn default() -> Self {
        Interpret {
            environment: HashMap::new(),
        }
    }
}

impl ExpressionVisitor<DataTypes> for Interpret {
    fn visit_binary(&mut self, left: &Expression, operator: &Token, right: &Expression) -> DataTypes {
        let left_val = self.visit_expr(left);
        let right_val = self.visit_expr(right);
        match operator.token_type {
            TokenType::MINUS => bin_expr::<f32>(left_val, right_val, |a, b| a - b),
            TokenType::PLUS => {
                match (&left_val, &right_val) {
                    (DataTypes::Number(a), DataTypes::Number(b)) => (a + b).into(),
                    (DataTypes::Str(a), DataTypes::Str(b)) => DataTypes::Str(format!("{}{}", a, b)),
                    _ => unimplemented!("PLUS not supported for these types"),
                }
            }
            TokenType::SLASH => bin_expr::<f32>(left_val, right_val, |a, b| a / b),
            TokenType::STAR => bin_expr::<f32>(left_val, right_val, |a, b| a * b),
            TokenType::EqualEqual => {
                let l: DataTypes = left_val.clone();
                let r: DataTypes = right_val.clone();
                match (l, r) {
                    (DataTypes::Number(a), DataTypes::Number(b)) => (a == b).into(),
                    (DataTypes::Bool(a), DataTypes::Bool(b)) => (a == b).into(),
                    (DataTypes::Str(a), DataTypes::Str(b)) => (a == b).into(),
                    _ => false.into(),
                }
            }
            TokenType::BangEqual => {
                let l: DataTypes = left_val.clone();
                let r: DataTypes = right_val.clone();
                match (l, r) {
                    (DataTypes::Number(a), DataTypes::Number(b)) => (a != b).into(),
                    (DataTypes::Bool(a), DataTypes::Bool(b)) => (a != b).into(),
                    (DataTypes::Str(a), DataTypes::Str(b)) => (a != b).into(),
                    _ => true.into(),
                }
            }
            TokenType::LessEqual => {
                let l: f32 = left_val.into();
                let r: f32 = right_val.into();
                (l <= r).into()
            }
            TokenType::LESS => {
                let l: f32 = left_val.into();
                let r: f32 = right_val.into();
                (l < r).into()
            }
            TokenType::GreaterEqual => {
                let l: f32 = left_val.into();
                let r: f32 = right_val.into();
                (l >= r).into()
            }
            TokenType::GREATER => {
                let l: f32 = left_val.into();
                let r: f32 = right_val.into();
                (l > r).into()
            }
            _ => unimplemented!(),
        }
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expression) -> DataTypes {
        let right_val = self.visit_expr(right);
        match operator.token_type {
            TokenType::MINUS => { -f32::from(right_val) }.into(),
            TokenType::BANG => { !bool::from(right_val) }.into(),
            _ => unimplemented!(),
        }
    }

    fn visit_literal(&mut self, token: &Token) -> DataTypes {
        match token.token_type {
            TokenType::NUMBER => token.literal.clone().into(),
            TokenType::STRING => token.literal.clone().into(),
            TokenType::TRUE => DataTypes::Bool(true),
            TokenType::FALSE => DataTypes::Bool(false),
            TokenType::NIL => DataTypes::Number(0.0),
            TokenType::IDENTIFIER => {
                self.environment
                    .get(&token.text)
                    .cloned()
                    .unwrap_or_else(|| DataTypes::Str(format!("undefined:{}", token.text)))
            }
            _ => unimplemented!(),
        }
    }

    fn visit_grouping(&mut self, expression: &Expression) -> DataTypes {
        self.visit_expr(expression)
    }

    fn visit_assign(&mut self, name: &str, value: &Expression) -> DataTypes {
        let val = self.visit_expr(value);
        self.environment.insert(name.to_string(), val.clone());
        val
    }
}

impl Interpret {
    pub fn visit_expr(&mut self, e: &Expression) -> DataTypes {
        match e {
            Expression::Binary(left, op, right) => self.visit_binary(left, op, right),
            Expression::Unary(op, right) => self.visit_unary(op, right),
            Expression::Literal(token) => self.visit_literal(token),
            Expression::Grouping(exp) => self.visit_grouping(exp),
            Expression::Assign(name, value) => self.visit_assign(name, value),
        }
    }

    pub fn execute(&mut self, stmt: &Statment) -> Option<DataTypes> {
        match stmt {
            Statment::ExpressionStatement(exp) => Some(self.visit_expr(exp)),
            Statment::PrintStatement(exp) => {
                let val = self.visit_expr(exp);
                println!("{}", val);
                None
            }
            Statment::LetStatement(name, exp) => {
                let val = self.visit_expr(exp);
                self.environment.insert(name.text.clone(), val.clone());
                Some(val)
            }
            Statment::IfStatement(condition, then_branch, else_branch) => {
                let cond_val = self.visit_expr(condition);
                if bool::from(cond_val) {
                    self.execute(then_branch);
                } else if let Some(else_stmt) = else_branch {
                    self.execute(else_stmt);
                }
                None
            }
            Statment::WhileStatement(condition, body) => {
                loop {
                    let cond_val = self.visit_expr(condition);
                    if !bool::from(cond_val) {
                        break;
                    }
                    self.execute(body);
                }
                None
            }
            Statment::ForStatement(init, condition, increment, body) => {
                if let Some(init_stmt) = init {
                    self.execute(init_stmt);
                }
                loop {
                    let should_continue = match condition {
                        Some(cond) => bool::from(self.visit_expr(cond)),
                        None => true,
                    };
                    if !should_continue {
                        break;
                    }
                    self.execute(body);
                    if let Some(inc_expr) = increment {
                        self.visit_expr(inc_expr);
                    }
                }
                None
            }
            Statment::BlockStatement(statements) => {
                for stmt in statements {
                    self.execute(stmt);
                }
                None
            }
        }
    }
}

fn bin_expr<D>(exp_a: DataTypes, exp_b: DataTypes, fun: impl FnOnce(D, D) -> D) -> DataTypes
where
    D: From<DataTypes> + Into<DataTypes>,
{
    // let arg_1 = D::from(exp_a);
    // let arg_2 = D::from(exp_b);
    fun(D::from(exp_a), D::from(exp_b)).into()
}

pub struct PrettyPrint;
impl ExpressionVisitor<String> for PrettyPrint {
    fn visit_binary(&mut self, left: &Expression, operator: &Token, right: &Expression) -> String {
        format!(
            "({} {} {})",
            operator.text,
            self.visit_expr(left.clone()),
            self.visit_expr(right.clone())
        )
    }

    fn visit_unary(&mut self, operator: &Token, right: &Expression) -> String {
        format!("({} {})", operator.text, self.visit_expr(right.clone()))
    }

    fn visit_literal(&mut self, token: &Token) -> String {
        token
            .literal
            .as_ref()
            .expect("INTERNAL ERROR: no literal is None")
            .print()
    }

    fn visit_grouping(&mut self, expression: &Expression) -> String {
        format!("(group {})", self.visit_expr(expression.clone()))
    }

    fn visit_assign(&mut self, name: &str, value: &Expression) -> String {
        format!("(assign {} {})", name, self.visit_expr(value.clone()))
    }
}

impl PrettyPrint {
    fn visit_expr(&mut self, e: Expression) -> String {
        match e {
            Expression::Binary(left, op, right) => self.visit_binary(&left, &op, &right),
            Expression::Unary(op, right) => self.visit_unary(&op, &right),
            Expression::Literal(token) => self.visit_literal(&token),
            Expression::Grouping(exp) => self.visit_grouping(&exp),
            Expression::Assign(name, value) => self.visit_assign(&name, &value),
        }
    }
}
pub enum Statment {
    ExpressionStatement(Expression),
    PrintStatement(Expression),
    LetStatement(Token, Expression), // name, initializer
    IfStatement(Expression, Box<Statment>, Option<Box<Statment>>), // condition, then_branch, else_branch
    WhileStatement(Expression, Box<Statment>), // condition, body
    ForStatement(Option<Box<Statment>>, Option<Expression>, Option<Expression>, Box<Statment>), // init, condition, increment, body
    BlockStatement(Vec<Statment>),
}

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }
    pub fn parse(&mut self) -> Result<Vec<Statment>, Errors> {
        let mut statements: Vec<Statment> = Vec::new();
        while self.tokens.peek().is_some_and(|t| t.token_type != TokenType::EOF) {
            statements.push(self.statement()?);
        }
        return Ok(statements);
    }

    fn statement(&mut self) -> Result<Statment, Errors> {
        if self.match_token(TokenType::PRINT) {
            let exp = self.expression()?;
            self.consume(TokenType::SEMICOLON, "Expect ';' after print statement")?;
            return Ok(Statment::PrintStatement(exp));
        }
        if self.match_token(TokenType::LET) {
            let name = self.consume(TokenType::IDENTIFIER, "Expect variable name")?;
            let mut initializer = None;
            if self.match_token(TokenType::EQUAL) {
                initializer = Some(self.expression()?);
            }
            self.consume(TokenType::SEMICOLON, "Expect ';' after variable declaration")?;
            return Ok(Statment::LetStatement(name, initializer.unwrap_or(Expression::Literal(Token::new(
                TokenType::NIL, "nil".to_string(), None, 1,
            )))));
        }
        if self.match_token(TokenType::IF) {
            self.consume(TokenType::LeftPara, "Expect '(' after 'if'")?;
            let condition = self.expression()?;
            self.consume(TokenType::RightPara, "Expect ')' after if condition")?;
            let then_branch = Box::new(self.statement()?);
            let else_branch = if self.match_token(TokenType::ELSE) {
                Some(Box::new(self.statement()?))
            } else {
                None
            };
            return Ok(Statment::IfStatement(condition, then_branch, else_branch));
        }
        if self.match_token(TokenType::WHILE) {
            self.consume(TokenType::LeftPara, "Expect '(' after 'while'")?;
            let condition = self.expression()?;
            self.consume(TokenType::RightPara, "Expect ')' after while condition")?;
            let body = Box::new(self.statement()?);
            return Ok(Statment::WhileStatement(condition, body));
        }
        if self.match_token(TokenType::FOR) {
            self.consume(TokenType::LeftPara, "Expect '(' after 'for'")?;
            let init = if self.match_token(TokenType::SEMICOLON) {
                None
            } else if self.match_token(TokenType::LET) {
                Some(Box::new(self.statement()?))
            } else {
                let exp = self.expression()?;
                self.consume(TokenType::SEMICOLON, "Expect ';' after for init")?;
                Some(Box::new(Statment::ExpressionStatement(exp)))
            };
            let condition = if self.check(TokenType::SEMICOLON) {
                None
            } else {
                Some(self.expression()?)
            };
            self.consume(TokenType::SEMICOLON, "Expect ';' after for condition")?;
            let increment = if self.check(TokenType::RightPara) {
                None
            } else {
                Some(self.expression()?)
            };
            self.consume(TokenType::RightPara, "Expect ')' after for clauses")?;
            let body = Box::new(self.statement()?);
            return Ok(Statment::ForStatement(init, condition, increment, body));
        }
        if self.match_token(TokenType::LeftBrace) {
            return Ok(Statment::BlockStatement(self.block()?));
        }
        let exp = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression")?;
        return Ok(Statment::ExpressionStatement(exp));
    }

    fn block(&mut self) -> Result<Vec<Statment>, Errors> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block")?;
        Ok(statements)
    }

    fn check(&mut self, token_type: TokenType) -> bool {
        self.tokens.peek().is_some_and(|t| t.token_type == token_type)
    }

    fn is_at_end(&mut self) -> bool {
        self.tokens.peek().is_some_and(|t| t.token_type == TokenType::EOF)
    }

    fn consume(&mut self, token_type: TokenType, _message: &str) -> Result<Token, Errors> {
        if self.check(token_type) {
            return Ok(self.tokens.next().unwrap());
        }
        Err(Errors::NonPrimaryToken)
    }

    fn expression(&mut self) -> Result<Expression, Errors> {
        return Ok(self.assignment()?);
    }

    fn assignment(&mut self) -> Result<Expression, Errors> {
        let exp = self.equality()?;

        if self.match_token(TokenType::EQUAL) {
            let value = self.assignment()?;
            if let Expression::Literal(token) = &exp {
                if token.token_type == TokenType::IDENTIFIER {
                    return Ok(Expression::Assign(token.text.clone(), Box::new(value)));
                }
            }
            return Err(Errors::NonPrimaryToken);
        }
        Ok(exp)
    }

    fn equality(&mut self) -> Result<Expression, Errors> {
        let mut exp: Expression = self.comp()?;

        while let Some(token) = self.tokens.next_if(|token| {
            [TokenType::BangEqual, TokenType::EqualEqual].contains(&token.token_type)
        }) {
            let expr_right = self.comp()?;
            exp = Expression::Binary(Box::new(exp), token, Box::new(expr_right))
        }
        return Ok(exp);
    }

    fn comp(&mut self) -> Result<Expression, Errors> {
        let mut exp: Expression = self.term()?;

        while let Some(token) = self.tokens.next_if(|token| {
            [
                TokenType::GreaterEqual,
                TokenType::GREATER,
                TokenType::LESS,
                TokenType::LessEqual,
            ]
            .contains(&token.token_type)
        }) {
            let expr_right = self.term()?;
            exp = Expression::Binary(Box::new(exp), token, Box::new(expr_right))
        }
        return Ok(exp);
    }

    fn term(&mut self) -> Result<Expression, Errors> {
        let mut exp: Expression = self.factor()?;

        while let Some(token) = self
            .tokens
            .next_if(|token| [TokenType::MINUS, TokenType::PLUS].contains(&token.token_type))
        {
            let expr_right = self.factor()?;
            exp = Expression::Binary(Box::new(exp), token, Box::new(expr_right))
        }
        return Ok(exp);
    }

    fn factor(&mut self) -> Result<Expression, Errors> {
        let mut exp: Expression = self.unary()?;

        while let Some(token) = self
            .tokens
            .next_if(|token| [TokenType::SLASH, TokenType::STAR].contains(&token.token_type))
        {
            let expr_right = self.unary()?;
            exp = Expression::Binary(Box::new(exp), token, Box::new(expr_right))
        }
        return Ok(exp);
    }

    fn unary(&mut self) -> Result<Expression, Errors> {
        if let Some(operator) = self
            .tokens
            .next_if(|token| [TokenType::BANG, TokenType::MINUS].contains(&token.token_type))
        {
            let exp_right = self.unary()?;
            return Ok(Expression::Unary(operator, Box::new(exp_right)));
        }

        return Ok(self.primary()?);
    }

    fn primary(&mut self) -> Result<Expression, Errors> {
        if let Some(token) = self.tokens.next() {
            match token.token_type {
                TokenType::TRUE | TokenType::FALSE | TokenType::NIL => {
                    return Ok(Expression::Literal(token));
                }

                TokenType::NUMBER | TokenType::STRING => return Ok(Expression::Literal(token)),

                TokenType::IDENTIFIER => return Ok(Expression::Literal(token)),

                TokenType::LeftPara => {
                    let exp = self.expression()?;
                    if let Some(_token) = self.tokens.next_if(|t| t.token_type == TokenType::RightPara) {
                        return Ok(Expression::Grouping(Box::new(exp)));
                    } else {
                        return Err(Errors::UnterminatedGroup);
                    }
                }
                _ => return Err(Errors::NonPrimaryToken),
            }
        }
        return Err(Errors::NonPrimaryToken);
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        self.tokens.next_if(|t| t.token_type == token_type).is_some()
    }
}

    // fn check(&self, token_type: TokenType) -> bool {
    //     self.tokens
    //         .peek()
    //         .is_some_and(|src_token| src_token.token_type == token_type)
    // }

// struct Binary {
//     left: Expression,
//     operator: BinaryOperator,
//     right: Expression,
// }
// enum BinaryOperator {}
// struct Uniary {}
// struct Grouping {}
// struct Literal {}
