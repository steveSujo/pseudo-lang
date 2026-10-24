use psudo::{lexer::Lexer, parser::{Interpret, Parser}};

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