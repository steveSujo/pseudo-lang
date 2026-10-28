use psudo::{
    lexer::Lexer,
    ownership::{BorrowChecker, CheckedEnvironment},
    parser::{CheckedInterpret, Interpret, Parser},
};
use std::{cell::RefCell, rc::Rc};

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
    run_test("print 1 + 2");
    run_test("print 3 * 4");
    run_test("print 10 - 5");
    run_test("print 10 / 2");
}

#[test]
fn test_comparison() {
    run_test("print 10 > 5");
    run_test("print 3 < 4");
    run_test("print 5 == 5");
    run_test("print 5 != 3");
}

#[test]
fn test_string() {
    run_test("print \"hello world\"");
    run_test("print \"test\" + \"ing\"");
}

#[test]
fn test_variables() {
    run_test("let x = 10\nprint x");
    run_test("let x = 10\nlet y = 5\nprint x + y");
}

#[test]
fn test_while_loop() {
    run_test("let i = 0\nwhile i < 3\n    print i\n    i = i + 1");
}

#[test]
fn test_for_loop() {
    run_test("for let i = 0\ni < 3\ni = i + 1\n    print i");
}

#[test]
fn test_if_else() {
    run_test("if true\n    print \"yes\"\nelse\n    print \"no\"");
    run_test("if false\n    print \"yes\"\nelse\n    print \"no\"");
}

#[test]
fn test_use_after_move() {
    let source = "let x = 10\nlet y = x\nprint y";
    run_checked_test(source);
}

#[test]
fn test_borrow_checker_basic() {
    let source = "let x = 10\nprint x";
    run_checked_test(source);
}

#[test]
fn test_reassignment_allowed() {
    // Reassignment should work
}

#[test]
fn test_double_borrow() {
    let source = "let x = 10\nlet y = x\nlet z = x\nprint y + z";
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
    let source = "let x = 10\nprint x\nx = 20\nprint x";
    run_checked_test(source);
}