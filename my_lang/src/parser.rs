//! Parser module for my_lang
//!
//! The parser builds an Abstract Syntax Tree (AST) from the token stream
//! produced by the lexer. This is the second phase of compilation.
//!
//! ## AST Node Types
//!
//! The AST represents the structure of my_lang programs:
//! - **Program**: Root node containing a list of declarations
//! - **Declarations**: Function, Input, and Output declarations
//! - **Statements**: Return, Assignment, and Expression statements
//! - **Expressions**: Literals, identifiers, binary operations, and function calls
//!
//! ## Language Semantics
//!
//! - `function` declarations define pure functions (must return a value, no side effects)
//! - `input` declarations allow input side effects (no return statements)
//! - `output` declarations allow output side effects (allows return statements)

use crate::lexer::Lexer;

/// Root of the Abstract Syntax Tree representing a complete program
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    /// List of declarations in the program (functions, inputs, outputs)
    pub declarations: Vec<Declaration>,
}

/// Top-level declarations in a program
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Declaration {
    /// Pure function declaration (no side effects, must return)
    Function(FunctionDecl),
    /// Input function declaration (allows input side effects, no return)
    Input(InputDecl),
    /// Output function declaration (allows output side effects, allows return)
    Output(OutputDecl),
}

/// Function declaration for pure functions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    /// Function name
    pub name: String,
    /// List of parameter names
    pub parameters: Vec<String>,
    /// Function body (list of statements)
    pub body: Vec<Statement>,
}

/// Input declaration for functions with input side effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputDecl {
    /// Input function name
    pub name: String,
    /// List of parameter names
    pub parameters: Vec<String>,
    /// Function body (list of statements, no return allowed)
    pub body: Vec<Statement>,
}

/// Output declaration for functions with output side effects
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputDecl {
    /// Output function name
    pub name: String,
    /// List of parameter names
    pub parameters: Vec<String>,
    /// Function body (list of statements, return allowed)
    pub body: Vec<Statement>,
}

/// Statements that can appear in function bodies
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    /// Return statement with an expression
    Return(Expression),
    /// Assignment statement (identifier = expression)
    Assignment {
        /// Variable name being assigned to
        name: String,
        /// Expression to assign
        value: Expression,
    },
    /// Expression statement (expression evaluated for side effects)
    Expression(Expression),
}

/// Expressions that produce values
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    /// Integer literal
    Integer(i64),
    /// String literal
    StringLiteral(String),
    /// Variable or function identifier
    Identifier(String),
    /// Binary operation (e.g., a + b, x == y)
    Binary {
        /// Left operand
        left: Box<Expression>,
        /// Operator
        operator: BinaryOperator,
        /// Right operand
        right: Box<Expression>,
    },
    /// Function call
    Call {
        /// Function name
        function: String,
        /// List of argument expressions
        arguments: Vec<Expression>,
    },
}

/// Binary operators supported in expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Subtract,
    /// Multiplication (*)
    Multiply,
    /// Division (/)
    Divide,
    /// Equality (==)
    Equal,
    /// Inequality (!=)
    NotEqual,
    /// Less than (<)
    LessThan,
    /// Greater than (>)
    GreaterThan,
    /// Less than or equal (<=)
    LessThanEqual,
    /// Greater than or equal (>=)
    GreaterThanEqual,
}

/// Placeholder AST node type (deprecated - use Program instead)
#[deprecated(since = "0.1.0", note = "Use Program struct instead")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstNode {
    /// Empty program
    Program,
}

/// Parser structure (not yet implemented)
///
/// Future implementation will parse tokens into an AST
pub struct Parser {
    _lexer: Lexer,
}

impl Parser {
    /// Create a new parser with the given lexer
    pub fn new(lexer: Lexer) -> Self {
        Self { _lexer: lexer }
    }

