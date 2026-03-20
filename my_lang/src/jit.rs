//! JIT compilation and execution for my_lang.
//!
//! This module provides the [`JitRunner`] type, which compiles a my_lang source
//! string all the way to native machine code via LLVM's JIT engine and allows
//! calling the resulting functions directly.
//!
//! ## Example
//!
//! ```ignore
//! use inkwell::context::Context;
//! use my_lang::jit::JitRunner;
//!
//! let context = Context::create();
//! let runner = JitRunner::new(&context, "function add(a, b) { a + b }")
//!     .expect("compile should succeed");
//! let result = runner.call("add", &[3, 4]).expect("call should succeed");
//! assert_eq!(result, 7);
//! ```

use inkwell::OptimizationLevel;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;

use crate::codegen::{CodeGenerator, CodegenError};
use crate::lexer::Lexer;
use crate::parser::Parser;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur when creating or using a [`JitRunner`].
#[derive(Debug)]
pub enum JitError {
    /// The source code could not be parsed.
    ParseError(String),
    /// Code generation failed.
    CodegenError(CodegenError),
    /// The LLVM JIT execution engine could not be created.
    JitCreationFailed(String),
    /// No function with the given name was found in the module.
    FunctionNotFound(String),
    /// The function was called with more arguments than are supported.
    TooManyArguments,
}

impl std::fmt::Display for JitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitError::ParseError(msg) => write!(f, "parse error: {msg}"),
            JitError::CodegenError(e) => write!(f, "codegen error: {e}"),
            JitError::JitCreationFailed(msg) => write!(f, "JIT creation failed: {msg}"),
            JitError::FunctionNotFound(name) => write!(f, "function not found: '{name}'"),
            JitError::TooManyArguments => write!(f, "too many arguments (max 8 supported)"),
        }
    }
}

impl std::error::Error for JitError {}

impl From<CodegenError> for JitError {
    fn from(e: CodegenError) -> Self {
        JitError::CodegenError(e)
    }
}

// ---------------------------------------------------------------------------
// JitRunner
// ---------------------------------------------------------------------------

/// Compiles and executes my_lang source code using LLVM's JIT engine.
///
/// The runner is tied to the lifetime of the [`Context`] provided at
/// construction time.  The context must outlive the runner.
///
/// All my_lang values are 64-bit signed integers (`i64`), so all function
/// parameters and return values are represented as [`i64`].
pub struct JitRunner<'ctx> {
    engine: ExecutionEngine<'ctx>,
}

impl<'ctx> JitRunner<'ctx> {
    /// Compile `source` and prepare it for JIT execution.
    ///
    /// This runs the full compilation pipeline: lexer → parser → LLVM codegen
    /// → JIT engine creation.
    ///
    /// # Errors
    ///
    /// Returns a [`JitError`] if parsing, code generation, or JIT engine
    /// creation fails.
    pub fn new(context: &'ctx Context, source: &str) -> Result<Self, JitError> {
        let lexer = Lexer::new(source.to_string());
        let mut parser = Parser::new(lexer);
        let program = parser
            .parse()
            .map_err(|e| JitError::ParseError(format!("{e:?}")))?;

        let mut codegen = CodeGenerator::new(context, "jit_module");
        codegen.compile_program(&program)?;

        let module = codegen.take_module();
        let engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| JitError::JitCreationFailed(e.to_string()))?;

