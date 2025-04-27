use std::{iter::Peekable, vec::IntoIter};

use crate::{
    error_handler::Errors,
    lexer::Lexer,
    tokens::{self, LiteralType, Token, TokenType},
};

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
struct Interpret;

impl ExpressionVistor<f32> for Interpret {
    fn vist_expr(&self, e: Expression) -> f32 {
        match e {
            Expression::Binary(exp_a, operator, exp_b) => match operator.token_type {
                TokenType::MINUS => self.vist_expr(*exp_a) - self.vist_expr(*exp_b),
                TokenType::PLUS => self.vist_expr(*exp_a) + self.vist_expr(*exp_b),
                TokenType::SLASH => self.vist_expr(*exp_a) / self.vist_expr(*exp_b),
                TokenType::STAR => self.vist_expr(*exp_a) * self.vist_expr(*exp_b),
                _ => todo!(),
            },
            Expression::Unary(operator, exp) => match operator.token_type {
                TokenType::MINUS => -self.vist_expr(*exp),
                _ => todo!(),
            },
            Expression::Literal(token) => match token.literal {
                Some(LiteralType::Number(num)) => num,
                _ => todo!(),
            },

            Expression::Grouping(exp) => self.vist_expr(*exp),
        }
    }
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

pub struct Parser {
    tokens: Peekable<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens: tokens.into_iter().peekable(),
        }
    }

    pub fn expression(&mut self) -> Result<Expression, Errors> {
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