    /// Parse the input and produce an AST
    ///
    /// This is a placeholder - actual parsing logic to be implemented
    #[allow(deprecated)]
    pub fn parse(&mut self) -> AstNode {
        AstNode::Program
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for AST node type construction

    #[test]
    fn test_integer_expression() {
        let expr = Expression::Integer(42);
        assert_eq!(expr, Expression::Integer(42));
    }

    #[test]
    fn test_string_literal_expression() {
        let expr = Expression::StringLiteral("hello".to_string());
        assert_eq!(expr, Expression::StringLiteral("hello".to_string()));
    }

    #[test]
    fn test_identifier_expression() {
        let expr = Expression::Identifier("x".to_string());
        assert_eq!(expr, Expression::Identifier("x".to_string()));
    }

    #[test]
    fn test_binary_expression() {
        // Represents: 5 + 3
        let expr = Expression::Binary {
            left: Box::new(Expression::Integer(5)),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Integer(3)),
        };

        match expr {
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(*left, Expression::Integer(5));
                assert_eq!(operator, BinaryOperator::Add);
                assert_eq!(*right, Expression::Integer(3));
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_nested_binary_expression() {
        // Represents: (2 + 3) * 4
        let expr = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Integer(2)),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Integer(3)),
            }),
            operator: BinaryOperator::Multiply,
            right: Box::new(Expression::Integer(4)),
        };

        match expr {
            Expression::Binary {
                left,
                operator,
                right,
            } => {
                assert_eq!(operator, BinaryOperator::Multiply);
                assert_eq!(*right, Expression::Integer(4));
                match *left {
                    Expression::Binary {
                        left: l2,
                        operator: op2,
                        right: r2,
                    } => {
                        assert_eq!(*l2, Expression::Integer(2));
                        assert_eq!(op2, BinaryOperator::Add);
                        assert_eq!(*r2, Expression::Integer(3));
                    }
                    _ => panic!("Expected nested Binary expression"),
                }
            }
            _ => panic!("Expected Binary expression"),
        }
    }

    #[test]
    fn test_function_call_expression() {
        // Represents: add(1, 2)
        let expr = Expression::Call {
            function: "add".to_string(),
            arguments: vec![Expression::Integer(1), Expression::Integer(2)],
        };

        match expr {
            Expression::Call {
                function,
                arguments,
            } => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2);
                assert_eq!(arguments[0], Expression::Integer(1));
                assert_eq!(arguments[1], Expression::Integer(2));
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    fn test_return_statement() {
        let stmt = Statement::Return(Expression::Integer(42));
        match stmt {
            Statement::Return(expr) => {
                assert_eq!(expr, Expression::Integer(42));
            }
            _ => panic!("Expected Return statement"),
        }
    }

    #[test]
    fn test_assignment_statement() {
        // Represents: x = 10
        let stmt = Statement::Assignment {
            name: "x".to_string(),
            value: Expression::Integer(10),
        };

        match stmt {
            Statement::Assignment { name, value } => {
                assert_eq!(name, "x");
                assert_eq!(value, Expression::Integer(10));
            }
            _ => panic!("Expected Assignment statement"),
        }
    }

    #[test]
    fn test_expression_statement() {
        // Represents an expression evaluated for side effects
        let stmt = Statement::Expression(Expression::Call {
            function: "print".to_string(),
            arguments: vec![Expression::StringLiteral("hello".to_string())],
        });

        match stmt {
            Statement::Expression(expr) => match expr {
                Expression::Call {
                    function,
                    arguments,
                } => {
                    assert_eq!(function, "print");
                    assert_eq!(arguments.len(), 1);
                }
                _ => panic!("Expected Call expression"),
            },
            _ => panic!("Expected Expression statement"),
        }
    }

    #[test]
    fn test_function_declaration() {
        // Represents: function add(a, b) { a + b }
        let decl = Declaration::Function(FunctionDecl {
            name: "add".to_string(),
            parameters: vec!["a".to_string(), "b".to_string()],
            body: vec![Statement::Return(Expression::Binary {
                left: Box::new(Expression::Identifier("a".to_string())),
                operator: BinaryOperator::Add,
                right: Box::new(Expression::Identifier("b".to_string())),
            })],
        });

        match decl {
            Declaration::Function(func) => {
                assert_eq!(func.name, "add");
                assert_eq!(func.parameters.len(), 2);
                assert_eq!(func.parameters[0], "a");
                assert_eq!(func.parameters[1], "b");
                assert_eq!(func.body.len(), 1);
            }
            _ => panic!("Expected Function declaration"),
        }
    }

    #[test]
    fn test_input_declaration() {
        // Represents: input readValue() { ... }
        let decl = Declaration::Input(InputDecl {
            name: "readValue".to_string(),
            parameters: vec![],
            body: vec![Statement::Assignment {
                name: "x".to_string(),
                value: Expression::Integer(5),
            }],
        });

        match decl {
            Declaration::Input(input) => {
                assert_eq!(input.name, "readValue");
                assert_eq!(input.parameters.len(), 0);
                assert_eq!(input.body.len(), 1);
            }
            _ => panic!("Expected Input declaration"),
        }
    }

    #[test]
    fn test_output_declaration() {
        // Represents: output writeValue(x) { ... }
        let decl = Declaration::Output(OutputDecl {
            name: "writeValue".to_string(),
            parameters: vec!["x".to_string()],
            body: vec![
                Statement::Expression(Expression::Call {
                    function: "print".to_string(),
                    arguments: vec![Expression::Identifier("x".to_string())],
                }),
                Statement::Return(Expression::Integer(0)),
            ],
        });

        match decl {
            Declaration::Output(output) => {
                assert_eq!(output.name, "writeValue");
                assert_eq!(output.parameters.len(), 1);
                assert_eq!(output.parameters[0], "x");
                assert_eq!(output.body.len(), 2);
            }
            _ => panic!("Expected Output declaration"),
        }
    }

    #[test]
    fn test_program_structure() {
        // Represents a program with multiple declarations
        let program = Program {
            declarations: vec![
                Declaration::Function(FunctionDecl {
                    name: "add".to_string(),
                    parameters: vec!["a".to_string(), "b".to_string()],
                    body: vec![Statement::Return(Expression::Binary {
                        left: Box::new(Expression::Identifier("a".to_string())),
                        operator: BinaryOperator::Add,
                        right: Box::new(Expression::Identifier("b".to_string())),
                    })],
                }),
                Declaration::Output(OutputDecl {
                    name: "main".to_string(),
                    parameters: vec![],
                    body: vec![Statement::Expression(Expression::Call {
                        function: "add".to_string(),
                        arguments: vec![Expression::Integer(1), Expression::Integer(2)],
                    })],
                }),
            ],
        };

        assert_eq!(program.declarations.len(), 2);
    }

    #[test]
    fn test_all_binary_operators() {
        let operators = vec![
            BinaryOperator::Add,
            BinaryOperator::Subtract,
            BinaryOperator::Multiply,
            BinaryOperator::Divide,
            BinaryOperator::Equal,
            BinaryOperator::NotEqual,
            BinaryOperator::LessThan,
            BinaryOperator::GreaterThan,
            BinaryOperator::LessThanEqual,
            BinaryOperator::GreaterThanEqual,
        ];

        // Verify all operators can be constructed and compared
        for op in operators {
            let expr = Expression::Binary {
                left: Box::new(Expression::Integer(1)),
                operator: op.clone(),
                right: Box::new(Expression::Integer(2)),
            };

            match expr {
                Expression::Binary { operator, .. } => {
                    assert_eq!(operator, op);
                }
                _ => panic!("Expected Binary expression"),
            }
        }
    }

    #[test]
    fn test_complex_expression_tree() {
        // Represents: add(x * 2, y + 3)
        let expr = Expression::Call {
            function: "add".to_string(),
            arguments: vec![
                Expression::Binary {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    operator: BinaryOperator::Multiply,
                    right: Box::new(Expression::Integer(2)),
                },
                Expression::Binary {
                    left: Box::new(Expression::Identifier("y".to_string())),
                    operator: BinaryOperator::Add,
                    right: Box::new(Expression::Integer(3)),
                },
            ],
        };

        match expr {
            Expression::Call {
                function,
                arguments,
            } => {
                assert_eq!(function, "add");
                assert_eq!(arguments.len(), 2);
            }
            _ => panic!("Expected Call expression"),
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_parser_placeholder() {
        let lexer = Lexer::new("placeholder input".to_string());
        let mut parser = Parser::new(lexer);
        assert_eq!(parser.parse(), AstNode::Program);
    }
}
