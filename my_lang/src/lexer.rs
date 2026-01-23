//! Lexer module for my_lang
//!
//! The lexer is responsible for tokenizing source code into a stream of tokens.
//! This will be the first phase of compilation.
//!
//! ## Token Types
//!
//! The lexer supports the following token types:
//! - **Keywords**: `let`, `fn`, `if`, `else`, `return`, `while`, `for`, `true`, `false`
//! - **Identifiers**: Variable and function names (starting with letter or underscore)
//! - **Literals**: Integer and string literals
//! - **Operators**: `+`, `-`, `*`, `/`, `=`, `==`, `!=`, `<`, `>`, `<=`, `>=`
//! - **Delimiters**: `(`, `)`, `{`, `}`, `,`, `;`

/// Token type representing all possible tokens in my_lang
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Keywords
    Let,
    Fn,
    If,
    Else,
    Return,
    While,
    For,
    True,
    False,

    // Identifiers and literals
    Identifier(String),
    Integer(i64),
    StringLiteral(String),

    // Operators
    Plus,
    Minus,
    Asterisk,
    Slash,
    Assign,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,

    // Special
    Eof,
    Illegal(char),
}

/// Lexer structure that tokenizes source code
pub struct Lexer {
    input: String,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    /// Create a new lexer for the given input
    pub fn new(input: String) -> Self {
        let current_char = input.chars().next();
        Self {
            input,
            position: 0,
            current_char,
        }
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char {
            None => Token::Eof,
            Some(ch) => match ch {
                '+' => {
                    self.advance();
                    Token::Plus
                }
                '-' => {
                    self.advance();
                    Token::Minus
                }
                '*' => {
                    self.advance();
                    Token::Asterisk
                }
                '/' => {
                    self.advance();
                    Token::Slash
                }
                '=' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::Equal
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::NotEqual
                    } else {
                        Token::Illegal('!')
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::LessThanEqual
                    } else {
                        Token::LessThan
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::GreaterThanEqual
                    } else {
                        Token::GreaterThan
                    }
                }
                '(' => {
                    self.advance();
                    Token::LeftParen
                }
                ')' => {
                    self.advance();
                    Token::RightParen
                }
                '{' => {
                    self.advance();
                    Token::LeftBrace
                }
                '}' => {
                    self.advance();
                    Token::RightBrace
                }
                ',' => {
                    self.advance();
                    Token::Comma
                }
                ';' => {
                    self.advance();
                    Token::Semicolon
                }
                '"' => self.read_string(),
                _ if ch.is_ascii_digit() => self.read_integer(),
                _ if ch.is_ascii_alphabetic() || ch == '_' => self.read_identifier(),
                _ => {
                    let illegal = ch;
                    self.advance();
                    Token::Illegal(illegal)
                }
            },
        }
    }

    /// Advance to the next character
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.chars().nth(self.position);
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> Token {
        let start = self.position;
        while let Some(ch) = self.current_char {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        let identifier = self.input[start..self.position].to_string();

        // Check if it's a keyword
        match identifier.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "while" => Token::While,
            "for" => Token::For,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(identifier),
        }
    }

    /// Read an integer literal
    fn read_integer(&mut self) -> Token {
        let start = self.position;
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        let number_str = &self.input[start..self.position];
        let number = number_str.parse::<i64>().unwrap_or(0);
        Token::Integer(number)
    }

    /// Read a string literal
    fn read_string(&mut self) -> Token {
        self.advance(); // Skip opening quote
        let start = self.position;
        while let Some(ch) = self.current_char {
            if ch == '"' {
                break;
            }
            self.advance();
        }
        let string = self.input[start..self.position].to_string();
        self.advance(); // Skip closing quote
        Token::StringLiteral(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_character_tokens() {
        let input = "+ - * / = ( ) { } , ;".to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::Plus,
            Token::Minus,
            Token::Asterisk,
            Token::Slash,
            Token::Assign,
            Token::LeftParen,
            Token::RightParen,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Comma,
            Token::Semicolon,
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_comparison_operators() {
        let input = "== != < > <= >=".to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::Equal,
            Token::NotEqual,
            Token::LessThan,
            Token::GreaterThan,
            Token::LessThanEqual,
            Token::GreaterThanEqual,
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_keywords() {
        let input = "let fn if else return while for true false".to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::Let,
            Token::Fn,
            Token::If,
            Token::Else,
            Token::Return,
            Token::While,
            Token::For,
            Token::True,
            Token::False,
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_identifiers() {
        let input = "x y foo bar _private my_var CamelCase".to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::Identifier("x".to_string()),
            Token::Identifier("y".to_string()),
            Token::Identifier("foo".to_string()),
            Token::Identifier("bar".to_string()),
            Token::Identifier("_private".to_string()),
            Token::Identifier("my_var".to_string()),
            Token::Identifier("CamelCase".to_string()),
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_integer_literals() {
        let input = "0 123 456789".to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::Integer(0),
            Token::Integer(123),
            Token::Integer(456789),
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_string_literals() {
        let input = r#""hello" "world" "with spaces""#.to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::StringLiteral("hello".to_string()),
            Token::StringLiteral("world".to_string()),
            Token::StringLiteral("with spaces".to_string()),
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_simple_program() {
        let input = r#"
            let x = 42;
            fn add(a, b) {
                return a + b;
            }
        "#
        .to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Integer(42),
            Token::Semicolon,
            Token::Fn,
            Token::Identifier("add".to_string()),
            Token::LeftParen,
            Token::Identifier("a".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_if_statement() {
        let input = "if x < 10 { return true; } else { return false; }".to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::If,
            Token::Identifier("x".to_string()),
            Token::LessThan,
            Token::Integer(10),
            Token::LeftBrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::RightBrace,
            Token::Else,
            Token::LeftBrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }

    #[test]
    fn test_illegal_character() {
        let input = "@".to_string();
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Illegal('@'));
    }

    #[test]
    fn test_empty_string() {
        let input = "".to_string();
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_whitespace_handling() {
        let input = "  let   x  =  42  ;  ".to_string();
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Assign,
            Token::Integer(42),
            Token::Semicolon,
            Token::Eof,
        ];

        for expected_token in expected {
            assert_eq!(lexer.next_token(), expected_token);
        }
    }
}
