//! Code generator module for my_lang
//!
//! This module compiles a parsed AST into LLVM IR using the [inkwell] library,
//! which provides safe Rust bindings to the LLVM C API.
//!
//! ## Architecture
//!
//! The code generator works in a single pass over the AST:
//!
//! 1. Each top-level declaration (function / input / output) is compiled into
//!    an LLVM function.
//! 2. Statements inside a body are lowered into LLVM basic-block instructions.
//! 3. Expressions are recursively compiled into LLVM values.
//!
//! ## Type System
//!
//! All values are currently represented as 64-bit signed integers (`i64`).
//! String literals are compiled as global byte-array constants; the address
//! of the first byte is returned as an `i64` via a `ptrtoint` instruction.
//!
//! ## LLVM Setup
//!
//! The caller is responsible for creating an [`inkwell::context::Context`] and
//! passing a reference to [`CodeGenerator::new`].  The generated module can be
//! retrieved via [`CodeGenerator::module`] and then used for JIT compilation,
//! verification, or IR printing.
//!
//! ## Example
//!
//! ```ignore
//! use inkwell::context::Context;
//! use my_lang::codegen::CodeGenerator;
//! use my_lang::lexer::Lexer;
//! use my_lang::parser::Parser;
//!
//! let source = "function add(a, b) { a + b }".to_string();
//! let lexer = Lexer::new(source);
//! let mut parser = Parser::new(lexer);
//! let program = parser.parse().unwrap();
//!
//! let ctx = Context::create();
//! let mut codegen = CodeGenerator::new(&ctx, "my_module");
//! codegen.compile_program(&program).unwrap();
//! let ir = codegen.module().print_to_string().to_string();
//! ```

use std::collections::HashMap;

use inkwell::{
    IntPredicate,
    builder::Builder,
    context::Context,
    module::Module,
    values::{FunctionValue, IntValue, PointerValue},
};

use crate::parser::{BinaryOperator, Declaration, Expression, Program, Statement};

/// Errors that can occur during code generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodegenError {
    /// An identifier was used before it was defined.
    UndefinedVariable(String),
    /// A call to an unknown function was encountered.
    UndefinedFunction(String),
    /// A `return` statement was found inside an `input` declaration.
    ReturnInInputDeclaration,
    /// An expression statement that produces no value was found where a value
    /// was required.
    ExpressionWithoutValue,
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::UndefinedVariable(name) => {
                write!(f, "undefined variable: '{name}'")
            }
            CodegenError::UndefinedFunction(name) => {
                write!(f, "undefined function: '{name}'")
            }
            CodegenError::ReturnInInputDeclaration => {
                write!(
                    f,
                    "return statement is not allowed inside an 'input' declaration"
                )
            }
            CodegenError::ExpressionWithoutValue => {
                write!(f, "expression statement produced no value")
            }
        }
    }
}

/// LLVM-backed code generator for my_lang.
///
/// Holds a reference to a [`Context`] together with the [`Module`] and
/// [`Builder`] used during compilation.  The lifetime `'ctx` ties the
/// generator to the context that owns all LLVM objects.
pub struct CodeGenerator<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    /// Counter used to generate unique names for global string constants.
    string_counter: usize,
}

