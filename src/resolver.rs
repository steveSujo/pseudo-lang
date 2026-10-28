// Resolver for static scope analysis (Chapter 11, Crafting Interpreters)

use crate::parser::{Expression, Statment};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeType {
    Function,
    Block,
    Class,
    Method,
}

#[derive(Debug, Clone)]
pub struct ResolverError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub struct ResolverResult {
    pub depths: HashMap<ExprId, usize>,
    pub errors: Vec<ResolverError>,
}

pub struct Resolver {
    scopes: Vec<HashMap<String, ScopeInfo>>,
    depths: HashMap<ExprId, usize>,
    errors: Vec<ResolverError>,
    current_function: Option<ScopeType>,
    current_class: Option<ScopeType>,
    next_expr_id: usize,
}

#[derive(Debug, Clone)]
struct ScopeInfo {
    initialized: bool,
    depth: usize,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            scopes: Vec::new(),
            depths: HashMap::new(),
            errors: Vec::new(),
            current_function: None,
            current_class: None,
            next_expr_id: 0,
        }
    }

    pub fn resolve(mut self, statements: &[Statment]) -> ResolverResult {
        // Add global scope
        self.begin_scope();
        
        for stmt in statements {
            self.resolve_statement(stmt);
        }
        
        self.end_scope();
        
        ResolverResult {
            depths: self.depths,
            errors: self.errors,
        }
    }

    fn next_id(&mut self) -> ExprId {
        let id = ExprId(self.next_expr_id);
        self.next_expr_id += 1;
        id
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str, line: usize, column: usize) {
        let depth = self.scopes.len() - 1;
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(name) {
                self.errors.push(ResolverError {
                    message: format!("Variable '{}' already declared in this scope", name),
                    line,
                    column,
                });
            } else {
                scope.insert(name.to_string(), ScopeInfo { initialized: false, depth });
            }
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            if let Some(info) = scope.get_mut(name) {
                info.initialized = true;
            }
        }
    }

    fn resolve_local(&mut self, expr_id: ExprId, name: &str, line: usize, column: usize) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                let depth = i;
                self.depths.insert(expr_id, depth);
                if let Some(info) = scope.get(name) {
                    if !info.initialized {
                        self.errors.push(ResolverError {
                            message: format!("Variable '{}' used before initialization", name),
                            line,
                            column,
                        });
                    }
                }
                return;
            }
        }
        // Not found in any scope - global or undefined
        if self.scopes.len() == 1 {
            // Global scope - allow but track
            self.depths.insert(expr_id, 0);
        } else {
            self.errors.push(ResolverError {
                message: format!("Undefined variable '{}'", name),
                line,
                column,
            });
        }
    }

    fn resolve_statement(&mut self, stmt: &Statment) {
        match stmt {
            Statment::ExpressionStatement(expr) => self.resolve_expression(expr),
            Statment::PrintStatement(expr) => self.resolve_expression(expr),
            Statment::LetStatement(token, initializer) => {
                self.declare(&token.text, token.span.start_line, token.span.start_col);
                self.resolve_expression(initializer);
                self.define(&token.text);
            }
            Statment::IfStatement(condition, then_branch, else_branch) => {
                self.resolve_expression(condition);
                self.resolve_statement(then_branch);
                if let Some(else_stmt) = else_branch {
                    self.resolve_statement(else_stmt);
                }
            }
            Statment::WhileStatement(condition, body) => {
                self.resolve_expression(condition);
                self.resolve_statement(body);
            }
            Statment::ForStatement(init, condition, increment, body) => {
                self.begin_scope();
                if let Some(init_stmt) = init {
                    self.resolve_statement(init_stmt);
                }
                if let Some(cond_expr) = condition {
                    self.resolve_expression(cond_expr);
                }
                if let Some(inc_expr) = increment {
                    self.resolve_expression(inc_expr);
                }
                self.resolve_statement(body);
                self.end_scope();
            }
            Statment::BlockStatement(statements) => {
                self.begin_scope();
                for stmt in statements {
                    self.resolve_statement(stmt);
                }
                self.end_scope();
            }
            Statment::ClassStatement(name_token, methods, _superclass_token) => {
                self.declare(&name_token.text, name_token.span.start_line, name_token.span.start_col);
                self.define(&name_token.text);
                
                self.begin_scope();
                // Declare 'this' in class scope
                self.declare("this", 0, 0);
                self.define("this");
                
                // Resolve each method in method scope
                for method in methods {
                    if let Statment::ExpressionStatement(Expression::Function(params, body)) = method {
                        self.resolve_function(params, body, ScopeType::Method);
                    }
                }
                self.end_scope();
            }
        }
    }

    fn resolve_expression(&mut self, expr: &Expression) {
        let expr_id = self.next_id();
        self.resolve_expression_inner(expr, expr_id);
    }

    fn resolve_expression_inner(&mut self, expr: &Expression, expr_id: ExprId) {
        match expr {
            Expression::Binary(left, _op, right) => {
                let left_id = self.next_id();
                let right_id = self.next_id();
                self.resolve_expression_inner(left, left_id);
                self.resolve_expression_inner(right, right_id);
            }
            Expression::Unary(_op, right) => {
                let right_id = self.next_id();
                self.resolve_expression_inner(right, right_id);
            }
            Expression::Literal(token) => {
                // Handle identifier literals (variable references)
                if token.token_type == crate::tokens::TokenType::IDENTIFIER {
                    self.resolve_local(expr_id, &token.text, token.span.start_line, token.span.start_col);
                }
            }
            Expression::Grouping(expr) => {
                let inner_id = self.next_id();
                self.resolve_expression_inner(expr, inner_id);
            }
            Expression::Assign(name, value) => {
                let value_id = self.next_id();
                self.resolve_expression_inner(value, value_id);
                self.resolve_local(expr_id, name, 0, 0);
            }
            Expression::Call(_name, args) => {
                let mut arg_ids = Vec::new();
                for _ in args {
                    arg_ids.push(self.next_id());
                }
                for (arg, arg_id) in args.iter().zip(arg_ids) {
                    self.resolve_expression_inner(arg, arg_id);
                }
            }
            Expression::Array(elements) => {
                let mut elem_ids = Vec::new();
                for _ in elements {
                    elem_ids.push(self.next_id());
                }
                for (elem, elem_id) in elements.iter().zip(elem_ids) {
                    self.resolve_expression_inner(elem, elem_id);
                }
            }
            Expression::Get(obj, _name) => {
                let obj_id = self.next_id();
                self.resolve_expression_inner(obj, obj_id);
            }
            Expression::Set(obj, _name, value) => {
                let obj_id = self.next_id();
                let value_id = self.next_id();
                self.resolve_expression_inner(obj, obj_id);
                self.resolve_expression_inner(value, value_id);
            }
            Expression::Function(params, body) => {
                self.resolve_function(params, body, ScopeType::Function);
            }
            Expression::Return(value) => {
                if let Some(val) = value {
                    let val_id = self.next_id();
                    self.resolve_expression_inner(val, val_id);
                }
            }
            Expression::This => {
                if self.current_class.is_none() {
                    self.errors.push(ResolverError {
                        message: "Cannot use 'this' outside of a class".to_string(),
                        line: 0,
                        column: 0,
                    });
                }
            }
            Expression::Super(_method) => {
                if self.current_class != Some(ScopeType::Class) {
                    self.errors.push(ResolverError {
                        message: "Cannot use 'super' outside of a class".to_string(),
                        line: 0,
                        column: 0,
                    });
                }
            }
            Expression::SuperCall(_method, args) => {
                if self.current_class != Some(ScopeType::Class) {
                    self.errors.push(ResolverError {
                        message: "Cannot use 'super' outside of a class".to_string(),
                        line: 0,
                        column: 0,
                    });
                }
                for arg in args {
                    let arg_id = self.next_id();
                    self.resolve_expression_inner(arg, arg_id);
                }
            }
            Expression::Index(obj, idx) => {
                let obj_id = self.next_id();
                let idx_id = self.next_id();
                self.resolve_expression_inner(obj, obj_id);
                self.resolve_expression_inner(idx, idx_id);
            }
            Expression::Object(fields) => {
                for (_, value) in fields {
                    let val_id = self.next_id();
                    self.resolve_expression_inner(value, val_id);
                }
            }
        }
    }

    fn resolve_function(&mut self, params: &[String], body: &Statment, scope_type: ScopeType) {
        let prev_function = self.current_function;
        let prev_class = self.current_class;
        
        self.current_function = Some(scope_type);
        self.begin_scope();
        
        for param in params {
            self.declare(param, 0, 0);
            self.define(param);
        }
        
        self.resolve_statement(body);
        
        self.end_scope();
        self.current_function = prev_function;
        self.current_class = prev_class;
    }

    // Helper methods for Class/Method resolution (Phase 2+)
    pub fn resolve_class(&mut self, name: &str, methods: &[Statment], superclass: Option<&Expression>) {
        self.declare(name, 0, 0);
        self.define(name);
        
        if let Some(superclass) = superclass {
            self.current_class = Some(ScopeType::Class);
            self.resolve_expression(superclass);
            self.begin_scope();
            self.declare("super", 0, 0);
            self.define("super");
        }
        
        self.begin_scope();
        self.declare("this", 0, 0);
        self.define("this");
        
        for method in methods {
            if let Statment::ExpressionStatement(Expression::Function(params, body)) = method {
                self.resolve_function(params, body, ScopeType::Method);
            }
        }
        
        self.end_scope();
        
        if self.current_class == Some(ScopeType::Class) {
            self.end_scope();
            self.current_class = None;
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}