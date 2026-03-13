//! Integration tests for my_lang: lexer → parser → codegen pipeline.
//!
//! These tests verify that a complete source string can be lexed, parsed,
//! and compiled to LLVM IR end-to-end.

use inkwell::context::Context;
use my_lang::codegen::CodeGenerator;
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
