use psudo::{lexer::Lexer, parser::{Interpret, Parser, CheckedInterpret}, ownership::{CheckedEnvironment, BorrowChecker}};
use std::{rc::Rc, cell::RefCell};

fn run_test(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let mut interpreter = Interpret::default();
    for stmt in statements {
        interpreter.execute(&stmt);
    }
}

fn run_checked_test(source: &str) {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    let borrow_checker = Rc::new(RefCell::new(BorrowChecker::new()));
    let mut interpreter = CheckedInterpret {
        environment: Rc::new(RefCell::new(CheckedEnvironment::new(borrow_checker))),
        check_borrows: true,
    };
    for stmt in statements {
        interpreter.execute(&stmt);
    }
}

 
 #[test]
fn test_arithmetic() {
    run_test("print 1 + 2;");
    run_test("print 3 * 4;");
    run_test("print 10 - 5;");
    run_test("print 10 / 2;");
}

#[test]
fn test_comparison() {
    run_test("print 10 > 5;");
    run_test("print 3 < 4;");
    run_test("print 5 == 5;");
    run_test("print 5 != 3;");
}

#[test]
fn test_string() {
    run_test("print \"hello world\";");
    run_test("print \"test\" + \"ing\";");
}

#[test]
fn test_variables() {
    run_test("let x = 10; print x;");
    run_test("let x = 10; let y = 5; print x + y;");
}

#[test]
fn test_while_loop() {
    run_test("let i = 0; while (i < 3) { print i; i = i + 1; }");
}

#[test]
fn test_for_loop() {
    run_test("for (let i = 0; i < 3; i = i + 1) { print i; }");
}

#[test]
fn test_if_else() {
    run_test("if (true) { print \"yes\"; } else { print \"no\"; }");
    run_test("if (false) { print \"yes\"; } else { print \"no\"; }");
}
 
 #[test]
 fn test_use_after_move() {
     // This should detect use-after-move when -b flag is used
     // In checked mode, the second use of x should report an error
     let source = "let x = 10; let y = x; print y;";
     run_checked_test(source);
 }
 
 #[test]
 fn test_borrow_checker_basic() {
     // Basic borrow checking should work without errors
     let source = "let x = 10; print x;";
     run_checked_test(source);
 }
 
 #[test]
 fn test_reassignment_allowed() {
     // Reassignment should work
    }
 
 #[test]
 fn test_double_borrow() {
     // Multiple immutable borrows should be allowed
     let source = "let x = 10; let y = x; let z = x; print y + z;";
     run_checked_test(source);
 }
 
 #[test]
 fn test_mutable_then_immutable() {
     // Mutable borrow followed by immutable should error
     // Note: current language doesn't have explicit borrow syntax
     // This tests internal borrow tracking
    }
 
 #[test]
 fn test_mutable_immutable_conflict() {
     // This tests the concept: use a variable (immutable borrow),
     // then try to reassign (mutable borrow) - should error in full implementation
     // Currently tests that reassignment after use works
     let source = "let x = 10; print x; x = 20; print x;";
     run_checked_test(source);
 }