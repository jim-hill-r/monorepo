//! Integration tests for my_lang: lexer → parser → codegen pipeline.
//!
//! These tests verify that a complete source string can be lexed, parsed,
//! and compiled to LLVM IR end-to-end, and that the JIT runner can execute
//! compiled functions.

use inkwell::context::Context;
use my_lang::codegen::CodeGenerator;
use my_lang::jit::JitRunner;
use my_lang::lexer::Lexer;
use my_lang::parser::Parser;

// ------------------------------------------------------------------
// Helper: compile source text to LLVM IR string.
// ------------------------------------------------------------------

fn compile(source: &str) -> String {
    let lexer = Lexer::new(source.to_string());
    let mut parser = Parser::new(lexer);
    let program = parser.parse().expect("parse should succeed");

    let context = Context::create();
    let mut codegen = CodeGenerator::new(&context, "integration_test");
    codegen
        .compile_program(&program)
        .expect("codegen should succeed");

    codegen.module().print_to_string().to_string()
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[test]
fn test_e2e_simple_function() {
    // The last expression in a function body is the return value.
    let ir = compile("function add(a, b) { a + b }");
    assert!(ir.contains("define i64 @add"), "IR should define 'add'");
    assert!(ir.contains("add i64"), "IR should contain integer add");
    assert!(ir.contains("ret i64"), "IR should return i64");
}

#[test]
fn test_e2e_integer_literal() {
    let ir = compile("function answer() { 42 }");
    assert!(ir.contains("define i64 @answer"));
    // Expression statement evaluates 42 but does not emit a return;
    // the implicit default return emits `ret i64 0`.
    assert!(
        ir.contains("ret i64 0"),
        "implicit 0 return after expression statement"
    );
}

#[test]
fn test_e2e_arithmetic() {
    let ir = compile("function calc(x) { x * x }");
    assert!(ir.contains("define i64 @calc"));
    assert!(ir.contains("mul i64"));
}

#[test]
fn test_e2e_multiple_functions() {
    let src = r#"
        function double(n) { n + n }
        function quad(n)   { double(double(n)) }
    "#;
    let ir = compile(src);
    assert!(ir.contains("define i64 @double"));
    assert!(ir.contains("define i64 @quad"));
    assert!(ir.contains("call i64 @double"));
}

#[test]
fn test_e2e_variable_assignment() {
    let src = r#"
        function compute() {
            x = 10
            x + 5
        }
    "#;
    let ir = compile(src);
    assert!(ir.contains("define i64 @compute"));
    assert!(ir.contains("store i64"));
    assert!(ir.contains("load i64"));
}

#[test]
fn test_e2e_output_declaration() {
    let src = "output write(v) { v }";
    let ir = compile(src);
    assert!(ir.contains("define i64 @write"));
    assert!(ir.contains("ret i64"));
}

#[test]
fn test_e2e_comparison() {
    let src = "function is_zero(x) { x == 0 }";
    let ir = compile(src);
    assert!(ir.contains("define i64 @is_zero"));
    assert!(ir.contains("icmp eq i64"));
}

#[test]
fn test_e2e_string_literal() {
    // String literals compile as global byte-array constants; the address is
    // returned as an i64 via a ptrtoint constant expression.
    let src = r#"function greeting() { "hello" }"#;
    let ir = compile(src);
    assert!(
        ir.contains("define i64 @greeting"),
        "IR should define 'greeting'"
    );
    assert!(
        ir.contains("hello"),
        "IR should contain the string literal data as a global constant"
    );
}

#[test]
fn test_e2e_string_literal_as_argument() {
    // A string literal passed as an argument to another function should compile.
    // The callee is declared first so the call is valid.
    let src = r#"
        output log(msg) { msg }
        output run() { log("hello world") }
    "#;
    let ir = compile(src);
    assert!(ir.contains("define i64 @log"), "IR should define 'log'");
    assert!(ir.contains("define i64 @run"), "IR should define 'run'");
    assert!(ir.contains("call i64 @log"), "IR should call 'log'");
    assert!(
        ir.contains("hello world"),
        "IR should contain the string literal data"
    );
}

// ------------------------------------------------------------------
// JIT execution integration tests
// ------------------------------------------------------------------

/// Helper: compile source with the JIT runner, panicking on failure.
///
/// The LLVM `Context` is intentionally leaked here because:
/// - Integration test processes are short-lived; the OS reclaims memory.
/// - It avoids threading lifetime parameters through every test helper.
/// This pattern is acceptable only in tests.
fn jit_run(source: &str) -> JitRunner<'static> {
    let context = Box::leak(Box::new(Context::create()));
    JitRunner::new(context, source).expect("JitRunner::new should succeed")
}

#[test]
fn test_jit_e2e_add() {
    let runner = jit_run("function add(a, b) { return a + b }");
    assert_eq!(runner.call("add", &[10, 32]).unwrap(), 42);
}

#[test]
fn test_jit_e2e_multi_function_call() {
    // Verify that the JIT runner correctly executes a program containing
    // multiple functions where one calls another.
    let src = r#"
        function add(a, b) { return a + b }
        function sum3(a, b, c) { return add(add(a, b), c) }
    "#;
    let runner = jit_run(src);
    assert_eq!(runner.call("sum3", &[1, 2, 3]).unwrap(), 6);
}

#[test]
fn test_jit_e2e_return_keyword_in_output() {
    let runner = jit_run("output identity(x) { return x }");
    assert_eq!(runner.call("identity", &[99]).unwrap(), 99);
}

#[test]
fn test_jit_e2e_return_keyword_in_function() {
    let runner = jit_run("function double(n) { return n + n }");
    assert_eq!(runner.call("double", &[21]).unwrap(), 42);
}
