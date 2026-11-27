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
    /// The function was called with a different number of arguments than it
    /// declares.
    ArgumentCountMismatch {
        /// The number of parameters the function declares.
        expected: usize,
        /// The number of arguments passed to [`JitRunner::call`].
        given: usize,
    },
}

impl std::fmt::Display for JitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JitError::ParseError(msg) => write!(f, "parse error: {msg}"),
            JitError::CodegenError(e) => write!(f, "codegen error: {e}"),
            JitError::JitCreationFailed(msg) => write!(f, "JIT creation failed: {msg}"),
            JitError::FunctionNotFound(name) => write!(f, "function not found: '{name}'"),
            JitError::TooManyArguments => write!(f, "too many arguments (max 8 supported)"),
            JitError::ArgumentCountMismatch { expected, given } => {
                write!(f, "expected {expected} arguments, got {given}")
            }
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
    /// This function is safe to call. Internally it uses the crate's only
    /// `unsafe` block to invoke the JIT-compiled native function pointer,
    /// after checking that `args` matches the function's declared arity.
    ///
    /// # Errors
    ///
    /// Returns [`JitError::FunctionNotFound`] if no function with `name`
    /// exists, [`JitError::TooManyArguments`] if more than 8 arguments are
    /// provided, or [`JitError::ArgumentCountMismatch`] if the number of
    /// arguments differs from the function's parameter count.
    pub fn call(&self, name: &str, args: &[i64]) -> Result<i64, JitError> {
        let addr = self
            .engine
            .get_function_address(name)
            .map_err(|_| JitError::FunctionNotFound(name.to_string()))?;

        if addr == 0 {
            return Err(JitError::FunctionNotFound(name.to_string()));
        }

        if args.len() > 8 {
            return Err(JitError::TooManyArguments);
        }

        // Calling a native function with the wrong number of arguments is
        // undefined behaviour, so check the declared arity first.
        let expected = self
            .engine
            .get_function_value(name)
            .map_err(|_| JitError::FunctionNotFound(name.to_string()))?
            .count_params() as usize;
        if args.len() != expected {
            return Err(JitError::ArgumentCountMismatch {
                expected,
                given: args.len(),
            });
        }

        // All my_lang parameters and return values are i64.
        // We dispatch on the number of arguments at compile time so that the
        // C calling convention is respected.
        //
        // SAFETY: `addr` is the non-zero address LLVM's execution engine gave
        // for `name`, a function the code generator emitted with the C calling
        // convention, `args.len()` parameters of type i64 (checked above) and
        // an i64 return. `self.engine` owns the machine code and lives as long
        // as `self`, so the pointer stays valid for the call. Every my_lang
        // function only does integer arithmetic and calls other functions in
        // the same module, so the call itself can't violate memory safety.
        #[allow(unsafe_code)]
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
                // Unreachable: more than 8 arguments was rejected above.
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
    fn test_jit_too_few_arguments_is_an_error() {
        let runner = make_runner("function add(a, b) { a + b }");
        let result = runner.call("add", &[1]);
        assert!(matches!(
            result,
            Err(JitError::ArgumentCountMismatch {
                expected: 2,
                given: 1
            })
        ));
    }

    #[test]
    fn test_jit_extra_arguments_are_an_error() {
        let runner = make_runner("function answer() { return 42 }");
        let result = runner.call("answer", &[1, 2]);
        assert!(matches!(
            result,
            Err(JitError::ArgumentCountMismatch {
                expected: 0,
                given: 2
            })
        ));
    }

    #[test]
    fn test_argument_count_mismatch_message() {
        let err = JitError::ArgumentCountMismatch {
            expected: 2,
            given: 1,
        };
        assert_eq!(err.to_string(), "expected 2 arguments, got 1");
    }

    #[test]
    fn test_jit_parse_error() {
        let context = Context::create();
        let result = JitRunner::new(&context, "this is not valid syntax !!!");
        assert!(matches!(result, Err(JitError::ParseError(_))));
    }
}
