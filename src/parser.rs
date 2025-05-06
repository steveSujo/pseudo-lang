use std::{fmt::Display, iter::Peekable, vec::IntoIter};

use crate::{
    error_handler::Errors,
    lexer::Lexer,
    tokens::{self, LiteralType, Token, TokenType},
};

pub enum DataTypes {
    Number(f32),
    Bool(bool),
}

impl Display for DataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataTypes::Number(num) => write!(f, "float_datatype: {num}"),
            DataTypes::Bool(bool) => write!(f, "bool_datatype: {bool}"),
        }
    }
}

impl From<DataTypes> for f32 {
    fn from(value: DataTypes) -> Self {
        match value {
            DataTypes::Number(num) => num,
            DataTypes::Bool(true) => 1.0,
            DataTypes::Bool(false) => 0.0,
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
            DataTypes::Bool(bool) => bool,
            DataTypes::Number(0.0) => false,
            DataTypes::Number(_) => true,
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
            Some(_) => unimplemented!(),
            None => unimplemented!(),
        }
    }
}

#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(Token),
    Grouping(Box<Expression>),
}

// let var:Option

// visitor pattern for better maintablity
pub trait ExpressionVistor<T> {
    fn vist_expr(&self, e: Expression) -> T;
    // fn vist_unary();
    // fn vist_literal();
    // fn vist_grouping();
}

//operations as classes
pub struct Interpret;

impl ExpressionVistor<DataTypes> for Interpret {
    //TODO: error handeling
    fn vist_expr(&self, e: Expression) -> DataTypes {
        match e {
            Expression::Binary(exp_a, operator, exp_b) => match operator.token_type {
                TokenType::MINUS => {
                    bin_expr::<f32>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| a - b)
                }

                TokenType::PLUS => {
                    bin_expr::<f32>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| a + b)
                }
                TokenType::SLASH => {
                    bin_expr::<f32>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| a / b)
                }
                TokenType::STAR => {
                    bin_expr::<f32>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| a * b)
                }
                TokenType::EqualEqual => {
                    bin_expr::<bool>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| {
                        a == b
                    })
                }
                TokenType::BangEqual => {
                    bin_expr::<bool>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| {
                        a != b
                    })
                }
                TokenType::LessEqual => {
                    bin_expr::<bool>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| {
                        a <= b
                    })
                }
                TokenType::LESS => {
                    bin_expr::<bool>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| a < b)
                }
                TokenType::GreaterEqual => {
                    bin_expr::<bool>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| {
                        a >= b
                    })
                }
                TokenType::GREATER => {
                    bin_expr::<bool>(self.vist_expr(*exp_a), self.vist_expr(*exp_b), |a, b| a > b)
                }

                _ => unimplemented!(),
            },
            Expression::Unary(operator, exp) => match operator.token_type {
                TokenType::MINUS => { -f32::from(self.vist_expr(*exp)) }.into(),
                TokenType::BANG => { !bool::from(self.vist_expr(*exp)) }.into(),
                _ => unimplemented!(),
            },
            Expression::Literal(token) => match token.token_type {
                TokenType::NUMBER => token.literal.into(),
                _ => unimplemented!(),
            },

            Expression::Grouping(exp) => self.vist_expr(*exp),
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
impl ExpressionVistor<String> for PrettyPrint {
    fn vist_expr(&self, e: Expression) -> String {
        match e {
            Expression::Binary(exp1, op, exp2) => {
                format!(
                    "({} {} {})",
                    op.text,
                    self.vist_expr(*exp1),
                    self.vist_expr(*exp2)
                )
            }
            Expression::Unary(op, exp) => {
                format!("({} {})", op.text, self.vist_expr(*exp))
            }
            Expression::Literal(token) => format!(
                "{}",
                token
                    .literal
                    .expect("INTERNAL ERROR: no literal is None")
                    .print()
            ),
            Expression::Grouping(exp) => format!("(group {})", self.vist_expr(*exp)),
        }
    }
}

enum Statment {
    ExpresisonStatment(Expression),
    PrintStatment(Expression),
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
    pub fn parse(&mut self) -> Vec<Statment> {
        let mut statemets: Vec<Statment> = Vec::new();
        while let Some(token) = self.tokens.next() {
            statemets.push();
        }
        return statemets;
    }
    fn expression(&mut self) -> Result<Expression, Errors> {
        return Ok(self.eqality()?);
    }

    fn eqality(&mut self) -> Result<Expression, Errors> {
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

                TokenType::LeftPara => {
                    let exp = self.expression()?;
                    if let Some(token) = self.tokens.next() {
                        return Ok(Expression::Grouping(Box::new(exp)));
                    } else {
                        return Err(Errors::UntermitedGroup);
                    }
                }
                _ => return Err(Errors::NonPrimaryToken),
            }
        }
        return Err(Errors::NonPrimaryToken);
    }

    // fn check(&self, token_type: TokenType) -> bool {
    //     self.tokens
    //         .peek()
    //         .is_some_and(|src_token| src_token.token_type == token_type)
    // }
}

// struct Binary {
//     left: Expression,
//     operator: BinaryOperator,
//     right: Expression,
// }
// enum BinaryOperator {}
// struct Uniary {}
// struct Grouping {}
// struct Literal {}
