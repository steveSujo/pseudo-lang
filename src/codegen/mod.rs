// Cranelift codegen module with cross-platform target support

use crate::mir::{MirProgram, MirFunction, MirInst, MirType, StackSlotId, FuncId, BlockId, IntCond};
use cranelift_codegen::ir::{self, InstBuilder, Type, Value, StackSlot, StackSlotData, StackSlotKind, AbiParam, Block};
use cranelift_codegen::ir::condcodes::IntCC;
use cranelift_codegen::isa::TargetIsa;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable as FrontendVariable};
use cranelift_module::{Module, Linkage, FuncId as ModuleFuncId, default_libcall_names};
use cranelift_object::{ObjectModule, ObjectBuilder, ObjectProduct};
use cranelift_native;
use cranelift_codegen::settings::{Configurable, FlagsOrIsa};
use target_lexicon::{Triple, Architecture, BinaryFormat, OperatingSystem, Environment};
use std::collections::HashMap;
use std::sync::Arc;

/// Target specification for cross-compilation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetSpec {
    pub triple: target_lexicon::Triple,
    pub is_windows: bool,
    pub is_macos: bool,
    pub is_linux: bool,
    pub is_musl: bool,
    pub pointer_size: u8,
}

impl TargetSpec {
    pub fn from_triple(triple: &str) -> Result<Self, String> {
        let triple = triple.parse::<target_lexicon::Triple>()
            .map_err(|e| format!("Invalid target triple: {}", e))?;
        
        let is_windows = triple.operating_system == target_lexicon::OperatingSystem::Windows;
        let is_macos = matches!(triple.operating_system, target_lexicon::OperatingSystem::MacOSX { .. });
        let is_linux = triple.operating_system == target_lexicon::OperatingSystem::Linux;
        let is_musl = triple.environment == target_lexicon::Environment::Musl;
        
        let pointer_size = match triple.architecture {
            target_lexicon::Architecture::X86_64 => 8,
            target_lexicon::Architecture::Aarch64(_) => 8,
            target_lexicon::Architecture::Arm(_) => 4,
            target_lexicon::Architecture::I386 => 4,
            _ => 8,
        };
        
        Ok(Self {
            triple,
            is_windows,
            is_macos,
            is_linux,
            is_musl,
            pointer_size,
        })
    }
    
    pub fn host() -> Self {
        let host_triple = target_lexicon::HOST;
        Self::from_triple(&host_triple.to_string()).unwrap()
    }
}

/// Codegen context for translating MIR to Cranelift IR with cross-platform support
pub struct CodegenContext {
    target: TargetSpec,
    isa: Arc<dyn TargetIsa>,
    module: ObjectModule,
    builder_context: FunctionBuilderContext,
    func_map: HashMap<FuncId, cranelift_module::FuncId>,
    stack_slots: HashMap<StackSlotId, cranelift_codegen::ir::StackSlot>,
    variables: HashMap<StackSlotId, cranelift_frontend::Variable>,
    block_map: HashMap<BlockId, cranelift_codegen::ir::Block>,
}

impl CodegenContext {
    pub fn for_host() -> Result<Self, String> {
        let target = TargetSpec::host();
        Self::new(target)
    }
    
    pub fn for_target(triple: &str) -> Result<Self, String> {
        let target = TargetSpec::from_triple(triple)?;
        Self::new(target)
    }
    
    fn new(target: TargetSpec) -> Result<Self, String> {
        let isa_builder = cranelift_native::builder()
            .map_err(|e| format!("Failed to create ISA builder: {}", e))?;
        
        let isa = isa_builder
            .finish()
            .map_err(|e| format!("Failed to finish ISA: {}", e))?;
        
        let module = ObjectModule::new(
            ObjectBuilder::new(
                target.triple.clone(),
                Arc::new(isa.clone()),
                default_libcall_names(),
            )
            .map_err(|e| format!("Failed to create ObjectBuilder: {}", e))?,
        );
        
        Ok(Self {
            target,
            isa,
            module,
            builder_context: FunctionBuilderContext::new(),
            func_map: HashMap::new(),
            stack_slots: HashMap::new(),
            variables: HashMap::new(),
            block_map: HashMap::new(),
        })
    }

    fn mir_type_to_clif(&self, ty: MirType) -> cranelift_codegen::ir::Type {
        match ty {
            MirType::I32 => cranelift_codegen::ir::Type::int(32),
            MirType::F32 => cranelift_codegen::ir::Type::f32,
            MirType::Bool => cranelift_codegen::ir::Type::int(8),
            MirType::Ptr => cranelift_codegen::ir::Type::int(self.target.pointer_size as u16 * 8),
            MirType::Ref(_) => cranelift_codegen::ir::Type::int(self.target.pointer_size as u16 * 8),
        }
    }

    fn int_cond_to_clif(&self, cond: IntCond) -> cranelift_codegen::ir::condcodes::IntCC {
        use cranelift_codegen::ir::condcodes::IntCC;
        match cond {
            IntCond::Eq => IntCC::Equal,
            IntCond::Ne => IntCC::NotEqual,
            IntCond::Lt => IntCC::SignedLessThan,
            IntCond::Le => IntCC::SignedLessThanOrEqual,
            IntCond::Gt => IntCC::SignedGreaterThan,
            IntCond::Ge => IntCC::SignedGreaterThanOrEqual,
        }
    }

