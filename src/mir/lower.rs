// MIR lowering: convert AST to MIR

use crate::mir::{MirBuilder, MirInst, MirType, StackSlotId, FuncId, BlockId, IntCond, Ownership};
use crate::parser::{Expression, Statment, DataTypes};
use crate::tokens::{Token, TokenType, LiteralType};
use std::collections::HashMap;

/// Lower AST to MIR
pub struct MirLower {
    builder: MirBuilder,
    slot_map: HashMap<String, StackSlotId>, // Variable name -> stack slot
    block_stack: Vec<BlockId>, // For break/continue
}

impl MirLower {
    pub fn new() -> Self {
        Self {
            builder: MirBuilder::new(),
            slot_map: HashMap::new(),
            block_stack: Vec::new(),
        }
    }

    pub fn lower_program(mut self, statements: &[Statment]) -> crate::mir::MirProgram {
        // Create main function
        let main_func = self.builder.start_function("main".to_string(), vec![], None);

        for stmt in statements {
            self.lower_statement(stmt);
        }

        // Add implicit return if needed
        let current_block = self.builder.current_block;
        if let Some(block_id) = current_block {
            let block = self.builder.current_func_mut().blocks.iter_mut()
                .find(|b| b.id == block_id);
            if let Some(block) = block {
                if block.insts.is_empty() || !matches!(block.insts.last(), Some(crate::mir::MirInst::Return { .. })) {
                    self.builder.add_inst(crate::mir::MirInst::Return { value: None });
                }
            }
        }

        self.builder.set_main(main_func);
        self.builder.finish()
    }

    fn lower_statement(&mut self, stmt: &Statment) {
        match stmt {
            Statment::ExpressionStatement(expr) => {
                // Handle return statements which are ExpressionStatement with Expression::Return
                if let Expression::Return(value) = expr {
                    let val_slot = value.as_ref().map(|e| self.lower_expression(e));
                    self.builder.add_inst(crate::mir::MirInst::Return { value: val_slot });
                } else {
                    // Handle function declaration (ExpressionStatement with Function)
                    if let Expression::Assign(name, value) = expr {
                        // Check if value is a Function expression
                        if let Expression::Function(params, body) = value.as_ref() {
                            self.lower_function_declaration(name, &params, &*body);
                        } else {
                            self.lower_expression(expr);
                        }
                    } else {
                        self.lower_expression(expr);
                    }
                }
            }
            Statment::PrintStatement(expr) => {
                // For now, just evaluate the expression
                self.lower_expression(expr);
                // TODO: Add print call
            }
            Statment::LetStatement(name, initializer) => {
                let slot = self.lower_let_statement(&name.text, initializer);
                self.slot_map.insert(name.text.clone(), slot);
            }
            Statment::IfStatement(condition, then_branch, else_branch) => {
                self.lower_if_statement(condition, then_branch, else_branch);
            }
            Statment::WhileStatement(condition, body) => {
                self.lower_while_statement(condition, body);
            }
            Statment::ForStatement(init, condition, increment, body) => {
                self.lower_for_statement(init, condition, increment, body);
            }
            Statment::BlockStatement(statements) => {
                for stmt in statements {
                    self.lower_statement(stmt);
                }
            }
        }
    }

    fn lower_let_statement(&mut self, _name: &str, initializer: &Expression) -> StackSlotId {
        let init_slot = self.lower_expression(initializer);
        let slot = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
        
        // Move the value to the new slot
        self.builder.add_inst(crate::mir::MirInst::Move { dest: slot, src: init_slot });
        
        slot
    }

