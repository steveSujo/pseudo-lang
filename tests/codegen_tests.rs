// Integration tests for MIR lowering and codegen

use psudo::{lexer::Lexer, parser::Parser, mir::MirLower};

fn lower_to_mir(source: &str) -> psudo::mir::MirProgram {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut lower = MirLower::new();
    lower.lower_program(&statements)
}

#[test]
fn test_mir_simple_arithmetic() {
    let source = "let x = 10; let y = 20; let z = x + y;";
    let program = lower_to_mir(source);
    
    assert_eq!(program.functions.len(), 1); // main function
    let main = &program.functions[0];
    assert_eq!(main.name, "main");
    
    // Check that we have basic blocks
    assert!(!main.blocks.is_empty());
    
    // Check that we have stack slots for variables
    assert!(main.stack_slots.len() >= 3); // x, y, z
}

#[test]
fn test_mir_if_statement() {
    let source = "let x = 10; if (x > 5) { let y = 1; } else { let y = 2; }";
    let program = lower_to_mir(source);
    
    let main = &program.functions[0];
    assert!(!main.blocks.is_empty());
    
    // Should have multiple blocks for if/else
    assert!(main.blocks.len() >= 3); // cond, then, else, merge
}

#[test]
fn test_mir_while_loop() {
    let source = "let i = 0; while (i < 10) { i = i + 1; }";
    let program = lower_to_mir(source);
    
    let main = &program.functions[0];
    assert!(!main.blocks.is_empty());
    
    // Should have cond, body, after blocks
    assert!(main.blocks.len() >= 3);
}

#[test]
fn test_mir_function_declaration() {
    let source = "fn add(a, b) { return a + b; }; let x = add(1, 2);";
    let program = lower_to_mir(source);
    
    // Should have main + add function
    assert_eq!(program.functions.len(), 2);
    
    let add_func = program.functions.iter().find(|f| f.name == "add");
    assert!(add_func.is_some());
    
    let add = add_func.unwrap();
    assert_eq!(add.params.len(), 2); // a, b
    assert_eq!(add.returns, Some(psudo::mir::MirType::I32));
}

#[test]
fn test_mir_function_call() {
    let source = "fn add(a, b) { return a + b; }; let x = add(1, 2);";
    let program = lower_to_mir(source);
    
    let main = &program.functions[0];
    
    // Check that main has Call instruction
    let has_call = main.blocks.iter().any(|block| {
        block.insts.iter().any(|inst| {
            matches!(inst, psudo::mir::MirInst::Call { .. })
        })
    });
    assert!(has_call);
}

#[test]
fn test_mir_return_statement() {
    let source = "fn test() { return 42; };";
    let program = lower_to_mir(source);
    
    let test_func = program.functions.iter().find(|f| f.name == "test");
    assert!(test_func.is_some());
    
    let func = test_func.unwrap();
    let has_return = func.blocks.iter().any(|block| {
        block.insts.iter().any(|inst| {
            matches!(inst, psudo::mir::MirInst::Return { .. })
        })
    });
    assert!(has_return);
}

#[test]
fn test_mir_comparison_ops() {
    let source = "let x = 10; let y = x > 5; let z = x == 10;";
    let program = lower_to_mir(source);
    
    let main = &program.functions[0];
    
    // Check for ICmp instructions
    let has_icmp = main.blocks.iter().any(|block| {
        block.insts.iter().any(|inst| {
            matches!(inst, psudo::mir::MirInst::ICmp { .. })
        })
    });
    assert!(has_icmp);
}

#[test]
fn test_codegen_compile_simple() {
    let source = "fn main() { let x = 10; let y = 20; return x + y; };";
    let mut lexer = psudo::lexer::Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    
    let mut lower = MirLower::new();
    let program = lower.lower_program(&statements);
    
    // Test that codegen doesn't panic
    let mut ctx = psudo::codegen::CodegenContext::for_host().unwrap();
    let result = ctx.compile_program(&program);
    assert!(result.is_ok());
}

#[test]
fn test_codegen_multiple_functions() {
    let source = "
    fn add(a, b) { return a + b; };
    fn main() { 
        let x = add(1, 2); 
        return x; 
    };
    ";
    let mut lexer = psudo::lexer::Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    
    let mut lower = MirLower::new();
    let program = lower.lower_program(&statements);
    
    let mut ctx = psudo::codegen::CodegenContext::for_host().unwrap();
    let result = ctx.compile_program(&program);
    assert!(result.is_ok());
}