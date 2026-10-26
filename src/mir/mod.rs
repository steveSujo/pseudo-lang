// MIR (Mid-level IR) with ownership annotations
// This is the intermediate representation between AST and Cranelift

use crate::parser::{Expression, Statment, DataTypes};
use crate::tokens::{Token, TokenType};
use crate::ownership::{Ownership, Lifetime};
use std::collections::HashMap;

/// MIR types that mirror Cranelift types
#[derive(Debug, Clone, PartialEq)]
pub enum MirType {
    I32,
    F32,
    Bool,
    Ptr,           // Pointer to owned/borrowed memory
    Ref(Ownership), // Reference with ownership info
}

/// MIR values with ownership tracking
#[derive(Debug, Clone, PartialEq)]
pub enum MirValue {
    /// Immediate integer value
    ConstI32(i32),
    /// Immediate float value
    ConstF32(f32),
    /// Immediate boolean value
    ConstBool(bool),
    /// Stack slot reference (local variable)
    StackSlot(StackSlotId),
    /// Function reference
    FuncRef(FuncId),
    /// Reference with ownership info
    Ref(StackSlotId, Ownership),
}

/// Stack slot for local variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackSlotId(pub usize);

/// Function identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncId(pub usize);

/// Basic block identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockId(pub usize);

/// MIR instruction with ownership semantics
#[derive(Debug, Clone)]
pub enum MirInst {
    /// Allocate stack slot for variable
    Alloc {
        slot: StackSlotId,
        ty: MirType,
        ownership: Ownership,
    },
    /// Load value from stack slot
    Load {
        dest: StackSlotId,
        src: StackSlotId,
        ty: MirType,
    },
    /// Store value to stack slot
    Store {
        dest: StackSlotId,
        src: StackSlotId,
        ty: MirType,
    },
    /// Move value (transfer ownership)
    Move {
        dest: StackSlotId,
        src: StackSlotId,
    },
    /// Borrow (create reference)
    Borrow {
        dest: StackSlotId,
        src: StackSlotId,
        mutable: bool,
    },
    /// Release borrow
    ReleaseBorrow {
        slot: StackSlotId,
    },
    /// Integer arithmetic
    IAdd { dest: StackSlotId, lhs: StackSlotId, rhs: StackSlotId },
    ISub { dest: StackSlotId, lhs: StackSlotId, rhs: StackSlotId },
    IMul { dest: StackSlotId, lhs: StackSlotId, rhs: StackSlotId },
    IDiv { dest: StackSlotId, lhs: StackSlotId, rhs: StackSlotId },
    /// Comparison
    ICmp { dest: StackSlotId, lhs: StackSlotId, rhs: StackSlotId, cond: IntCond },
    /// Conditional branch
    BrIf { cond: StackSlotId, then_block: BlockId, else_block: BlockId },
    /// Unconditional branch
    Br { target: BlockId },
    /// Function call
    Call {
        dest: Option<StackSlotId>,
        func: FuncId,
        args: Vec<StackSlotId>,
    },
    /// Return from function
    Return { value: Option<StackSlotId> },
    /// Load constant integer
    IConst { dest: StackSlotId, value: i32 },
    /// Nop (for placeholder)
    Nop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IntCond {
    Eq, Ne, Lt, Le, Gt, Ge,
}

/// Basic block with instructions
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub insts: Vec<MirInst>,
    pub params: Vec<(StackSlotId, MirType)>, // Block parameters (for loops, etc.)
}

/// Function in MIR
#[derive(Debug, Clone)]
pub struct MirFunction {
    pub id: FuncId,
    pub name: String,
    pub params: Vec<(StackSlotId, MirType)>,
    pub returns: Option<MirType>,
    pub blocks: Vec<BasicBlock>,
    pub entry_block: BlockId,
    pub stack_slots: HashMap<StackSlotId, (MirType, Ownership)>,
}

/// MIR program containing all functions
#[derive(Debug, Clone)]
pub struct MirProgram {
    pub functions: Vec<MirFunction>,
    pub main_func: FuncId,
}

/// Context for building MIR
pub struct MirBuilder {
    next_slot: usize,
    next_block: usize,
    next_func: usize,
    current_func: Option<FuncId>,
    current_block: Option<BlockId>,
    program: MirProgram,
}

impl MirBuilder {
    pub fn new() -> Self {
        Self {
            next_slot: 0,
            next_block: 0,
            next_func: 0,
            current_func: None,
            current_block: None,
            program: MirProgram {
                functions: Vec::new(),
                main_func: FuncId(0),
            },
        }
    }

    fn new_slot(&mut self) -> StackSlotId {
        let id = StackSlotId(self.next_slot);
        self.next_slot += 1;
        id
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.next_block);
        self.next_block += 1;
        id
    }

    fn new_func(&mut self) -> FuncId {
        let id = FuncId(self.next_func);
        self.next_func += 1;
        id
    }

    pub fn start_function(&mut self, name: String, params: Vec<(String, MirType)>, returns: Option<MirType>) -> FuncId {
        let func_id = self.new_func();
        let mut param_slots = Vec::new();

        for (param_name, param_ty) in params {
            let slot = self.new_slot();
            param_slots.push((slot, param_ty));
        }

        let entry_block = self.new_block();

        let func = MirFunction {
            id: func_id,
            name,
            params: param_slots.clone(),
            returns,
            blocks: vec![BasicBlock {
                id: entry_block,
                insts: Vec::new(),
                params: param_slots,
            }],
            entry_block,
            stack_slots: HashMap::new(),
        };

        self.current_func = Some(func_id);
        self.current_block = Some(entry_block);
        self.program.functions.push(func);

        func_id
    }

    pub fn end_function(&mut self) {
        self.current_func = None;
        self.current_block = None;
    }

    fn current_func_mut(&mut self) -> &mut MirFunction {
        let func_id = self.current_func.expect("No current function");
        self.program.functions.iter_mut()
            .find(|f| f.id == func_id)
            .expect("Function not found")
    }

    fn current_block_mut(&mut self) -> &mut BasicBlock {
        let block_id = self.current_block.expect("No current block");
        let func = self.current_func_mut();
        func.blocks.iter_mut()
            .find(|b| b.id == block_id)
            .expect("Block not found")
    }

    pub fn add_inst(&mut self, inst: MirInst) {
        self.current_block_mut().insts.push(inst);
    }

    pub fn new_slot_typed(&mut self, ty: MirType, ownership: Ownership) -> StackSlotId {
        let slot = self.new_slot();
        self.current_func_mut().stack_slots.insert(slot, (ty, ownership));
        slot
    }

    pub fn new_block_with_params(&mut self, params: Vec<(StackSlotId, MirType)>) -> BlockId {
        let block_id = self.new_block();
        self.current_func_mut().blocks.push(BasicBlock {
            id: block_id,
            insts: Vec::new(),
            params,
        });
        self.current_block = Some(block_id);
        block_id
    }

    pub fn set_main(&mut self, func_id: FuncId) {
        self.program.main_func = func_id;
    }

    pub fn finish(mut self) -> MirProgram {
        // Ensure main function is set
        if self.program.functions.is_empty() {
            panic!("No functions defined");
        }
        self.program
    }
}

pub mod lower;
pub use lower::MirLower;

impl Default for MirBuilder {
    fn default() -> Self {
        Self::new()
    }
}