    fn lower_expression(&mut self, expr: &Expression) -> StackSlotId {
        match expr {
            Expression::Binary(left, op, right) => {
                let lhs = self.lower_expression(left);
                let rhs = self.lower_expression(right);
                let dest = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                
                let inst = match op.token_type {
                    TokenType::PLUS => crate::mir::MirInst::IAdd { dest, lhs, rhs },
                    TokenType::MINUS => crate::mir::MirInst::ISub { dest, lhs, rhs },
                    TokenType::STAR => crate::mir::MirInst::IMul { dest, lhs, rhs },
                    TokenType::SLASH => crate::mir::MirInst::IDiv { dest, lhs, rhs },
                    TokenType::EqualEqual => crate::mir::MirInst::ICmp { dest, lhs, rhs, cond: IntCond::Eq },
                    TokenType::BangEqual => crate::mir::MirInst::ICmp { dest, lhs, rhs, cond: IntCond::Ne },
                    TokenType::LESS => crate::mir::MirInst::ICmp { dest, lhs, rhs, cond: IntCond::Lt },
                    TokenType::LessEqual => crate::mir::MirInst::ICmp { dest, lhs, rhs, cond: IntCond::Le },
                    TokenType::GREATER => crate::mir::MirInst::ICmp { dest, lhs, rhs, cond: IntCond::Gt },
                    TokenType::GreaterEqual => crate::mir::MirInst::ICmp { dest, lhs, rhs, cond: IntCond::Ge },
                    _ => unimplemented!("Binary op {:?}", op.token_type),
                };
                
                self.builder.add_inst(inst);
                dest
            }
            Expression::Unary(op, right) => {
                let operand = self.lower_expression(right);
                // For now, only handle negation
                if op.token_type == TokenType::MINUS {
                    let dest = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                    let zero = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                    self.builder.add_inst(crate::mir::MirInst::IConst { dest: zero, value: 0 });
                    self.builder.add_inst(crate::mir::MirInst::ISub { dest, lhs: zero, rhs: operand });
                    dest
                } else {
                    // ! operator - logical not
                    let dest = self.builder.new_slot_typed(MirType::Bool, Ownership::Owned);
                    // For simplicity, treat as integer comparison with 0
                    let zero = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                    self.builder.add_inst(crate::mir::MirInst::IConst { dest: zero, value: 0 });
                    self.builder.add_inst(crate::mir::MirInst::ICmp { 
                        dest, 
                        lhs: operand, 
                        rhs: zero, 
                        cond: IntCond::Eq 
                    });
                    dest
                }
            }
            Expression::Literal(token) => {
                // Handle variable access (IDENTIFIER token)
                if token.token_type == TokenType::IDENTIFIER {
                    if let Some(&slot) = self.slot_map.get(&token.text) {
                        slot
                    } else {
                        // Create new slot for undefined variable
                        let slot = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                        self.slot_map.insert(token.text.clone(), slot);
                        slot
                    }
                } else {
                    // Regular literal (number, bool, etc.)
                    let dest = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                    let value = match &token.literal {
                        Some(LiteralType::Number(n)) => *n as i32,
                        Some(LiteralType::Bool(b)) => if *b { 1 } else { 0 },
                        _ => 0,
                    };
                    self.builder.add_inst(crate::mir::MirInst::IConst { dest, value });
                    dest
                }
            }
            Expression::Grouping(exp) => {
                self.lower_expression(exp)
            }
            Expression::Assign(name, value) => {
                let val_slot = self.lower_expression(value);
                if let Some(&slot) = self.slot_map.get(name) {
                    // Variable exists, move the value
                    self.builder.add_inst(crate::mir::MirInst::Move { dest: slot, src: val_slot });
                    slot
                } else {
                    // Create new slot
                    let slot = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                    self.builder.add_inst(crate::mir::MirInst::Move { dest: slot, src: val_slot });
                    self.slot_map.insert(name.clone(), slot);
                    slot
                }
            }
            Expression::Call(name, args) => {
                let arg_slots: Vec<StackSlotId> = args.iter().map(|a| self.lower_expression(a)).collect();
                
                // Find function
                let func_id = self.builder.program.functions.iter()
                    .find(|f| f.name == *name)
                    .map(|f| f.id)
                    .unwrap_or_else(|| {
                        // Create external function placeholder
                        let id = self.builder.new_func();
                        self.builder.program.functions.push(crate::mir::MirFunction {
                            id,
                            name: name.clone(),
                            params: vec![],
                            returns: Some(MirType::I32),
                            blocks: vec![],
                            entry_block: crate::mir::BlockId(0),
                            stack_slots: std::collections::HashMap::new(),
                        });
                        id
                    });
                
                let dest = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                self.builder.add_inst(crate::mir::MirInst::Call {
                    dest: Some(dest),
                    func: func_id,
                    args: arg_slots,
                });
                dest
            }
            Expression::Function(params, body) => {
                // Function literal - handled at statement level
                let dest = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                self.builder.add_inst(crate::mir::MirInst::IConst { dest, value: 0 });
                dest
            }
            Expression::Return(value) => {
                let dest = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                self.builder.add_inst(crate::mir::MirInst::IConst { dest, value: 0 });
                dest
            }
            _ => {
                // Fallback for unsupported expressions
                let dest = self.builder.new_slot_typed(MirType::I32, Ownership::Owned);
                self.builder.add_inst(crate::mir::MirInst::IConst { dest, value: 0 });
                dest
            }
        }
    }

    fn lower_if_statement(&mut self, condition: &Expression, then_branch: &Statment, else_branch: &Option<Box<Statment>>) {
        let cond_slot = self.lower_expression(condition);
        
        let then_block = self.builder.new_block_with_params(vec![]);
        let else_block = self.builder.new_block_with_params(vec![]);
        let merge_block = self.builder.new_block_with_params(vec![]);
        
        self.builder.add_inst(crate::mir::MirInst::BrIf {
            cond: cond_slot,
            then_block,
            else_block: else_branch.is_some().then_some(else_block).unwrap_or(merge_block),
        });
        
        // Lower then branch
        self.builder.current_block = Some(then_block);
        self.lower_statement(then_branch);
        self.builder.add_inst(crate::mir::MirInst::Br { target: merge_block });
        
        // Lower else branch
        if let Some(else_stmt) = else_branch {
            self.builder.current_block = Some(else_block);
            self.lower_statement(else_stmt);
        } else {
            self.builder.current_block = Some(else_block);
        }
        self.builder.add_inst(crate::mir::MirInst::Br { target: merge_block });
        
        // Continue at merge block
        self.builder.current_block = Some(merge_block);
    }