    /// Compile a MIR program to Cranelift IR and emit object file
    pub fn compile_program(&mut self, program: &MirProgram) -> Result<(), String> {
        // First pass: declare all functions
        for func in &program.functions {
            let sig = self.make_signature(func);
            let func_id = self.module.declare_function(
                &func.name,
                cranelift_module::Linkage::Export,
                &sig,
            ).map_err(|e| format!("Failed to declare function {}: {}", func.name, e))?;
            
            self.func_map.insert(func.id, func_id);
        }

        // Second pass: define all functions
        for func in &program.functions {
            self.compile_function(func)?;
        }

        Ok(())
    }

    fn make_signature(&self, func: &MirFunction) -> cranelift_codegen::ir::Signature {
        let mut sig = cranelift_codegen::ir::Signature::new(
            self.isa.frontend_config().default_call_conv,
        );
        
        for (_, ty) in &func.params {
            sig.params.push(cranelift_codegen::ir::AbiParam::new(self.mir_type_to_clif(ty.clone())));
        }
        
        if let Some(ret_ty) = &func.returns {
            sig.returns.push(cranelift_codegen::ir::AbiParam::new(self.mir_type_to_clif(ret_ty.clone())));
        }
        
        sig
    }

    fn compile_function(&mut self, func: &MirFunction) -> Result<(), String> {
        let func_id = self.func_map[&func.id];
        let mut func_ctx = cranelift_codegen::Context::new();
        func_ctx.func.signature = self.make_signature(func);
        func_ctx.func.name = cranelift_codegen::ir::UserFuncName::user(0, func.id.0 as u32);

        let mut builder = FunctionBuilder::new(&mut func_ctx.func, &mut FunctionBuilderContext::new());

        // Clear state for this function
        self.stack_slots.clear();
        self.variables.clear();
        self.block_map.clear();

        // Create entry block
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        
        // Seal entry block after adding params
        builder.seal_block(entry_block);
        self.block_map.insert(func.entry_block, entry_block);

        // Create stack slots for local variables
        for (slot_id, (ty, _ownership)) in &func.stack_slots {
            let slot = builder.create_stack_slot(cranelift_codegen::ir::StackSlotData::new(
                cranelift_codegen::ir::StackSlotKind::ExplicitSlot,
                self.mir_type_to_clif(*ty).bytes() as u32,
                8, // alignment
            ));
            self.stack_slots.insert(*slot_id, slot);
        }

        // Create variables for stack slots
        for slot_id in func.stack_slots.keys() {
            let var = cranelift_frontend::Variable::new(self.variables.len() as u32);
            self.variables.insert(*slot_id, var);
            builder.declare_var(var, self.mir_type_to_clif(func.stack_slots[slot_id].0));
        }

        // Create blocks
        for block in &func.blocks {
            let clif_block = builder.create_block();
            self.block_map.insert(block.id, clif_block);
            
            // Add block parameters
            for (_slot, ty) in &block.params {
                builder.append_block_param(clif_block, self.mir_type_to_clif(*ty));
            }
        }

        // Compile each block
        for block in &func.blocks {
            let clif_block = self.block_map[&block.id];
            builder.switch_to_block(clif_block);
            
            // Add block parameters to variables if needed
            for (slot, ty) in &block.params {
                let param_index = block.params.iter().position(|(s, _)| s == slot).unwrap();
                let param = builder.block_params(clif_block)[param_index];
                let var = self.variables[slot];
                builder.def_var(var, param);
            }
            
            for inst in &block.insts {
                self.compile_inst(&mut builder, func, inst)?;
            }
        }

        // Seal all blocks
        for clif_block in self.block_map.values() {
            builder.seal_block(*clif_block);
        }

        builder.finalize();
        self.module.define_function(self.func_map[&func.id], &mut func_ctx)
            .map_err(|e| format!("Failed to define function {}: {}", func.name, e))?;

        Ok(())
    }