impl<'ctx> CodeGenerator<'ctx> {
    /// Create a new code generator bound to `context`.
    ///
    /// `module_name` is the name embedded in the LLVM module (visible in the
    /// printed IR and error messages).
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            string_counter: 0,
        }
    }

    /// Return a reference to the generated LLVM [`Module`].
    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    /// Compile a complete [`Program`] into LLVM IR.
    ///
    /// Each declaration is compiled into an LLVM function.  The generated
    /// code is accumulated in the module held by `self`.
    pub fn compile_program(&mut self, program: &Program) -> Result<(), CodegenError> {
        for decl in &program.declarations {
            self.compile_declaration(decl)?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Declaration compilation
    // -----------------------------------------------------------------------

    fn compile_declaration(
        &mut self,
        decl: &Declaration,
    ) -> Result<FunctionValue<'ctx>, CodegenError> {
        match decl {
            Declaration::Function(f) => {
                // Pure functions must return a value.
                self.compile_function_body(&f.name, &f.parameters, &f.body, true)
            }
            Declaration::Input(i) => {
                // Input declarations do not allow return statements.
                self.compile_function_body(&i.name, &i.parameters, &i.body, false)
            }
            Declaration::Output(o) => {
                self.compile_function_body(&o.name, &o.parameters, &o.body, true)
            }
        }
    }

    /// Compile a function body into an LLVM function.
    ///
    /// All parameters are treated as `i64`.  Local variables are stored on
    /// the stack via `alloca` instructions so that assignments are supported.
    ///
    /// `allows_return` controls whether a `return` statement is valid inside
    /// this function.  When `false` (i.e. for `input` declarations) a
    /// `CodegenError::ReturnInInputDeclaration` is returned instead.
    fn compile_function_body(
        &mut self,
        name: &str,
        parameters: &[String],
        body: &[Statement],
        allows_return: bool,
    ) -> Result<FunctionValue<'ctx>, CodegenError> {
        let i64_type = self.context.i64_type();

        // Build the LLVM function type: all params are i64, return type is i64.
        let param_types: Vec<_> = parameters.iter().map(|_| i64_type.into()).collect();
        let fn_type = i64_type.fn_type(&param_types, false);
        let function = self.module.add_function(name, fn_type, None);

        let entry_block = self.context.append_basic_block(function, "entry");
        self.builder.position_at_end(entry_block);

        // Allocate stack slots for each parameter and store the initial value.
        let mut locals: HashMap<String, PointerValue<'ctx>> = HashMap::new();
        for (i, param_name) in parameters.iter().enumerate() {
            let alloca = self.builder.build_alloca(i64_type, param_name).unwrap();
            let param_val = function.get_nth_param(i as u32).unwrap().into_int_value();
            self.builder.build_store(alloca, param_val).unwrap();
            locals.insert(param_name.clone(), alloca);
        }

        // Compile each statement.
        let mut explicit_return = false;
        for stmt in body {
            if self.compile_statement(stmt, &mut locals, allows_return)? {
                explicit_return = true;
                break; // Stop generating code after a return statement.
            }
        }

        // If no explicit return was emitted, return 0 as a default value.
        if !explicit_return {
            let zero = i64_type.const_int(0, false);
            self.builder.build_return(Some(&zero)).unwrap();
        }

        Ok(function)
    }

    // -----------------------------------------------------------------------
    // Statement compilation
    // -----------------------------------------------------------------------

    /// Compile a single statement.
    ///
    /// Returns `true` if the statement was a `return` (i.e. no more code
    /// should be emitted in the current basic block).
    fn compile_statement(
        &mut self,
        stmt: &Statement,
        locals: &mut HashMap<String, PointerValue<'ctx>>,
        allows_return: bool,
    ) -> Result<bool, CodegenError> {
        match stmt {
            Statement::Return(expr) => {
                if !allows_return {
                    return Err(CodegenError::ReturnInInputDeclaration);
                }
                let val = self.compile_expression(expr, locals)?;
                self.builder.build_return(Some(&val)).unwrap();
                Ok(true)
            }
            Statement::Assignment { name, value } => {
                let i64_type = self.context.i64_type();
                let val = self.compile_expression(value, locals)?;
                let alloca = locals
                    .entry(name.clone())
                    .or_insert_with(|| self.builder.build_alloca(i64_type, name).unwrap());
                self.builder.build_store(*alloca, val).unwrap();
                Ok(false)
            }
            Statement::Expression(expr) => {
                // Evaluate for side effects (e.g. function calls).
                self.compile_expression(expr, locals)?;
                Ok(false)
            }
        }
    }

    // -----------------------------------------------------------------------
    // Expression compilation
    // -----------------------------------------------------------------------

    fn compile_expression(
        &mut self,
        expr: &Expression,
        locals: &HashMap<String, PointerValue<'ctx>>,
    ) -> Result<IntValue<'ctx>, CodegenError> {
        match expr {
            Expression::Integer(n) => {
                let i64_type = self.context.i64_type();
                // Use sign_extend = true so negative values are handled correctly.
                Ok(i64_type.const_int(*n as u64, true))
            }
            Expression::StringLiteral(s) => {
                let name = format!("str_{}", self.string_counter);
                self.string_counter += 1;
                let global_str = self.builder.build_global_string_ptr(s, &name).unwrap();
                let i64_type = self.context.i64_type();
                let ptr_name = format!("{name}_ptr");
                Ok(self
                    .builder
                    .build_ptr_to_int(global_str.as_pointer_value(), i64_type, &ptr_name)
                    .unwrap())
            }
            Expression::Identifier(name) => {
                let alloca = locals
                    .get(name)
                    .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))?;
                let i64_type = self.context.i64_type();
                Ok(self
                    .builder
                    .build_load(i64_type, *alloca, name)
                    .unwrap()
                    .into_int_value())
            }
            Expression::Binary {
                left,
                operator,
                right,
            } => self.compile_binary(left, operator, right, locals),
            Expression::Call {
                function,
                arguments,
            } => self.compile_call(function, arguments, locals),
        }
    }

    fn compile_binary(
        &mut self,
        left: &Expression,
        operator: &BinaryOperator,
        right: &Expression,
        locals: &HashMap<String, PointerValue<'ctx>>,
    ) -> Result<IntValue<'ctx>, CodegenError> {
        let lhs = self.compile_expression(left, locals)?;
        let rhs = self.compile_expression(right, locals)?;

        let result = match operator {
            BinaryOperator::Add => self.builder.build_int_add(lhs, rhs, "add").unwrap(),
            BinaryOperator::Subtract => self.builder.build_int_sub(lhs, rhs, "sub").unwrap(),
            BinaryOperator::Multiply => self.builder.build_int_mul(lhs, rhs, "mul").unwrap(),
            BinaryOperator::Divide => self.builder.build_int_signed_div(lhs, rhs, "div").unwrap(),
            BinaryOperator::Equal => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq")
                    .unwrap();
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "eq_ext")
                    .unwrap()
            }
            BinaryOperator::NotEqual => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::NE, lhs, rhs, "ne")
                    .unwrap();
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "ne_ext")
                    .unwrap()
            }
            BinaryOperator::LessThan => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt")
                    .unwrap();
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "lt_ext")
                    .unwrap()
            }
            BinaryOperator::GreaterThan => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt")
                    .unwrap();
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "gt_ext")
                    .unwrap()
            }
            BinaryOperator::LessThanEqual => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SLE, lhs, rhs, "le")
                    .unwrap();
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "le_ext")
                    .unwrap()
            }
            BinaryOperator::GreaterThanEqual => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge")
                    .unwrap();
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "ge_ext")
                    .unwrap()
            }
        };
        Ok(result)
    }

    fn compile_call(
        &mut self,
        function_name: &str,
        arguments: &[Expression],
        locals: &HashMap<String, PointerValue<'ctx>>,
    ) -> Result<IntValue<'ctx>, CodegenError> {
        let function = self
            .module
            .get_function(function_name)
            .ok_or_else(|| CodegenError::UndefinedFunction(function_name.to_string()))?;

        let mut compiled_args = Vec::with_capacity(arguments.len());
        for arg in arguments {
            let val = self.compile_expression(arg, locals)?;
            compiled_args.push(val.into());
        }

        let call_result = self
            .builder
            .build_call(function, &compiled_args, "call")
            .unwrap();

        let return_val = call_result
            .try_as_basic_value()
            .basic()
            .ok_or(CodegenError::ExpressionWithoutValue)?
            .into_int_value();

        Ok(return_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        BinaryOperator, Declaration, Expression, FunctionDecl, InputDecl, OutputDecl, Program,
        Statement,
    };
    use inkwell::context::Context;

    // ------------------------------------------------------------------
    // Helper: build a single-function program and return the LLVM IR string.
    // ------------------------------------------------------------------

    fn compile_program_to_ir(program: &Program) -> Result<String, CodegenError> {
        let context = Context::create();
        let mut codegen = CodeGenerator::new(&context, "test_module");
        codegen.compile_program(program)?;
        let ir = codegen.module().print_to_string().to_string();
        Ok(ir)
    }

    // ------------------------------------------------------------------
    // Integer literals
    // ------------------------------------------------------------------

    #[test]
    fn test_integer_literal() {
        // function const42() { 42 }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "const42".to_string(),
                parameters: vec![],
                body: vec![Statement::Expression(Expression::Integer(42))],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("const42"), "IR should contain function name");
        assert!(ir.contains("i64"), "IR should use i64 type");
    }

    #[test]
    fn test_integer_literal_return() {
        // function seven() { return 7 }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "seven".to_string(),
                parameters: vec![],
                body: vec![Statement::Return(Expression::Integer(7))],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("seven"));
        assert!(ir.contains("ret i64 7"));
    }

    // ------------------------------------------------------------------
    // Arithmetic operations
    // ------------------------------------------------------------------

    #[test]
    fn test_addition() {
        // function add(a, b) { a + b }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "add".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Return(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("add"), "IR should contain function 'add'");
        assert!(
            ir.contains("add i64"),
            "IR should contain integer add instruction"
        );
    }

    #[test]
    fn test_subtraction() {
        // function sub(a, b) { a - b }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "sub".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Return(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Subtract,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("sub i64"));
    }

    #[test]
    fn test_multiplication() {
        // function mul(a, b) { a * b }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "mul".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Return(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Multiply,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("mul i64"));
    }

    #[test]
    fn test_division() {
        // function div(a, b) { a / b }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "div".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Return(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Divide,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("sdiv"));
    }

    // ------------------------------------------------------------------
    // Comparison operators
    // ------------------------------------------------------------------

    #[test]
    fn test_equality() {
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "eq".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Return(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::Equal,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("icmp eq"));
    }

    #[test]
    fn test_less_than() {
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "lt".to_string(),
                parameters: vec!["a".to_string(), "b".to_string()],
                body: vec![Statement::Return(Expression::Binary {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    operator: BinaryOperator::LessThan,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("icmp slt"));
    }

    // ------------------------------------------------------------------
    // Function declarations and parameters
    // ------------------------------------------------------------------

    #[test]
    fn test_function_with_parameters() {
        // function identity(x) { x }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "identity".to_string(),
                parameters: vec!["x".to_string()],
                body: vec![Statement::Return(Expression::Identifier("x".to_string()))],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("identity"));
        assert!(ir.contains("i64 %"));
    }

    #[test]
    fn test_function_no_explicit_return() {
        // A body with no return statement should implicitly return 0.
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "noop".to_string(),
                parameters: vec![],
                body: vec![],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("ret i64 0"));
    }

    // ------------------------------------------------------------------
    // Variable assignment
    // ------------------------------------------------------------------

    #[test]
    fn test_variable_assignment_and_use() {
        // function compute() { x = 10; x + 5 }
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "compute".to_string(),
                parameters: vec![],
                body: vec![
                    Statement::Assignment {
                        name: "x".to_string(),
                        value: Expression::Integer(10),
                    },
                    Statement::Return(Expression::Binary {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        operator: BinaryOperator::Add,
                        right: Box::new(Expression::Integer(5)),
                    }),
                ],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("compute"));
        assert!(ir.contains("alloca"));
        assert!(ir.contains("store"));
    }

    // ------------------------------------------------------------------
    // Function calls
    // ------------------------------------------------------------------

    #[test]
    fn test_function_call() {
        // function double(x) { x + x }
        // function quad(x)   { double(double(x)) }
        let program = Program {
            declarations: vec![
                Declaration::Function(FunctionDecl {
                    name: "double".to_string(),
                    parameters: vec!["x".to_string()],
                    body: vec![Statement::Return(Expression::Binary {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        operator: BinaryOperator::Add,
                        right: Box::new(Expression::Identifier("x".to_string())),
                    })],
                }),
                Declaration::Function(FunctionDecl {
                    name: "quad".to_string(),
                    parameters: vec!["x".to_string()],
                    body: vec![Statement::Return(Expression::Call {
                        function: "double".to_string(),
                        arguments: vec![Expression::Call {
                            function: "double".to_string(),
                            arguments: vec![Expression::Identifier("x".to_string())],
                        }],
                    })],
                }),
            ],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("call i64 @double"));
    }

    // ------------------------------------------------------------------
    // Input / Output declarations
    // ------------------------------------------------------------------

    #[test]
    fn test_input_declaration_compiled() {
        let program = Program {
            declarations: vec![Declaration::Input(InputDecl {
                name: "read_value".to_string(),
                parameters: vec![],
                body: vec![],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("read_value"));
    }

    #[test]
    fn test_output_declaration_allows_return() {
        let program = Program {
            declarations: vec![Declaration::Output(OutputDecl {
                name: "write_value".to_string(),
                parameters: vec!["v".to_string()],
                body: vec![Statement::Return(Expression::Identifier("v".to_string()))],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("write_value"));
        assert!(ir.contains("ret i64"));
    }

    #[test]
    fn test_input_declaration_return_error() {
        // `input` declarations must not contain `return` statements.
        let program = Program {
            declarations: vec![Declaration::Input(InputDecl {
                name: "bad_input".to_string(),
                parameters: vec![],
                body: vec![Statement::Return(Expression::Integer(0))],
            })],
        };
        let context = Context::create();
        let mut codegen = CodeGenerator::new(&context, "test_module");
        let result = codegen.compile_program(&program);
        assert_eq!(result, Err(CodegenError::ReturnInInputDeclaration));
    }

    // ------------------------------------------------------------------
    // Error cases
    // ------------------------------------------------------------------

    #[test]
    fn test_undefined_variable_error() {
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "bad".to_string(),
                parameters: vec![],
                body: vec![Statement::Return(Expression::Identifier(
                    "nonexistent".to_string(),
                ))],
            })],
        };
        let context = Context::create();
        let mut codegen = CodeGenerator::new(&context, "test_module");
        let result = codegen.compile_program(&program);
        assert_eq!(
            result,
            Err(CodegenError::UndefinedVariable("nonexistent".to_string()))
        );
    }

    #[test]
    fn test_undefined_function_error() {
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "caller".to_string(),
                parameters: vec![],
                body: vec![Statement::Return(Expression::Call {
                    function: "ghost".to_string(),
                    arguments: vec![],
                })],
            })],
        };
        let context = Context::create();
        let mut codegen = CodeGenerator::new(&context, "test_module");
        let result = codegen.compile_program(&program);
        assert_eq!(
            result,
            Err(CodegenError::UndefinedFunction("ghost".to_string()))
        );
    }

    #[test]
    fn test_string_literal_as_pointer() {
        // String literals are compiled as global constants; the address is
        // returned as an i64 via a ptrtoint instruction.
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "get_greeting".to_string(),
                parameters: vec![],
                body: vec![Statement::Return(Expression::StringLiteral(
                    "hello".to_string(),
                ))],
            })],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(
            ir.contains("get_greeting"),
            "IR should define 'get_greeting'"
        );
        assert!(
            ir.contains("ptrtoint"),
            "IR should convert string pointer to i64"
        );
        assert!(
            ir.contains("hello"),
            "IR should contain the string literal data"
        );
    }

    // ------------------------------------------------------------------
    // Multiple declarations
    // ------------------------------------------------------------------

    #[test]
    fn test_multiple_functions() {
        let program = Program {
            declarations: vec![
                Declaration::Function(FunctionDecl {
                    name: "inc".to_string(),
                    parameters: vec!["n".to_string()],
                    body: vec![Statement::Return(Expression::Binary {
                        left: Box::new(Expression::Identifier("n".to_string())),
                        operator: BinaryOperator::Add,
                        right: Box::new(Expression::Integer(1)),
                    })],
                }),
                Declaration::Function(FunctionDecl {
                    name: "dec".to_string(),
                    parameters: vec!["n".to_string()],
                    body: vec![Statement::Return(Expression::Binary {
                        left: Box::new(Expression::Identifier("n".to_string())),
                        operator: BinaryOperator::Subtract,
                        right: Box::new(Expression::Integer(1)),
                    })],
                }),
            ],
        };
        let ir = compile_program_to_ir(&program).unwrap();
        assert!(ir.contains("inc"));
        assert!(ir.contains("dec"));
    }
}