    fn lower_while_statement(&mut self, condition: &Expression, body: &Statment) {
        let cond_block = self.builder.new_block_with_params(vec![]);
        let body_block = self.builder.new_block_with_params(vec![]);
        let after_block = self.builder.new_block_with_params(vec![]);
        
        // Save block stack for break/continue
        self.block_stack.push(after_block);
        
        // Jump to condition
        self.builder.add_inst(crate::mir::MirInst::Br { target: cond_block });
        
        // Condition block
        self.builder.current_block = Some(cond_block);
        let cond_slot = self.lower_expression(condition);
        self.builder.add_inst(crate::mir::MirInst::BrIf {
            cond: cond_slot,
            then_block: body_block,
            else_block: after_block,
        });
        
        // Body block
        self.builder.current_block = Some(body_block);
        self.lower_statement(body);
        self.builder.add_inst(crate::mir::MirInst::Br { target: cond_block });
        
        // After block
        self.builder.current_block = Some(after_block);
        self.block_stack.pop();
    }

    fn lower_for_statement(&mut self, init: &Option<Box<Statment>>, condition: &Option<Expression>, increment: &Option<Expression>, body: &Statment) {
        // Lower init
        if let Some(init_stmt) = init {
            self.lower_statement(init_stmt);
        }
        
        let cond_block = self.builder.new_block_with_params(vec![]);
        let body_block = self.builder.new_block_with_params(vec![]);
        let inc_block = self.builder.new_block_with_params(vec![]);
        let after_block = self.builder.new_block_with_params(vec![]);
        
        self.block_stack.push(after_block);
        
        // Jump to condition
        self.builder.add_inst(crate::mir::MirInst::Br { target: cond_block });
        
        // Condition block
        self.builder.current_block = Some(cond_block);
        let cond_slot = if let Some(cond_expr) = condition {
            self.lower_expression(cond_expr)
        } else {
            let true_slot = self.builder.new_slot_typed(MirType::Bool, Ownership::Owned);
            self.builder.add_inst(crate::mir::MirInst::IConst { dest: true_slot, value: 1 });
            true_slot
        };
        self.builder.add_inst(crate::mir::MirInst::BrIf {
            cond: cond_slot,
            then_block: body_block,
            else_block: after_block,
        });
        
        // Body block
        self.builder.current_block = Some(body_block);
        self.lower_statement(body);
        self.builder.add_inst(crate::mir::MirInst::Br { target: inc_block });
        
        // Increment block
        self.builder.current_block = Some(inc_block);
        if let Some(inc_expr) = increment {
            self.lower_expression(inc_expr);
        }
        self.builder.add_inst(crate::mir::MirInst::Br { target: cond_block });
        
        // After block
        self.builder.current_block = Some(after_block);
        self.block_stack.pop();
    }

    fn lower_function_declaration(&mut self, name: &str, params: &[String], body: &Statment) {
        // Create function
        let param_tys: Vec<(String, MirType)> = params.iter().map(|p| (p.clone(), MirType::I32)).collect();
        let func_id = self.builder.start_function(name.to_string(), param_tys, Some(MirType::I32));
        
        // Allocate slots for parameters
        let param_slots: Vec<StackSlotId> = self.builder.current_func_mut().params.iter()
            .map(|(slot, _)| *slot)
            .collect();
        
        for (param, slot) in params.iter().zip(param_slots.iter()) {
            self.slot_map.insert(param.clone(), *slot);
        }
        
        // Lower body
        self.lower_statement(body);
        
        // Add implicit return if needed
        let current_block = self.builder.current_block;
        if let Some(block_id) = current_block {
            let block = self.builder.current_func_mut().blocks.iter_mut()
                .find(|b| b.id == block_id);
            if let Some(block) = block {
                if block.insts.is_empty() || !matches!(block.insts.last(), Some(crate::mir::MirInst::Return { .. })) {
                    self.builder.add_inst(crate::mir::MirInst::Return { value: None });
                }
            }
        }
        
        self.builder.end_function();
        
        // Restore main function context
        let main_id = self.builder.program.main_func;
        if let Some(main_func) = self.builder.program.functions.iter().find(|f| f.id == main_id) {
            self.builder.current_func = Some(main_id);
            self.builder.current_block = Some(main_func.entry_block);
        }
    }
}

impl Default for MirLower {
    fn default() -> Self {
        Self::new()
    }
}
