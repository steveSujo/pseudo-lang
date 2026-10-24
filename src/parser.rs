use std::{fmt::Display, iter::Peekable, vec::IntoIter, rc::Rc, cell::RefCell};

use crate::{
    error_handler::Errors,
    tokens::{LiteralType, Token, TokenType},
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum DataTypes {
    Number(f32),
    Bool(bool),
    Str(String),
    Array(Vec<DataTypes>),
    Object(std::collections::HashMap<String, DataTypes>),
    Function {
        params: Vec<String>,
        body: Box<Statment>,
        capture: Rc<RefCell<Environment>>,
    },
    Result(Result<Box<DataTypes>, Box<DataTypes>>),
}
impl Display for DataTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataTypes::Number(num) => write!(f, "{num}"),
            DataTypes::Bool(b) => write!(f, "{b}"),
            DataTypes::Str(s) => write!(f, "\"{s}\""),
            DataTypes::Array(arr) => write!(f, "[{}]", arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
            DataTypes::Object(obj) => write!(f, "{{{}}}", obj.iter().map(|(k,v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ")),
            DataTypes::Function { params, .. } => write!(f, "<fn {}>", params.join(", ")),
            DataTypes::Result(r) => match r {
                Ok(v) => write!(f, "Ok({v})"),
                Err(e) => write!(f, "Err({e})"),
            },
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
            DataTypes::Array(_) => 0.0,
            DataTypes::Object(_) => 0.0,
            DataTypes::Function { .. } => 0.0,
            DataTypes::Result(_) => 0.0,
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
            DataTypes::Array(arr) => !arr.is_empty(),
            DataTypes::Object(obj) => !obj.is_empty(),
            DataTypes::Function { .. } => true,
            DataTypes::Result(r) => r.is_ok(),
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
    Assign(String, Box<Expression>),
    Call(String, Vec<Expression>), // function name, arguments
    Array(Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Object(Vec<(String, Expression)>),
    Get(Box<Expression>, String),
    Function(Vec<String>, Box<Statment>),
    Return(Option<Box<Expression>>),
}

// let var:Option

// visitor pattern for better maintainability
pub trait ExpressionVisitor<T> {
    fn visit_binary(&mut self, left: &Expression, operator: &Token, right: &Expression) -> T;
    fn visit_unary(&mut self, operator: &Token, right: &Expression) -> T;
    fn visit_literal(&mut self, token: &Token) -> T;
    fn visit_grouping(&mut self, expression: &Expression) -> T;
    fn visit_assign(&mut self, name: &str, value: &Expression) -> T;
    fn visit_call(&mut self, name: &str, args: &[Expression]) -> T;
    fn visit_array(&mut self, elements: &[Expression]) -> T;
    fn visit_index(&mut self, object: &Expression, index: &Expression) -> T;
    fn visit_object(&mut self, fields: &[(String, Expression)]) -> T;
    fn visit_get(&mut self, object: &Expression, name: &str) -> T;
    fn visit_function(&mut self, params: &[String], body: &Statment) -> T;
    fn visit_return(&mut self, value: &Option<Box<Expression>>) -> T;
}
//operations as classes

#[derive(Debug, Clone)]
pub struct Environment {
    vars: HashMap<String, DataTypes>,
    parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(vars: HashMap<String, DataTypes>) -> Self {
        Environment { vars, parent: None }
    }

    pub fn with_parent(vars: HashMap<String, DataTypes>, parent: Rc<RefCell<Environment>>) -> Self {
        Environment { vars, parent: Some(parent) }
    }

    pub fn get(&self, name: &str) -> Option<DataTypes> {
        if let Some(v) = self.vars.get(name) {
            Some(v.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set(&mut self, name: String, val: DataTypes) {
        self.vars.insert(name, val);
    }

    pub fn define(&mut self, name: String, val: DataTypes) {
        self.vars.insert(name, val);
    }
}

pub struct Interpret {
    pub environment: Rc<RefCell<Environment>>,
}

impl Default for Interpret {
    fn default() -> Self {
        Interpret {
            environment: Rc::new(RefCell::new(Environment::new(HashMap::new()))),
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
                    .borrow()
                    .get(&token.text)
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
        self.environment.borrow_mut().set(name.to_string(), val.clone());
        val
    }

    fn visit_call(&mut self, name: &str, args: &[Expression]) -> DataTypes {
        // Evaluate all arguments first
        let arg_vals: Vec<DataTypes> = args.iter().map(|a| self.visit_expr(a)).collect();

        // Check for user-defined function in environment
        let func_data = self.environment.borrow().get(name);
        if let Some(DataTypes::Function { params, body, capture }) = func_data {
            // Create new environment with captured vars
            let mut new_env = Environment::with_parent(HashMap::new(), capture.clone());
            // Bind parameters
            for (i, param) in params.iter().enumerate() {
                if i < arg_vals.len() {
                    new_env.define(param.clone(), arg_vals[i].clone());
                }
            }
            // Save current environment
            let old_env = self.environment.clone();
            self.environment = Rc::new(RefCell::new(new_env));
            // Execute function body
            let result = self.execute(&body);
            // Restore environment
            self.environment = old_env;
            return result.unwrap_or(DataTypes::Number(0.0));
        }

        // Built-in functions
        match name {
            "readFile" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("readFile expects 1 argument".to_string());
                }
                let path = match &arg_vals[0] {
                    DataTypes::Str(s) => s.clone(),
                    _ => return DataTypes::Str("readFile expects string argument".to_string()),
                };
                match crate::fs::read_file(&path) {
                    Ok(content) => DataTypes::Str(content),
                    Err(e) => DataTypes::Str(format!("Error: {}", e)),
                }
            }
            "writeFile" => {
                if arg_vals.len() != 2 {
                    return DataTypes::Str("writeFile expects 2 arguments".to_string());
                }
                let path = match &arg_vals[0] {
                    DataTypes::Str(s) => s.clone(),
                    _ => return DataTypes::Str("writeFile expects string path".to_string()),
                };
                let content = match &arg_vals[1] {
                    DataTypes::Str(s) => s.clone(),
                    DataTypes::Number(n) => n.to_string(),
                    DataTypes::Bool(b) => b.to_string(),
                    DataTypes::Array(arr) => format!("[{}]", arr.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
                    DataTypes::Object(obj) => format!("{{{}}}", obj.iter().map(|(k,v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ")),
                    DataTypes::Function { params, .. } => format!("<fn {}>", params.join(", ")),
                    DataTypes::Result(r) => match r {
                        Ok(v) => format!("Ok({})", v),
                        Err(e) => format!("Err({})", e),
                    },
                    _ => "unknown".to_string(),
                };
                match crate::fs::write_file(&path, &content) {
                    Ok(_) => DataTypes::Number(0.0), // nil equivalent
                    Err(e) => DataTypes::Str(format!("Error: {}", e)),
                }
            }
            "listDir" => {
                let path = if arg_vals.is_empty() {
                    ".".to_string()
                } else {
                    match &arg_vals[0] {
                        DataTypes::Str(s) => s.clone(),
                        _ => return DataTypes::Str("listDir expects string argument".to_string()),
                    }
                };
                match crate::fs::list_dir(&path) {
                    Ok(entries) => {
                        let joined = entries.join("\n");
                        DataTypes::Str(joined)
                    }
                    Err(e) => DataTypes::Str(format!("Error: {}", e)),
                }
            }
            "exists" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("exists expects 1 argument".to_string());
                }
                let path = match &arg_vals[0] {
                    DataTypes::Str(s) => s.clone(),
                    _ => return DataTypes::Str("exists expects string argument".to_string()),
                };
                DataTypes::Bool(crate::fs::exists(&path))
            }
            "isFile" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("isFile expects 1 argument".to_string());
                }
                let path = match &arg_vals[0] {
                    DataTypes::Str(s) => s.clone(),
                    _ => return DataTypes::Str("isFile expects string argument".to_string()),
                };
                DataTypes::Bool(crate::fs::is_file(&path))
            }
            "isDir" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("isDir expects 1 argument".to_string());
                }
                let path = match &arg_vals[0] {
                    DataTypes::Str(s) => s.clone(),
                    _ => return DataTypes::Str("isDir expects string argument".to_string()),
                };
                DataTypes::Bool(crate::fs::is_dir(&path))
            }
            "mkdir" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("mkdir expects 1 argument".to_string());
                }
                let path = match &arg_vals[0] {
                    DataTypes::Str(s) => s.clone(),
                    _ => return DataTypes::Str("mkdir expects string argument".to_string()),
                };
                match crate::fs::mkdir(&path) {
                    Ok(_) => DataTypes::Number(0.0),
                    Err(e) => DataTypes::Str(format!("Error: {}", e)),
                }
            }
            "len" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("len expects 1 argument".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Array(arr) => DataTypes::Number(arr.len() as f32),
                    DataTypes::Str(s) => DataTypes::Number(s.len() as f32),
                    DataTypes::Object(obj) => DataTypes::Number(obj.len() as f32),
                    _ => DataTypes::Str("len expects array, string, or object".to_string()),
                }
            }
            "push" => {
                if arg_vals.len() != 2 {
                    return DataTypes::Str("push expects 2 arguments".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Array(arr) => {
                        let mut new_arr = arr.clone();
                        new_arr.push(arg_vals[1].clone());
                        DataTypes::Array(new_arr)
                    }
                    _ => DataTypes::Str("push expects array as first argument".to_string()),
                }
            }
            "pop" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("pop expects 1 argument".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Array(arr) => {
                        let mut new_arr = arr.clone();
                        new_arr.pop();
                        DataTypes::Array(new_arr)
                    }
                    _ => DataTypes::Str("pop expects array".to_string()),
                }
            }
            "slice" => {
                if arg_vals.len() < 2 || arg_vals.len() > 3 {
                    return DataTypes::Str("slice expects 2 or 3 arguments".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Array(arr) => {
                        let start_num: f32 = arg_vals[1].clone().into();
                        let start = start_num as i32;
                        let end = if arg_vals.len() == 3 {
                            let end_num: f32 = arg_vals[2].clone().into();
                            end_num as i32
                        } else {
                            arr.len() as i32
                        };
                        let start = if start < 0 { (arr.len() as i32 + start).max(0) as usize } else { start as usize };
                        let end = if end < 0 { (arr.len() as i32 + end).max(0) as usize } else { end as usize };
                        let end = end.min(arr.len());
                        if start <= end && start < arr.len() {
                            DataTypes::Array(arr[start..end].to_vec())
                        } else {
                            DataTypes::Array(vec![])
                        }
                    }
                    _ => DataTypes::Str("slice expects array".to_string()),
                }
            }
            "keys" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("keys expects 1 argument".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Object(obj) => {
                        let keys: Vec<DataTypes> = obj.keys().map(|k| DataTypes::Str(k.clone())).collect();
                        DataTypes::Array(keys)
                    }
                    _ => DataTypes::Str("keys expects object".to_string()),
                }
            }
            "values" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("values expects 1 argument".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Object(obj) => {
                        let vals: Vec<DataTypes> = obj.values().cloned().collect();
                        DataTypes::Array(vals)
                    }
                    _ => DataTypes::Str("values expects object".to_string()),
                }
            }
            "has" => {
                if arg_vals.len() != 2 {
                    return DataTypes::Str("has expects 2 arguments".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Object(obj) => {
                        let key = match &arg_vals[1] {
                            DataTypes::Str(s) => s.clone(),
                            _ => return DataTypes::Str("has expects string key".to_string()),
                        };
                        DataTypes::Bool(obj.contains_key(&key))
                    }
                    _ => DataTypes::Str("has expects object".to_string()),
                }
            }
            "ok" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("ok expects 1 argument".to_string());
                }
                DataTypes::Result(Ok(Box::new(arg_vals[0].clone())))
            }
            "err" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("err expects 1 argument".to_string());
                }
                DataTypes::Result(Err(Box::new(arg_vals[0].clone())))
            }
            "isError" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("isError expects 1 argument".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Result(r) => DataTypes::Bool(r.is_err()),
                    _ => DataTypes::Bool(false),
                }
            }
            "isOk" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("isOk expects 1 argument".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Result(r) => DataTypes::Bool(r.is_ok()),
                    _ => DataTypes::Bool(false),
                }
            }
            "unwrap" => {
                if arg_vals.len() != 1 {
                    return DataTypes::Str("unwrap expects 1 argument".to_string());
                }
                match &arg_vals[0] {
                    DataTypes::Result(r) => match r {
                        Ok(v) => *v.clone(),
                        Err(e) => DataTypes::Str(format!("unwrap on Err: {}", e)),
                    },
                    _ => DataTypes::Str("unwrap expects Result".to_string()),
                }
            }
            "expect" => {
                if arg_vals.len() != 2 {
                    return DataTypes::Str("expect expects 2 arguments".to_string());
                }
                let msg = match &arg_vals[1] {
                    DataTypes::Str(s) => s.clone(),
                    _ => return DataTypes::Str("expect expects string message".to_string()),
                };
                match &arg_vals[0] {
                    DataTypes::Result(r) => match r {
                        Ok(v) => *v.clone(),
                        Err(e) => DataTypes::Str(format!("{}: {}", msg, e)),
                    },
                    _ => DataTypes::Str("expect expects Result".to_string()),
                }
            }
            _ => DataTypes::Str(format!("Unknown function: {}", name)),
        }
    }
    fn visit_array(&mut self, elements: &[Expression]) -> DataTypes {
        DataTypes::Array(elements.iter().map(|e| self.visit_expr(e)).collect())
    }

    fn visit_index(&mut self, object: &Expression, index: &Expression) -> DataTypes {
        let obj = self.visit_expr(object);
        let idx = self.visit_expr(index);
        match (&obj, &idx) {
            (DataTypes::Array(arr), DataTypes::Number(i)) => {
                let i = *i as usize;
                if i < arr.len() {
                    arr[i].clone()
                } else {
                    DataTypes::Str(format!("Index out of bounds: {}", i))
                }
            }
            (DataTypes::Object(map), DataTypes::Str(key)) => {
                map.get(key).cloned().unwrap_or(DataTypes::Number(0.0))
            }
            _ => DataTypes::Str("Invalid index operation".to_string()),
        }
    }

    fn visit_object(&mut self, fields: &[(String, Expression)]) -> DataTypes {
        let mut map = std::collections::HashMap::new();
        for (key, expr) in fields {
            map.insert(key.clone(), self.visit_expr(expr));
        }
        DataTypes::Object(map)
    }

    fn visit_get(&mut self, object: &Expression, name: &str) -> DataTypes {
        let obj = self.visit_expr(object);
        match obj {
            DataTypes::Object(map) => map.get(name).cloned().unwrap_or(DataTypes::Number(0.0)),
            _ => DataTypes::Str("Not an object".to_string()),
        }
    }

    fn visit_function(&mut self, params: &[String], body: &Statment) -> DataTypes {
        let parent_env = self.environment.clone();
        let capture = Rc::new(RefCell::new(Environment::with_parent(HashMap::new(), parent_env)));
        DataTypes::Function {
            params: params.to_vec(),
            body: Box::new(body.clone()),
            capture,
        }
    }

    fn visit_return(&mut self, value: &Option<Box<Expression>>) -> DataTypes {
        match value {
            Some(expr) => self.visit_expr(expr),
            None => DataTypes::Number(0.0),
        }
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
            Expression::Call(name, args) => self.visit_call(name, args),
            Expression::Array(elements) => self.visit_array(elements),
            Expression::Index(object, index) => self.visit_index(object, index),
            Expression::Object(fields) => self.visit_object(fields),
            Expression::Get(object, name) => self.visit_get(object, name),
            Expression::Function(params, body) => self.visit_function(params, body),
            Expression::Return(value) => self.visit_return(value),
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
                self.environment.borrow_mut().define(name.text.clone(), val.clone());
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
                let mut last_val = None;
                for stmt in statements {
                    match self.execute(stmt) {
                        Some(v) => {
                            // Check if it's a return from inside
                            // For now, just capture the last expression value
                            last_val = Some(v);
                        }
                        None => {}
                    }
                }
                last_val
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

    fn visit_call(&mut self, name: &str, args: &[Expression]) -> String {
        let arg_strs: Vec<String> = args.iter().map(|a| self.visit_expr(a.clone())).collect();
        format!("(call {} {})", name, arg_strs.join(" "))
    }

    fn visit_array(&mut self, elements: &[Expression]) -> String {
        let elem_strs: Vec<String> = elements.iter().map(|e| self.visit_expr(e.clone())).collect();
        format!("[{}]", elem_strs.join(", "))
    }

    fn visit_index(&mut self, object: &Expression, index: &Expression) -> String {
        format!("(index {} {})", self.visit_expr(object.clone()), self.visit_expr(index.clone()))
    }

    fn visit_object(&mut self, fields: &[(String, Expression)]) -> String {
        let field_strs: Vec<String> = fields.iter().map(|(k, v)| format!("{}: {}", k, self.visit_expr(v.clone()))).collect();
        format!("{{{}}}", field_strs.join(", "))
    }

    fn visit_get(&mut self, object: &Expression, name: &str) -> String {
        format!("(get {} {})", self.visit_expr(object.clone()), name)
    }

    fn visit_function(&mut self, params: &[String], body: &Statment) -> String {
        format!("(fn {} ...)", params.join(", "))
    }

    fn visit_return(&mut self, value: &Option<Box<Expression>>) -> String {
        match value {
            Some(expr) => format!("(return {})", self.visit_expr(*expr.clone())),
            None => "(return)".to_string(),
        }
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
            Expression::Call(name, args) => self.visit_call(&name, &args),
            Expression::Array(elements) => self.visit_array(&elements),
            Expression::Index(object, index) => self.visit_index(&object, &index),
            Expression::Object(fields) => self.visit_object(&fields),
            Expression::Get(object, name) => self.visit_get(&object, &name),
            Expression::Function(params, body) => self.visit_function(&params, &*body),
            Expression::Return(value) => self.visit_return(&value),
        }
    }
}
#[derive(Debug, Clone)]
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
        if self.match_token(TokenType::RETURN) {
            eprintln!("DEBUG: matched RETURN token");
            let value = if self.check(TokenType::SEMICOLON) || self.check(TokenType::RightBrace) || self.is_at_end() {
                None
            } else {
                Some(Box::new(self.expression()?))
            };
            self.consume(TokenType::SEMICOLON, "Expect ';' after return statement")?;
            return Ok(Statment::ExpressionStatement(Expression::Return(value)));
        }
        if self.match_token(TokenType::LeftBrace) {
            return Ok(Statment::BlockStatement(self.block()?));
        }
        if self.match_token(TokenType::FN) {
            // Function declaration: fn name(params) { body } or fn name(params) => expr
            let name_token = self.consume(TokenType::IDENTIFIER, "Expect function name")?;
            let func_expr = self.function_literal()?;
            // Store function in environment with name
            let assign_expr = Expression::Assign(name_token.text, Box::new(func_expr));
            self.consume(TokenType::SEMICOLON, "Expect ';' after function declaration")?;
            return Ok(Statment::ExpressionStatement(assign_expr));
        }
        // Debug: print unknown token
        if let Some(tok) = self.tokens.peek() {
            eprintln!("DEBUG statement: unknown token {:?}", tok.token_type);
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

                TokenType::LeftBracket => {
                    return self.array_literal();
                }

                TokenType::LeftBrace => {
                    return self.object_literal();
                }

                TokenType::FN => {
                    return self.function_literal();
                }

                TokenType::RETURN => {
                    let value = if self.check(TokenType::SEMICOLON) || self.check(TokenType::RightBrace) || self.is_at_end() {
                        None
                    } else {
                        Some(Box::new(self.expression()?))
                    };
                    return Ok(Expression::Return(value));
                }

                TokenType::IDENTIFIER => {
                    // Check if this is a function call (identifier followed by '(')
                    let name = token.text.clone();
                    let mut expr = Expression::Literal(token);
                    if self.check(TokenType::LeftPara) {
                        self.consume(TokenType::LeftPara, "Expect '(' after function name")?;
                        let mut args = Vec::new();
                        if !self.check(TokenType::RightPara) {
                            loop {
                                args.push(self.expression()?);
                                if !self.match_token(TokenType::COMMA) {
                                    break;
                                }
                            }
                        }
                        self.consume(TokenType::RightPara, "Expect ')' after arguments")?;
                        expr = Expression::Call(name, args);
                    }
                    // Handle index and get expressions
                    loop {
                        if self.match_token(TokenType::LeftBracket) {
                            let index = self.expression()?;
                            self.consume(TokenType::RightBracket, "Expect ']' after index")?;
                            expr = Expression::Index(Box::new(expr), Box::new(index));
                        } else if self.match_token(TokenType::DOT) {
                            let name_token = self.consume(TokenType::IDENTIFIER, "Expect property name after '.'")?;
                            expr = Expression::Get(Box::new(expr), name_token.text);
                        } else {
                            break;
                        }
                    }
                    return Ok(expr);
                }

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

    fn array_literal(&mut self) -> Result<Expression, Errors> {
        let mut elements = Vec::new();
        if !self.check(TokenType::RightBracket) {
            loop {
                elements.push(self.expression()?);
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightBracket, "Expect ']' after array elements")?;
        Ok(Expression::Array(elements))
    }

    fn object_literal(&mut self) -> Result<Expression, Errors> {
        let mut fields = Vec::new();
        if !self.check(TokenType::RightBrace) {
            loop {
                let key_token = self.consume(TokenType::IDENTIFIER, "Expect property name")?;
                self.consume(TokenType::COLON, "Expect ':' after property name")?;
                let value = self.expression()?;
                fields.push((key_token.text, value));
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after object fields")?;
        Ok(Expression::Object(fields))
    }

    fn function_literal(&mut self) -> Result<Expression, Errors> {
        self.consume(TokenType::LeftPara, "Expect '(' after 'fn'")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightPara) {
            loop {
                let param = self.consume(TokenType::IDENTIFIER, "Expect parameter name")?;
                params.push(param.text);
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightPara, "Expect ')' after parameters")?;

        // Check for expression body (=>) or block body
        if self.match_token(TokenType::ARROW) {
            // Expression body
            let expr = self.expression()?;
            let body = Statment::ExpressionStatement(expr);
            Ok(Expression::Function(params, Box::new(body)))
        } else {
            // Block body
            self.consume(TokenType::LeftBrace, "Expect '{' before function body")?;
            let body = Statment::BlockStatement(self.block()?);
            Ok(Expression::Function(params, Box::new(body)))
        }
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
