//! Recursive descent arithmetic expression parser with variable resolution.

use crate::core::variables::{get_variable, has_variable};

// Internal parser state tracking the input string and current character position.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    // Creates a new expression parser for the given input string.
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    // Peeks at the next character without advancing the position.
    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    // Advances the position past any leading whitespace characters.
    fn skip_whitespace(&mut self) {
        if let Some(offset) = self.input[self.pos..].find(|c: char| !c.is_whitespace()) {
            self.pos += offset;
        } else {
            self.pos = self.input.len();
        }
    }

    // Parses a parenthesized expression: ( <expression> ).
    fn parse_parenthesized(&mut self) -> Result<f32, String> {
        // Skip opening '('
        self.pos += 1;

        let value = self.parse_expression()?;

        self.skip_whitespace();

        if self.input.as_bytes().get(self.pos) != Some(&b')') {
            return Err("expected ')'".to_string());
        }

        // Skip closing ')'
        self.pos += 1;

        Ok(value)
    }

    // Parses a variable identifier following '$' and returns its stored value.
    fn parse_variable(&mut self) -> Result<f32, String> {
        // Skip '$' prefix
        self.pos += 1;

        let start = self.pos;

        // Variable must start with a letter or underscore.
        let first_char = self.peek_char();
        match first_char {
            Some(c) if c.is_alphabetic() || c == '_' => {
                self.pos += c.len_utf8();
            }
            _ => return Err("expected variable name after '$'".to_string()),
        }

        // Remaining characters can be alphanumeric or underscores.
        while let Some(c) = self.peek_char() {
            if c.is_alphabetic() || c.is_numeric() || c == '_' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }

        let name = &self.input[start..self.pos];

        if !has_variable(name) {
            return Err(format!("undefined variable: ${}", name));
        }

        Ok(get_variable(name))
    }

    // Parses a numeric integer or decimal literal into an f32.
    fn parse_number(&mut self) -> Result<f32, String> {
        let start = self.pos;
        let bytes = self.input.as_bytes();

        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            if b.is_ascii_digit() || b == b'.' {
                self.pos += 1;
            } else {
                break;
            }
        }

        let number_str = &self.input[start..self.pos];

        number_str
            .parse::<f32>()
            .map_err(|_| format!("invalid number: {}", number_str))
    }

    // Parses a primary expression (number literal, variable reference, or parenthesized sub-expression).
    fn parse_primary(&mut self) -> Result<f32, String> {
        self.skip_whitespace();

        if self.pos >= self.input.len() {
            return Err("unexpected end of expression".to_string());
        }

        let c = self.peek_char().unwrap();

        match c {
            '(' => self.parse_parenthesized(),
            '$' => self.parse_variable(),
            c if c.is_numeric() || c == '.' => self.parse_number(),
            _ => Err(format!("unexpected character: {}", c)),
        }
    }

    // Parses a factor, handling unary '+' and '-' signs before primary expressions.
    fn parse_factor(&mut self) -> Result<f32, String> {
        self.skip_whitespace();

        match self.input.as_bytes().get(self.pos) {
            Some(&b'-') => {
                self.pos += 1;
                let value = self.parse_factor()?;
                Ok(-value)
            }
            Some(&b'+') => {
                self.pos += 1;
                self.parse_factor()
            }
            _ => self.parse_primary(),
        }
    }

    // Parses terms with multiplication and division operators (*, /).
    fn parse_term(&mut self) -> Result<f32, String> {
        let mut value = self.parse_factor()?;

        loop {
            self.skip_whitespace();

            let Some(&op) = self.input.as_bytes().get(self.pos) else {
                break;
            };

            if op != b'*' && op != b'/' {
                break;
            }

            self.pos += 1;

            let rhs = self.parse_factor()?;

            if op == b'*' {
                value *= rhs;
            } else if rhs == 0.0 {
                return Err("division by zero".to_string());
            } else {
                value /= rhs;
            }
        }

        Ok(value)
    }

    // Parses expressions with addition and subtraction operators (+, -).
    fn parse_expression(&mut self) -> Result<f32, String> {
        let mut value = self.parse_term()?;

        loop {
            self.skip_whitespace();

            let Some(&op) = self.input.as_bytes().get(self.pos) else {
                break;
            };

            if op != b'+' && op != b'-' {
                break;
            }

            self.pos += 1;

            let rhs = self.parse_term()?;

            if op == b'+' {
                value += rhs;
            } else {
                value -= rhs;
            }
        }

        Ok(value)
    }
}

/// Evaluates an arithmetic expression string and returns the computed result.
///
/// Supports numbers, variables (`$name`), parentheses, unary plus/minus,
/// and basic operators (`+`, `-`, `*`, `/`).
pub fn evaluate_expression(expression: &str) -> Result<f32, String> {
    let mut parser = Parser::new(expression);

    let result = parser.parse_expression()?;

    parser.skip_whitespace();

    // Ensure all characters were consumed.
    if parser.pos != parser.input.len() {
        let c = parser.peek_char().unwrap();
        return Err(format!("unexpected character: {}", c));
    }

    Ok(result)
}