        Ok(Self { engine })
    }

    /// Call a compiled function by name with `i64` arguments and return its
    /// `i64` result.
    ///
    /// Up to 8 arguments are supported.  All my_lang values are `i64`, so
    /// every argument and the return value must be a 64-bit signed integer.
    ///
    /// # Safety
    ///
    /// This function is safe to call, but internally uses `unsafe` code to
    /// invoke the JIT-compiled native function pointer.
    ///
    /// # Errors
    ///
    /// Returns [`JitError::FunctionNotFound`] if no function with `name`
    /// exists, or [`JitError::TooManyArguments`] if more than 8 arguments are
    /// provided.
    pub fn call(&self, name: &str, args: &[i64]) -> Result<i64, JitError> {
        let addr = self
            .engine
            .get_function_address(name)
            .map_err(|_| JitError::FunctionNotFound(name.to_string()))?;

        if addr == 0 {
            return Err(JitError::FunctionNotFound(name.to_string()));
        }

        // All my_lang parameters and return values are i64.
        // We dispatch on the number of arguments at compile time so that the
        // C calling convention is respected.
        let result = unsafe {
            type F0 = unsafe extern "C" fn() -> i64;
            type F1 = unsafe extern "C" fn(i64) -> i64;
            type F2 = unsafe extern "C" fn(i64, i64) -> i64;
            type F3 = unsafe extern "C" fn(i64, i64, i64) -> i64;
            type F4 = unsafe extern "C" fn(i64, i64, i64, i64) -> i64;
            type F5 = unsafe extern "C" fn(i64, i64, i64, i64, i64) -> i64;
            type F6 = unsafe extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64;
            type F7 = unsafe extern "C" fn(i64, i64, i64, i64, i64, i64, i64) -> i64;
            type F8 = unsafe extern "C" fn(i64, i64, i64, i64, i64, i64, i64, i64) -> i64;

            match args {
                [] => std::mem::transmute::<usize, F0>(addr)(),
                [a0] => std::mem::transmute::<usize, F1>(addr)(*a0),
                [a0, a1] => std::mem::transmute::<usize, F2>(addr)(*a0, *a1),
                [a0, a1, a2] => std::mem::transmute::<usize, F3>(addr)(*a0, *a1, *a2),
                [a0, a1, a2, a3] => std::mem::transmute::<usize, F4>(addr)(*a0, *a1, *a2, *a3),
                [a0, a1, a2, a3, a4] => {
                    std::mem::transmute::<usize, F5>(addr)(*a0, *a1, *a2, *a3, *a4)
                }
                [a0, a1, a2, a3, a4, a5] => {
                    std::mem::transmute::<usize, F6>(addr)(*a0, *a1, *a2, *a3, *a4, *a5)
                }
                [a0, a1, a2, a3, a4, a5, a6] => {
                    std::mem::transmute::<usize, F7>(addr)(*a0, *a1, *a2, *a3, *a4, *a5, *a6)
                }
                [a0, a1, a2, a3, a4, a5, a6, a7] => {
                    std::mem::transmute::<usize, F8>(addr)(*a0, *a1, *a2, *a3, *a4, *a5, *a6, *a7)
                }
                _ => return Err(JitError::TooManyArguments),
            }
        };

        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_runner(source: &str) -> JitRunner<'_> {
        // The LLVM Context must outlive JitRunner.  In tests we intentionally
        // leak the context because:
        //   1. The test process is short-lived — the OS reclaims all memory on exit.
        //   2. Keeping a `Context` alive for the test's lifetime via `Box::leak`
        //      avoids threading lifetime parameters through every test function.
        // This is acceptable only in tests; production code should manage the
        // Context lifetime explicitly.
        let context = Box::leak(Box::new(Context::create()));
        JitRunner::new(context, source).expect("JitRunner::new should succeed")
    }

    // ------------------------------------------------------------------
    // Basic arithmetic
    // ------------------------------------------------------------------

    #[test]
    fn test_jit_no_args_returns_constant() {
        let runner = make_runner("function answer() { 42 }");
        // The body is an expression statement (not a return), so the function
        // returns the implicit 0.
        let result = runner.call("answer", &[]).expect("call should succeed");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_jit_explicit_return_constant() {
        let runner = make_runner("function answer() { return 42 }");
        let result = runner.call("answer", &[]).expect("call should succeed");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_jit_add_two_numbers() {
        let runner = make_runner("function add(a, b) { return a + b }");
        let result = runner.call("add", &[3, 4]).expect("call should succeed");
        assert_eq!(result, 7);
    }

    #[test]
    fn test_jit_subtract() {
        let runner = make_runner("function sub(a, b) { return a - b }");
        let result = runner.call("sub", &[10, 3]).expect("call should succeed");
        assert_eq!(result, 7);
    }

    #[test]
    fn test_jit_multiply() {
        let runner = make_runner("function mul(a, b) { return a * b }");
        let result = runner.call("mul", &[6, 7]).expect("call should succeed");
        assert_eq!(result, 42);
    }

    #[test]
    fn test_jit_divide() {
        let runner = make_runner("function div(a, b) { return a / b }");
        let result = runner.call("div", &[84, 2]).expect("call should succeed");
        assert_eq!(result, 42);
    }

    // ------------------------------------------------------------------
    // Variable assignment
    // ------------------------------------------------------------------

    #[test]
    fn test_jit_variable_assignment() {
        let src = r#"
            function compute() {
                x = 21
                return x + x
            }
        "#;
        let runner = make_runner(src);
        let result = runner.call("compute", &[]).expect("call should succeed");
        assert_eq!(result, 42);
    }

    // ------------------------------------------------------------------
    // Function calls
    // ------------------------------------------------------------------

    #[test]
    fn test_jit_cross_function_call() {
        let src = r#"
            function double(n) { return n + n }
            function quad(n)   { return double(double(n)) }
        "#;
        let runner = make_runner(src);
        assert_eq!(runner.call("double", &[5]).unwrap(), 10);
        assert_eq!(runner.call("quad", &[5]).unwrap(), 20);
    }

    // ------------------------------------------------------------------
    // Comparison
    // ------------------------------------------------------------------

    #[test]
    fn test_jit_comparison_equal() {
        let runner = make_runner("function eq(a, b) { return a == b }");
        assert_eq!(runner.call("eq", &[3, 3]).unwrap(), 1);
        assert_eq!(runner.call("eq", &[3, 4]).unwrap(), 0);
    }

    #[test]
    fn test_jit_comparison_less_than() {
        let runner = make_runner("function lt(a, b) { return a < b }");
        assert_eq!(runner.call("lt", &[2, 5]).unwrap(), 1);
        assert_eq!(runner.call("lt", &[5, 2]).unwrap(), 0);
    }

    // ------------------------------------------------------------------
    // Multi-argument functions (testing dispatch)
    // ------------------------------------------------------------------

    #[test]
    fn test_jit_three_args() {
        let runner = make_runner("function sum3(a, b, c) { return a + b + c }");
        assert_eq!(runner.call("sum3", &[1, 2, 3]).unwrap(), 6);
    }

    #[test]
    fn test_jit_four_args() {
        let runner = make_runner("function sum4(a, b, c, d) { return a + b + c + d }");
        assert_eq!(runner.call("sum4", &[1, 2, 3, 4]).unwrap(), 10);
    }

    // ------------------------------------------------------------------
    // Error cases
    // ------------------------------------------------------------------

    #[test]
    fn test_jit_function_not_found() {
        let runner = make_runner("function foo() { return 1 }");
        let result = runner.call("bar", &[]);
        assert!(matches!(result, Err(JitError::FunctionNotFound(_))));
    }

    #[test]
    fn test_jit_too_many_arguments() {
        let runner = make_runner("function foo() { return 1 }");
        let many_args = [0i64; 9];
        let result = runner.call("foo", &many_args);
        assert!(matches!(result, Err(JitError::TooManyArguments)));
    }

    #[test]
    fn test_jit_parse_error() {
        let context = Context::create();
        let result = JitRunner::new(&context, "this is not valid syntax !!!");
        assert!(matches!(result, Err(JitError::ParseError(_))));
    }
}
