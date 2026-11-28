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
    builder::{Builder, BuilderError},
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
    /// A `function` declaration whose body gives no value: it is empty, or
    /// ends in an assignment, with no `return` anywhere.
    FunctionWithoutValue(String),
    /// LLVM's instruction builder rejected an instruction.
    Builder(String),
    /// LLVM did not provide a parameter the function was declared with.
    MissingParameter {
        /// The function being compiled.
        function: String,
        /// The zero-based index of the missing parameter.
        index: usize,
    },
}

impl From<BuilderError> for CodegenError {
    fn from(err: BuilderError) -> Self {
        CodegenError::Builder(err.to_string())
    }
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
            CodegenError::FunctionWithoutValue(name) => {
                write!(
                    f,
                    "function '{name}' gives no value: add a 'return' or end its body with an expression"
                )
            }
            CodegenError::Builder(message) => {
                write!(f, "LLVM builder error: {message}")
            }
            CodegenError::MissingParameter { function, index } => {
                write!(f, "function '{function}' has no parameter at index {index}")
            }
        }
    }
}

/// Whether a body gives a value: it contains a `return`, or its last
/// statement is an expression.
fn body_gives_value(body: &[Statement]) -> bool {
    body.iter().any(|stmt| matches!(stmt, Statement::Return(_)))
        || matches!(body.last(), Some(Statement::Expression(_)))
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

    /// Consume the code generator and return the underlying LLVM [`Module`].
    ///
    /// Use this when you need to pass the module to another component (e.g.,
    /// a JIT execution engine) that requires ownership.
    pub fn take_module(self) -> Module<'ctx> {
        self.module
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
                // Pure functions must give a value: they cannot be void.
                if !body_gives_value(&f.body) {
                    return Err(CodegenError::FunctionWithoutValue(f.name.clone()));
                }
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
            let alloca = self.builder.build_alloca(i64_type, param_name)?;
            let param_val = u32::try_from(i)
                .ok()
                .and_then(|index| function.get_nth_param(index))
                .ok_or_else(|| CodegenError::MissingParameter {
                    function: name.to_string(),
                    index: i,
                })?
                .into_int_value();
            self.builder.build_store(alloca, param_val)?;
            locals.insert(param_name.clone(), alloca);
        }

        // Compile each statement.  In a declaration that may return a value,
        // a trailing expression statement is the value, as if it were
        // written `return <expression>`.
        let mut explicit_return = false;
        for (index, stmt) in body.iter().enumerate() {
            let is_last = index + 1 == body.len();
            if let (true, true, Statement::Expression(expr)) = (allows_return, is_last, stmt) {
                let val = self.compile_expression(expr, &locals)?;
                self.builder.build_return(Some(&val))?;
                explicit_return = true;
                break;
            }
            if self.compile_statement(stmt, &mut locals, allows_return)? {
                explicit_return = true;
                break; // Stop generating code after a return statement.
            }
        }

        // An `output` body with no return and no trailing expression (empty,
        // or ending in an assignment), and every `input` declaration, returns
        // 0. A `function` body like that is rejected before it gets here.
        if !explicit_return {
            let zero = i64_type.const_int(0, false);
            self.builder.build_return(Some(&zero))?;
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
                self.builder.build_return(Some(&val))?;
                Ok(true)
            }
            Statement::Assignment { name, value } => {
                let i64_type = self.context.i64_type();
                let val = self.compile_expression(value, locals)?;
                let alloca = match locals.get(name) {
                    Some(alloca) => *alloca,
                    None => {
                        let alloca = self.builder.build_alloca(i64_type, name)?;
                        locals.insert(name.clone(), alloca);
                        alloca
                    }
                };
                self.builder.build_store(alloca, val)?;
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
                let global_str = self.builder.build_global_string_ptr(s, &name)?;
                let i64_type = self.context.i64_type();
                let ptr_name = format!("{name}_ptr");
                Ok(self.builder.build_ptr_to_int(
                    global_str.as_pointer_value(),
                    i64_type,
                    &ptr_name,
                )?)
            }
            Expression::Identifier(name) => {
                let alloca = locals
                    .get(name)
                    .ok_or_else(|| CodegenError::UndefinedVariable(name.clone()))?;
                let i64_type = self.context.i64_type();
                Ok(self
                    .builder
                    .build_load(i64_type, *alloca, name)?
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
            BinaryOperator::Add => self.builder.build_int_add(lhs, rhs, "add")?,
            BinaryOperator::Subtract => self.builder.build_int_sub(lhs, rhs, "sub")?,
            BinaryOperator::Multiply => self.builder.build_int_mul(lhs, rhs, "mul")?,
            BinaryOperator::Divide => self.builder.build_int_signed_div(lhs, rhs, "div")?,
            BinaryOperator::Equal => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::EQ, lhs, rhs, "eq")?;
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "eq_ext")?
            }
            BinaryOperator::NotEqual => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::NE, lhs, rhs, "ne")?;
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "ne_ext")?
            }
            BinaryOperator::LessThan => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SLT, lhs, rhs, "lt")?;
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "lt_ext")?
            }
            BinaryOperator::GreaterThan => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SGT, lhs, rhs, "gt")?;
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "gt_ext")?
            }
            BinaryOperator::LessThanEqual => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SLE, lhs, rhs, "le")?;
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "le_ext")?
            }
            BinaryOperator::GreaterThanEqual => {
                let cmp = self
                    .builder
                    .build_int_compare(IntPredicate::SGE, lhs, rhs, "ge")?;
                self.builder
                    .build_int_z_extend(cmp, self.context.i64_type(), "ge_ext")?
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

        let call_result = self.builder.build_call(function, &compiled_args, "call")?;

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
    fn test_function_without_value_empty_body_is_an_error() {
        // A `function` cannot be void: an empty body names the function.
        let program = Program {
            declarations: vec![Declaration::Function(FunctionDecl {
                name: "noop".to_string(),
                parameters: vec![],
                body: vec![],
            })],
        };
        let err = compile_program_to_ir(&program).unwrap_err();
        assert_eq!(err, CodegenError::FunctionWithoutValue("noop".to_string()));
        assert!(err.to_string().contains("'noop'"));
    }

    #[test]
    fn test_output_with_empty_body_gives_zero() {
        // `output` declarations may still end without a value.
        let program = Program {
            declarations: vec![Declaration::Output(OutputDecl {
                name: "flush".to_string(),
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

    // ------------------------------------------------------------------
    // LLVM builder errors are returned, not panicked on
    // ------------------------------------------------------------------

    #[test]
    fn test_builder_error_converts_to_codegen_error() {
        let err = CodegenError::from(BuilderError::UnsetPosition);
        assert_eq!(
            err,
            CodegenError::Builder("Builder position is not set".to_string())
        );
        assert_eq!(
            err.to_string(),
            "LLVM builder error: Builder position is not set"
        );
    }

    #[test]
    fn test_missing_parameter_error_message() {
        let err = CodegenError::MissingParameter {
            function: "f".to_string(),
            index: 2,
        };
        assert_eq!(err.to_string(), "function 'f' has no parameter at index 2");
    }
}
