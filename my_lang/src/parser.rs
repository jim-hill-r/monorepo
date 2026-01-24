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

use crate::lexer::{Lexer, Token};

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
    /// Function body (list of statements)
    ///
    /// Note: The parser must validate that no Return statements appear in input functions,
    /// as the language semantics prohibit return statements in input declarations.
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

/// Parser errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Unexpected token encountered
    UnexpectedToken {
        /// Expected token description
        expected: String,
        /// Actual token found
        found: Token,
    },
    /// Unexpected end of input
    UnexpectedEof,
}

/// Parser result type
pub type ParseResult<T> = Result<T, ParseError>;

/// Parser structure for building AST from tokens
///
/// Uses recursive descent parsing to build an Abstract Syntax Tree
/// from the token stream produced by the lexer.
pub struct Parser {
    /// Lexer for tokenizing input
    lexer: Lexer,
    /// Current token being examined
    current_token: Token,
    /// Next token (for lookahead)
    peek_token: Token,
}

impl Parser {
    /// Create a new parser with the given lexer
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Self {
            lexer,
            current_token,
            peek_token,
        }
    }

    /// Advance to the next token
    fn advance(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    /// Check if current token matches the expected token type
    fn expect_token(&mut self, expected: Token) -> ParseResult<()> {
        if self.current_token == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: self.current_token.clone(),
            })
        }
    }

    /// Parse the input and produce a Program AST
    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut declarations = Vec::new();

        // Parse declarations until we reach EOF
        while self.current_token != Token::Eof {
            let decl = self.parse_declaration()?;
            declarations.push(decl);
        }

        Ok(Program { declarations })
    }

    /// Parse a top-level declaration (function, input, or output)
    fn parse_declaration(&mut self) -> ParseResult<Declaration> {
        match &self.current_token {
            Token::Function => {
                self.advance();
                let func = self.parse_function_decl()?;
                Ok(Declaration::Function(func))
            }
            Token::Input => {
                self.advance();
                let input = self.parse_input_decl()?;
                Ok(Declaration::Input(input))
            }
            Token::Output => {
                self.advance();
                let output = self.parse_output_decl()?;
                Ok(Declaration::Output(output))
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "function, input, or output".to_string(),
                found: self.current_token.clone(),
            }),
        }
    }

    /// Parse a function declaration
    fn parse_function_decl(&mut self) -> ParseResult<FunctionDecl> {
        // Parse function name
        let name = match &self.current_token {
            Token::Identifier(name) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: self.current_token.clone(),
                });
            }
        };

        // Parse parameters
        let parameters = self.parse_parameters()?;

        // Parse body
        let body = self.parse_block()?;

        Ok(FunctionDecl {
            name,
            parameters,
            body,
        })
    }

    /// Parse an input declaration
    fn parse_input_decl(&mut self) -> ParseResult<InputDecl> {
        // Parse input function name
        let name = match &self.current_token {
            Token::Identifier(name) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: self.current_token.clone(),
                });
            }
        };

        // Parse parameters
        let parameters = self.parse_parameters()?;

        // Parse body
        let body = self.parse_block()?;

        Ok(InputDecl {
            name,
            parameters,
            body,
        })
    }

    /// Parse an output declaration
    fn parse_output_decl(&mut self) -> ParseResult<OutputDecl> {
        // Parse output function name
        let name = match &self.current_token {
            Token::Identifier(name) => {
                let n = name.clone();
                self.advance();
                n
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: self.current_token.clone(),
                });
            }
        };

        // Parse parameters
        let parameters = self.parse_parameters()?;

        // Parse body
        let body = self.parse_block()?;

        Ok(OutputDecl {
            name,
            parameters,
            body,
        })
    }

    /// Parse function parameters: (param1, param2, ...)
    fn parse_parameters(&mut self) -> ParseResult<Vec<String>> {
        self.expect_token(Token::LeftParen)?;

        let mut parameters = Vec::new();

        // Check for empty parameter list
        if self.current_token == Token::RightParen {
            self.advance();
            return Ok(parameters);
        }

        // Parse first parameter
        match &self.current_token {
            Token::Identifier(name) => {
                parameters.push(name.clone());
                self.advance();
            }
            _ => {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier".to_string(),
                    found: self.current_token.clone(),
                });
            }
        }

        // Parse remaining parameters
        while self.current_token == Token::Comma {
            self.advance(); // consume comma

            match &self.current_token {
                Token::Identifier(name) => {
                    parameters.push(name.clone());
                    self.advance();
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "identifier".to_string(),
                        found: self.current_token.clone(),
                    });
                }
            }
        }

        self.expect_token(Token::RightParen)?;
        Ok(parameters)
    }

    /// Parse a block of statements: { stmt1; stmt2; ... }
    fn parse_block(&mut self) -> ParseResult<Vec<Statement>> {
        self.expect_token(Token::LeftBrace)?;

        let mut statements = Vec::new();

        while self.current_token != Token::RightBrace && self.current_token != Token::Eof {
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            // Semicolons are optional after statements
            if self.current_token == Token::Semicolon {
                self.advance();
            }
        }

        self.expect_token(Token::RightBrace)?;
        Ok(statements)
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> ParseResult<Statement> {
        // Check for assignment: identifier = expression
        if let Token::Identifier(name) = &self.current_token
            && self.peek_token == Token::Assign
        {
            let name = name.clone();
            self.advance(); // consume identifier
            self.advance(); // consume '='

            let value = self.parse_expression()?;
            return Ok(Statement::Assignment { name, value });
        }

        // Try to parse expression
        let expr = self.parse_expression()?;

        Ok(Statement::Expression(expr))
    }

    /// Parse an expression
    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_comparison()
    }

    /// Parse comparison operators (==, !=, <, >, <=, >=)
    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_additive()?;

        while let Some(op) = match &self.current_token {
            Token::Equal => Some(BinaryOperator::Equal),
            Token::NotEqual => Some(BinaryOperator::NotEqual),
            Token::LessThan => Some(BinaryOperator::LessThan),
            Token::GreaterThan => Some(BinaryOperator::GreaterThan),
            Token::LessThanEqual => Some(BinaryOperator::LessThanEqual),
            Token::GreaterThanEqual => Some(BinaryOperator::GreaterThanEqual),
            _ => None,
        } {
            self.advance();
            let right = self.parse_additive()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse additive operators (+, -)
    fn parse_additive(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_multiplicative()?;

        while let Some(op) = match &self.current_token {
            Token::Plus => Some(BinaryOperator::Add),
            Token::Minus => Some(BinaryOperator::Subtract),
            _ => None,
        } {
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse multiplicative operators (*, /)
    fn parse_multiplicative(&mut self) -> ParseResult<Expression> {
        let mut left = self.parse_primary()?;

        while let Some(op) = match &self.current_token {
            Token::Asterisk => Some(BinaryOperator::Multiply),
            Token::Slash => Some(BinaryOperator::Divide),
            _ => None,
        } {
            self.advance();
            let right = self.parse_primary()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse primary expressions (literals, identifiers, function calls, parenthesized expressions)
    fn parse_primary(&mut self) -> ParseResult<Expression> {
        match &self.current_token {
            Token::Integer(value) => {
                let val = *value;
                self.advance();
                Ok(Expression::Integer(val))
            }
            Token::StringLiteral(s) => {
                let string = s.clone();
                self.advance();
                Ok(Expression::StringLiteral(string))
            }
            Token::Identifier(name) => {
                let n = name.clone();
                self.advance();

                // Check for function call
                if self.current_token == Token::LeftParen {
                    self.advance();
                    let arguments = self.parse_arguments()?;
                    self.expect_token(Token::RightParen)?;
                    Ok(Expression::Call {
                        function: n,
                        arguments,
                    })
                } else {
                    Ok(Expression::Identifier(n))
                }
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.current_token.clone(),
            }),
        }
    }

    /// Parse function call arguments
    fn parse_arguments(&mut self) -> ParseResult<Vec<Expression>> {
        let mut arguments = Vec::new();

        // Check for empty argument list
        if self.current_token == Token::RightParen {
            return Ok(arguments);
        }

        // Parse first argument
        arguments.push(self.parse_expression()?);

        // Parse remaining arguments
        while self.current_token == Token::Comma {
            self.advance();
            arguments.push(self.parse_expression()?);
        }

        Ok(arguments)
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
    fn test_parser_simple_function() {
        // Test: function add(a, b) { a + b }
        let input = "function add(a, b) { a + b }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(func) => {
                assert_eq!(func.name, "add");
                assert_eq!(func.parameters, vec!["a", "b"]);
                assert_eq!(func.body.len(), 1);

                match &func.body[0] {
                    Statement::Expression(Expression::Binary {
                        left,
                        operator,
                        right,
                    }) => {
                        assert_eq!(**left, Expression::Identifier("a".to_string()));
                        assert_eq!(*operator, BinaryOperator::Add);
                        assert_eq!(**right, Expression::Identifier("b".to_string()));
                    }
                    _ => panic!("Expected binary expression statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_function_with_literals() {
        // Test: function get_answer() { 42 }
        let input = "function get_answer() { 42 }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");
        assert_eq!(program.declarations.len(), 1);

        match &program.declarations[0] {
            Declaration::Function(func) => {
                assert_eq!(func.name, "get_answer");
                assert_eq!(func.parameters.len(), 0);
                assert_eq!(func.body.len(), 1);

                match &func.body[0] {
                    Statement::Expression(Expression::Integer(42)) => {}
                    _ => panic!("Expected integer expression"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_assignment() {
        // Test: function test() { x = 10 }
        let input = "function test() { x = 10 }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Function(func) => {
                assert_eq!(func.body.len(), 1);
                match &func.body[0] {
                    Statement::Assignment { name, value } => {
                        assert_eq!(name, "x");
                        assert_eq!(*value, Expression::Integer(10));
                    }
                    _ => panic!("Expected assignment statement"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_function_call() {
        // Test: function main() { print(42) }
        let input = "function main() { print(42) }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Function(func) => match &func.body[0] {
                Statement::Expression(Expression::Call {
                    function,
                    arguments,
                }) => {
                    assert_eq!(function, "print");
                    assert_eq!(arguments.len(), 1);
                    assert_eq!(arguments[0], Expression::Integer(42));
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_complex_expression() {
        // Test: function calc() { 2 + 3 * 4 }
        let input = "function calc() { 2 + 3 * 4 }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Function(func) => {
                // Should parse as: 2 + (3 * 4) due to operator precedence
                match &func.body[0] {
                    Statement::Expression(Expression::Binary {
                        left,
                        operator,
                        right,
                    }) => {
                        assert_eq!(**left, Expression::Integer(2));
                        assert_eq!(*operator, BinaryOperator::Add);

                        // Right side should be (3 * 4)
                        match &**right {
                            Expression::Binary {
                                left: l2,
                                operator: op2,
                                right: r2,
                            } => {
                                assert_eq!(**l2, Expression::Integer(3));
                                assert_eq!(*op2, BinaryOperator::Multiply);
                                assert_eq!(**r2, Expression::Integer(4));
                            }
                            _ => panic!("Expected nested binary expression"),
                        }
                    }
                    _ => panic!("Expected binary expression"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_comparison_operators() {
        // Test: function compare(a, b) { a < b }
        let input = "function compare(a, b) { a < b }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Function(func) => match &func.body[0] {
                Statement::Expression(Expression::Binary {
                    left,
                    operator,
                    right,
                }) => {
                    assert_eq!(**left, Expression::Identifier("a".to_string()));
                    assert_eq!(*operator, BinaryOperator::LessThan);
                    assert_eq!(**right, Expression::Identifier("b".to_string()));
                }
                _ => panic!("Expected comparison expression"),
            },
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_input_declaration() {
        // Test: input read_number() { x = 5 }
        let input = "input read_number() { x = 5 }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Input(input_decl) => {
                assert_eq!(input_decl.name, "read_number");
                assert_eq!(input_decl.parameters.len(), 0);
                assert_eq!(input_decl.body.len(), 1);
            }
            _ => panic!("Expected input declaration"),
        }
    }

    #[test]
    fn test_parser_output_declaration() {
        // Test: output write_value(x) { print(x) }
        let input = "output write_value(x) { print(x) }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Output(output_decl) => {
                assert_eq!(output_decl.name, "write_value");
                assert_eq!(output_decl.parameters, vec!["x"]);
                assert_eq!(output_decl.body.len(), 1);
            }
            _ => panic!("Expected output declaration"),
        }
    }

    #[test]
    fn test_parser_multiple_declarations() {
        // Test multiple declarations in one program
        let input = "function add(a, b) { a + b } function multiply(x, y) { x * y }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");
        assert_eq!(program.declarations.len(), 2);

        match &program.declarations[0] {
            Declaration::Function(func) => assert_eq!(func.name, "add"),
            _ => panic!("Expected function declaration"),
        }

        match &program.declarations[1] {
            Declaration::Function(func) => assert_eq!(func.name, "multiply"),
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_parenthesized_expression() {
        // Test: function calc() { (2 + 3) * 4 }
        let input = "function calc() { (2 + 3) * 4 }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Function(func) => {
                // Should parse as: (2 + 3) * 4
                match &func.body[0] {
                    Statement::Expression(Expression::Binary {
                        left,
                        operator,
                        right,
                    }) => {
                        assert_eq!(*operator, BinaryOperator::Multiply);
                        assert_eq!(**right, Expression::Integer(4));

                        // Left side should be (2 + 3)
                        match &**left {
                            Expression::Binary {
                                left: l2,
                                operator: op2,
                                right: r2,
                            } => {
                                assert_eq!(**l2, Expression::Integer(2));
                                assert_eq!(*op2, BinaryOperator::Add);
                                assert_eq!(**r2, Expression::Integer(3));
                            }
                            _ => panic!("Expected nested binary expression"),
                        }
                    }
                    _ => panic!("Expected binary expression"),
                }
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_string_literals() {
        // Test: function greet() { "hello" }
        let input = r#"function greet() { "hello" }"#.to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Function(func) => match &func.body[0] {
                Statement::Expression(Expression::StringLiteral(s)) => {
                    assert_eq!(s, "hello");
                }
                _ => panic!("Expected string literal expression"),
            },
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_semicolons() {
        // Test: function test() { x = 1; y = 2; }
        let input = "function test() { x = 1; y = 2; }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse().expect("Failed to parse");

        match &program.declarations[0] {
            Declaration::Function(func) => {
                assert_eq!(func.body.len(), 2);
            }
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parser_error_missing_paren() {
        // Test: function add(a, b { a + b }  -- missing closing paren
        let input = "function add(a, b { a + b }".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let result = parser.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_error_unexpected_token() {
        // Test: 123 -- starts with number, not a declaration
        let input = "123".to_string();
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let result = parser.parse();
        assert!(result.is_err());
    }
}