    fn compile_inst(&mut self, builder: &mut FunctionBuilder, func: &MirFunction, inst: &MirInst) -> Result<(), String> {
        use cranelift_codegen::ir::InstBuilder;
        
        match inst {
            MirInst::Alloc { slot, ty, ownership: _ } => {
                let stack_slot = self.stack_slots[slot];
                let var = self.variables[slot];
                let zero = builder.ins().iconst(self.mir_type_to_clif(*ty), 0);
                builder.def_var(var, zero);
            }
            MirInst::Load { dest, src, ty } => {
                let stack_slot = self.stack_slots[src];
                let value = builder.ins().stack_load(self.mir_type_to_clif(*ty), stack_slot, 0);
                let var = self.variables[dest];
                builder.def_var(var, value);
            }
            MirInst::Store { dest, src, ty } => {
                let stack_slot = self.stack_slots[dest];
                let var = self.variables[src];
                let value = builder.use_var(var);
                builder.ins().stack_store(value, stack_slot, 0);
            }
            MirInst::Move { dest, src } => {
                let src_var = self.variables[src];
                let dest_var = self.variables[dest];
                let value = builder.use_var(src_var);
                builder.def_var(dest_var, value);
            }
            MirInst::Borrow { dest, src, mutable: _ } => {
                let stack_slot = self.stack_slots[src];
                let ptr = builder.ins().stack_addr(self.mir_type_to_clif(MirType::Ptr), stack_slot, 0);
                let var = self.variables[dest];
                builder.def_var(var, ptr);
            }
            MirInst::ReleaseBorrow { slot: _ } => {
                // No-op for now
            }
            MirInst::IAdd { dest, lhs, rhs } => {
                let lhs_val = builder.use_var(self.variables[lhs]);
                let rhs_val = builder.use_var(self.variables[rhs]);
                let result = builder.ins().iadd(lhs_val, rhs_val);
                let var = self.variables[dest];
                builder.def_var(var, result);
            }
            MirInst::ISub { dest, lhs, rhs } => {
                let lhs_val = builder.use_var(self.variables[lhs]);
                let rhs_val = builder.use_var(self.variables[rhs]);
                let result = builder.ins().isub(lhs_val, rhs_val);
                let var = self.variables[dest];
                builder.def_var(var, result);
            }
            MirInst::IMul { dest, lhs, rhs } => {
                let lhs_val = builder.use_var(self.variables[lhs]);
                let rhs_val = builder.use_var(self.variables[rhs]);
                let result = builder.ins().imul(lhs_val, rhs_val);
                let var = self.variables[dest];
                builder.def_var(var, result);
            }
            MirInst::IDiv { dest, lhs, rhs } => {
                let lhs_val = builder.use_var(self.variables[lhs]);
                let rhs_val = builder.use_var(self.variables[rhs]);
                let result = builder.ins().sdiv(lhs_val, rhs_val);
                let var = self.variables[dest];
                builder.def_var(var, result);
            }
            MirInst::ICmp { dest, lhs, rhs, cond } => {
                let lhs_val = builder.use_var(self.variables[lhs]);
                let rhs_val = builder.use_var(self.variables[rhs]);
                let result = builder.ins().icmp(self.int_cond_to_clif(*cond), lhs_val, rhs_val);
                let var = self.variables[dest];
                builder.def_var(var, result);
            }
            MirInst::BrIf { cond, then_block, else_block } => {
                let cond_val = builder.use_var(self.variables[cond]);
                let then_ebb = self.block_map[then_block];
                let else_ebb = self.block_map[else_block];
                builder.ins().brif(cond_val, then_ebb, &[], else_ebb, &[]);
            }
            MirInst::Br { target } => {
                let target_ebb = self.block_map[target];
                builder.ins().jump(target_ebb, &[]);
            }
            MirInst::Call { dest, func, args } => {
                let func_id = self.func_map[func];
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(builder.use_var(self.variables[arg]));
                }
                let call = builder.ins().call(self.module.get_func_ref(self.func_map[func]), &arg_vals);
                if let Some(dest) = dest {
                    let results = builder.inst_results(call);
                    if !results.is_empty() {
                        let result = results[0];
                        let var = self.variables[dest];
                        builder.def_var(var, result);
                    }
                }
            }
            MirInst::Return { value } => {
                if let Some(val) = value {
                    let val = builder.use_var(self.variables[val]);
                    builder.ins().return_(&[val]);
                } else {
                    builder.ins().return_(&[]);
                }
            }
            MirInst::IConst { dest, value } => {
                let ty = self.mir_type_to_clif(MirType::I32);
                let const_val = builder.ins().iconst(ty, *value as i64);
                let var = self.variables[dest];
                builder.def_var(var, const_val);
            }
            MirInst::Nop => {
                // No-op
            }
        }
        Ok(())
    }

    /// Emit the object file
    pub fn emit_object(&mut self) -> Result<Vec<u8>, String> {
        let product = self.module.finish();
        let obj_bytes = product.emit().map_err(|e| format!("Failed to emit object: {}", e))?;
        Ok(obj_bytes)
    }
}

impl Default for CodegenContext {
    fn default() -> Self {
        Self::for_host().unwrap()
    }
}

/// Compile MIR program to object file
pub fn compile_mir_to_object(program: &MirProgram, _output_path: &str) -> Result<(), String> {
    let mut ctx = CodegenContext::for_host()?;
    ctx.compile_program(program)?;
    
    // Emit object file
    let obj_bytes = ctx.emit_object()?;
    
    println!("Object file size: {} bytes", obj_bytes.len());
    
    Ok(())
}

/// Compile MIR program for a specific target
pub fn compile_mir_for_target(program: &MirProgram, target_triple: &str, _output_path: &str) -> Result<(), String> {
    let target = TargetSpec::from_triple(target_triple)?;
    let mut ctx = CodegenContext::new(target)?;
    ctx.compile_program(program)?;
    
    let obj_bytes = ctx.emit_object()?;
    
    println!("Object file size: {} bytes for target {}", obj_bytes.len(), target_triple);
    
    Ok(())
}