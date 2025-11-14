//! Lexical analyzer for Forth source code

use crate::ast::{SourceLocation, Token};
use crate::error::{ForthError, Result};

/// Lexer state
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation {
            line: self.line,
            column: self.column,
        }
    }

    /// Peek at the next character without consuming it
    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    /// Consume and return the next character
    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.position += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    /// Skip whitespace
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a line comment (starting with \)
    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.advance() {
            if ch == '\n' {
                break;
            }
        }
    }

    /// Parse a parenthesized comment or stack effect
    fn parse_paren_comment(&mut self) -> Result<Token> {
        // Peek ahead to check if this is a stack effect before consuming anything
        let saved_position = self.position;
        let saved_line = self.line;
        let saved_column = self.column;

        self.advance(); // consume '('
        let mut content = String::new();
        let mut depth = 1;

        while depth > 0 {
            match self.peek() {
                Some('(') => {
                    content.push('(');
                    self.advance();
                    depth += 1;
                }
                Some(')') => {
                    depth -= 1;
                    if depth > 0 {
                        content.push(')');
                        self.advance();
                    } else {
                        // Don't consume the closing paren yet
                        break;
                    }
                }
                Some(ch) => {
                    content.push(ch);
                    self.advance();
                }
                None => {
                    return Err(ForthError::LexError {
                        position: self.position,
                        message: "Unclosed parenthesized comment".to_string(),
                    });
                }
            }
        }

        // Check if this is a stack effect (contains --)
        if content.contains("--") {
            // Reset position to just after the '('
            // The parser will re-parse the content as tokens
            self.position = saved_position;
            self.line = saved_line;
            self.column = saved_column;
            self.advance(); // consume '(' again
            Ok(Token::LeftParen)
        } else {
            // It's a regular comment, consume the closing paren and skip it
            self.advance(); // consume ')'
            // Skip the comment entirely
            self.next_token()
        }
    }

    /// Parse a string literal
    fn parse_string(&mut self) -> Result<Token> {
        self.advance(); // consume opening quote
        let mut value = String::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    return Ok(Token::String(value));
                }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        Some('n') => value.push('\n'),
                        Some('t') => value.push('\t'),
                        Some('r') => value.push('\r'),
                        Some('\\') => value.push('\\'),
                        Some('"') => value.push('"'),
                        Some(ch) => value.push(ch),
                        None => {
                            return Err(ForthError::LexError {
                                position: self.position,
                                message: "Unexpected end of string".to_string(),
                            })
                        }
                    }
                }
                Some(ch) => {
                    value.push(ch);
                    self.advance();
                }
                None => {
                    return Err(ForthError::LexError {
                        position: self.position,
                        message: "Unterminated string literal".to_string(),
                    })
                }
            }
        }
    }

    /// Parse a number (integer or float)
    fn parse_number(&mut self, first_char: char) -> Result<Token> {
        let mut num_str = String::new();
        num_str.push(first_char);

        let mut is_float = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else if ch == 'e' || ch == 'E' {
                is_float = true;
                num_str.push(ch);
                self.advance();
                if let Some('+' | '-') = self.peek() {
                    num_str.push(self.advance().unwrap());
                }
            } else {
                break;
            }
        }

        if is_float {
            num_str.parse::<f64>()
                .map(Token::Float)
                .map_err(|_| ForthError::LexError {
                    position: self.position,
                    message: format!("Invalid float literal: {}", num_str),
                })
        } else {
            num_str.parse::<i64>()
                .map(Token::Integer)
                .map_err(|_| ForthError::LexError {
                    position: self.position,
                    message: format!("Invalid integer literal: {}", num_str),
                })
        }
    }

    /// Parse a word/identifier
    fn parse_word(&mut self, first_char: char) -> Token {
        let mut word = String::new();
        word.push(first_char);

        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            word.push(ch);
            self.advance();
        }

        // Check for keywords (case-insensitive in ANS Forth)
        match word.to_uppercase().as_str() {
            "IF" => Token::If,
            "THEN" => Token::Then,
            "ELSE" => Token::Else,
            "DO" => Token::Do,
            "LOOP" => Token::Loop,
            "+LOOP" => Token::PlusLoop,
            "BEGIN" => Token::Begin,
            "UNTIL" => Token::Until,
            "WHILE" => Token::While,
            "REPEAT" => Token::Repeat,
            "VARIABLE" => Token::Variable,
            "CONSTANT" => Token::Constant,
            "IMMEDIATE" => Token::Immediate,
            _ => Token::Word(word),
        }
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        match self.peek() {
            None => Ok(Token::Eof),
            Some(':') => {
                self.advance();
                Ok(Token::Colon)
            }
            Some(';') => {
                self.advance();
                Ok(Token::Semicolon)
            }
            Some('(') => self.parse_paren_comment(),
            Some(')') => {
                self.advance();
                Ok(Token::RightParen)
            }
            Some('"') => self.parse_string(),
            Some('\\') => {
                self.skip_line_comment();
                self.next_token()
            }
            Some('-') => {
                self.advance();
                if self.peek() == Some('-') {
                    self.advance();
                    Ok(Token::StackEffectSep)
                } else if let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        self.parse_number('-')
                    } else {
                        Ok(Token::Word("-".to_string()))
                    }
                } else {
                    Ok(Token::Word("-".to_string()))
                }
            }
            Some(ch) if ch.is_ascii_digit() => {
                // Peek ahead to see if this is a number or a word starting with a digit
                // (like 2dup, 2swap, etc.)
                let saved_pos = self.position;
                self.advance();

                // Check if the next character would make this a word
                let is_word = if let Some(next_ch) = self.peek() {
                    !next_ch.is_ascii_digit() && !next_ch.is_whitespace() && next_ch != '(' && next_ch != ')' && next_ch != '.'
                } else {
                    false
                };

                if is_word {
                    // Reset and parse as word
                    self.position = saved_pos;
                    self.advance();
                    Ok(self.parse_word(ch))
                } else {
                    // Parse as number
                    self.parse_number(ch)
                }
            }
            Some(ch) => {
                self.advance();
                Ok(self.parse_word(ch))
            }
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let mut lexer = Lexer::new(": double 2 * ;");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Colon);
        assert_eq!(tokens[1], Token::Word("double".to_string()));
        assert_eq!(tokens[2], Token::Integer(2));
        assert_eq!(tokens[3], Token::Word("*".to_string()));
        assert_eq!(tokens[4], Token::Semicolon);
    }

    #[test]
    fn test_tokenize_with_comment() {
        let mut lexer = Lexer::new(": test ( n -- n*2 ) 2 * ;");
        let tokens = lexer.tokenize().unwrap();
        // Comment should be skipped or marked
        assert!(tokens.iter().any(|t| matches!(t, Token::Colon)));
    }

    #[test]
    fn test_tokenize_control_structures() {
        let mut lexer = Lexer::new("IF 1 ELSE 0 THEN");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::If);
        assert_eq!(tokens[1], Token::Integer(1));
        assert_eq!(tokens[2], Token::Else);
        assert_eq!(tokens[3], Token::Integer(0));
        assert_eq!(tokens[4], Token::Then);
    }

    #[test]
    fn test_tokenize_float() {
        let mut lexer = Lexer::new("3.14159 1.0e-10");
        let tokens = lexer.tokenize().unwrap();
        match tokens[0] {
            Token::Float(f) => assert!((f - 3.14159).abs() < 0.0001),
            _ => panic!("Expected float token"),
        }
    }
}
