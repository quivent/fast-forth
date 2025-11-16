//! Parser for Forth source code

use crate::ast::*;
use crate::error::{ForthError, Result};
use crate::lexer::Lexer;

/// Parser state
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    /// Peek at current token
    fn peek(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    /// Consume and return current token
    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        self.position += 1;
        token
    }

    /// Check if current token matches expected
    fn expect(&mut self, expected: Token) -> Result<()> {
        let token = self.advance();
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            Ok(())
        } else {
            Err(ForthError::ParseError {
                line: 0,
                column: 0,
                message: format!("Expected {:?}, found {:?}", expected, token),
            })
        }
    }

    /// Parse the entire program
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut program = Program::new();
        let mut pending_value: Option<i64> = None;

        while !matches!(self.peek(), Token::Eof) {
            match self.peek() {
                Token::Colon => {
                    // If we have a pending value, push it first
                    if let Some(value) = pending_value.take() {
                        program.top_level_code.push(Word::IntLiteral(value));
                    }
                    let def = self.parse_definition()?;
                    program.definitions.push(def);
                }
                Token::Variable => {
                    // If we have a pending value, push it first
                    if let Some(value) = pending_value.take() {
                        program.top_level_code.push(Word::IntLiteral(value));
                    }
                    self.advance();
                    if let Token::Word(name) = self.advance() {
                        program.top_level_code.push(Word::Variable { name });
                    } else {
                        return Err(ForthError::ParseError {
                            line: 0,
                            column: 0,
                            message: "Expected variable name".to_string(),
                        });
                    }
                }
                Token::Constant => {
                    self.advance();
                    // The value should have been parsed as the previous token
                    if let Some(value) = pending_value.take() {
                        if let Token::Word(name) = self.advance() {
                            program.top_level_code.push(Word::Constant { name, value });
                        } else {
                            return Err(ForthError::ParseError {
                                line: 0,
                                column: 0,
                                message: "Expected constant name".to_string(),
                            });
                        }
                    } else {
                        return Err(ForthError::ParseError {
                            line: 0,
                            column: 0,
                            message: "Expected constant value before CONSTANT".to_string(),
                        });
                    }
                }
                Token::Integer(value) => {
                    // If we have a pending value, push it first
                    if let Some(prev_value) = pending_value.take() {
                        program.top_level_code.push(Word::IntLiteral(prev_value));
                    }
                    // Save this value in case the next token is CONSTANT
                    pending_value = Some(*value);
                    self.advance();
                }
                _ => {
                    // If we have a pending value, push it first
                    if let Some(value) = pending_value.take() {
                        program.top_level_code.push(Word::IntLiteral(value));
                    }
                    let word = self.parse_word()?;
                    program.top_level_code.push(word);
                }
            }
        }

        // Push any remaining pending value
        if let Some(value) = pending_value {
            program.top_level_code.push(Word::IntLiteral(value));
        }

        Ok(program)
    }

    /// Parse a word definition (: name ... ;)
    fn parse_definition(&mut self) -> Result<Definition> {
        self.expect(Token::Colon)?;

        let name = match self.advance() {
            Token::Word(name) => name,
            token => {
                return Err(ForthError::ParseError {
                    line: 0,
                    column: 0,
                    message: format!("Expected word name, found {:?}", token),
                })
            }
        };

        let location = SourceLocation::default();

        // Parse optional stack effect comment
        let stack_effect = if matches!(self.peek(), Token::LeftParen) {
            self.parse_stack_effect()?
        } else {
            None
        };

        let mut body = Vec::new();
        let mut immediate = false;

        // Parse definition body
        loop {
            match self.peek() {
                Token::Semicolon => {
                    self.advance();
                    break;
                }
                Token::Eof => {
                    return Err(ForthError::ParseError {
                        line: 0,
                        column: 0,
                        message: format!("Unterminated definition: {}", name),
                    })
                }
                _ => {
                    let word = self.parse_word()?;
                    body.push(word);
                }
            }
        }

        // Check for IMMEDIATE after semicolon
        if matches!(self.peek(), Token::Immediate) {
            self.advance();
            immediate = true;
        }

        Ok(Definition {
            name,
            body,
            immediate,
            stack_effect,
            location,
        })
    }

    /// Parse a stack effect comment ( a b -- c )
    fn parse_stack_effect(&mut self) -> Result<Option<StackEffect>> {
        if !matches!(self.peek(), Token::LeftParen) {
            return Ok(None);
        }
        self.advance();

        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut before_separator = true;

        loop {
            match self.peek() {
                Token::RightParen => {
                    self.advance();
                    break;
                }
                Token::StackEffectSep => {
                    self.advance();
                    before_separator = false;
                }
                Token::Word(name) => {
                    let name = name.clone();
                    self.advance();
                    let stack_type = match name.as_str() {
                        "n" | "i" | "int" => StackType::Int,
                        "f" | "float" => StackType::Float,
                        "addr" | "a" => StackType::Addr,
                        "bool" | "flag" => StackType::Bool,
                        "c" | "char" => StackType::Char,
                        "s" | "string" => StackType::String,
                        _ => StackType::Unknown,
                    };

                    if before_separator {
                        inputs.push(stack_type);
                    } else {
                        outputs.push(stack_type);
                    }
                }
                Token::Eof => {
                    return Err(ForthError::ParseError {
                        line: 0,
                        column: 0,
                        message: "Unterminated stack effect".to_string(),
                    })
                }
                _ => {
                    self.advance(); // Skip other tokens in comments
                }
            }
        }

        Ok(Some(StackEffect::new(inputs, outputs)))
    }

    /// Parse a single word
    fn parse_word(&mut self) -> Result<Word> {
        match self.peek().clone() {
            Token::Integer(value) => {
                self.advance();
                Ok(Word::IntLiteral(value))
            }
            Token::Float(value) => {
                self.advance();
                Ok(Word::FloatLiteral(value))
            }
            Token::String(value) => {
                self.advance();
                Ok(Word::StringLiteral(value))
            }
            Token::If => {
                self.advance();
                self.parse_if()
            }
            Token::Begin => {
                self.advance();
                self.parse_begin()
            }
            Token::Do => {
                self.advance();
                self.parse_do_loop()
            }
            Token::Word(name) => {
                self.advance();
                Ok(Word::WordRef {
                    name,
                    location: SourceLocation::default(),
                })
            }
            token => Err(ForthError::ParseError {
                line: 0,
                column: 0,
                message: format!("Unexpected token: {:?}", token),
            }),
        }
    }

    /// Parse IF...THEN or IF...ELSE...THEN
    fn parse_if(&mut self) -> Result<Word> {
        let mut then_branch = Vec::new();
        let mut else_branch = None;

        loop {
            match self.peek() {
                Token::Then => {
                    self.advance();
                    break;
                }
                Token::Else => {
                    self.advance();
                    let mut else_body = Vec::new();
                    loop {
                        match self.peek() {
                            Token::Then => {
                                self.advance();
                                else_branch = Some(else_body);
                                return Ok(Word::If {
                                    then_branch,
                                    else_branch,
                                });
                            }
                            Token::Eof => {
                                return Err(ForthError::ParseError {
                                    line: 0,
                                    column: 0,
                                    message: "Unterminated IF...ELSE".to_string(),
                                })
                            }
                            _ => {
                                let word = self.parse_word()?;
                                else_body.push(word);
                            }
                        }
                    }
                }
                Token::Eof => {
                    return Err(ForthError::ParseError {
                        line: 0,
                        column: 0,
                        message: "Unterminated IF".to_string(),
                    })
                }
                _ => {
                    let word = self.parse_word()?;
                    then_branch.push(word);
                }
            }
        }

        Ok(Word::If {
            then_branch,
            else_branch,
        })
    }

    /// Parse BEGIN...UNTIL or BEGIN...WHILE...REPEAT
    fn parse_begin(&mut self) -> Result<Word> {
        let mut body = Vec::new();

        loop {
            match self.peek() {
                Token::Until => {
                    self.advance();
                    return Ok(Word::BeginUntil { body });
                }
                Token::While => {
                    self.advance();
                    let condition = body;
                    let mut repeat_body = Vec::new();

                    loop {
                        match self.peek() {
                            Token::Repeat => {
                                self.advance();
                                return Ok(Word::BeginWhileRepeat {
                                    condition,
                                    body: repeat_body,
                                });
                            }
                            Token::Eof => {
                                return Err(ForthError::ParseError {
                                    line: 0,
                                    column: 0,
                                    message: "Unterminated BEGIN...WHILE".to_string(),
                                })
                            }
                            _ => {
                                let word = self.parse_word()?;
                                repeat_body.push(word);
                            }
                        }
                    }
                }
                Token::Eof => {
                    return Err(ForthError::ParseError {
                        line: 0,
                        column: 0,
                        message: "Unterminated BEGIN".to_string(),
                    })
                }
                _ => {
                    let word = self.parse_word()?;
                    body.push(word);
                }
            }
        }
    }

    /// Parse DO...LOOP or DO...+LOOP
    fn parse_do_loop(&mut self) -> Result<Word> {
        let mut body = Vec::new();

        loop {
            match self.peek() {
                Token::Loop => {
                    self.advance();
                    return Ok(Word::DoLoop { body, increment: 1 });
                }
                Token::PlusLoop => {
                    self.advance();
                    // TODO: Handle variable increment
                    return Ok(Word::DoLoop { body, increment: 1 });
                }
                Token::Eof => {
                    return Err(ForthError::ParseError {
                        line: 0,
                        column: 0,
                        message: "Unterminated DO loop".to_string(),
                    })
                }
                _ => {
                    let word = self.parse_word()?;
                    body.push(word);
                }
            }
        }
    }
}

/// Parse a Forth program from source code
pub fn parse_program(source: &str) -> Result<Program> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_definition() {
        let program = parse_program(": double 2 * ;").unwrap();
        assert_eq!(program.definitions.len(), 1);
        assert_eq!(program.definitions[0].name, "double");
        assert_eq!(program.definitions[0].body.len(), 2);
    }

    #[test]
    fn test_parse_with_stack_effect() {
        let program = parse_program(": square ( n -- n*n ) dup * ;").unwrap();
        assert_eq!(program.definitions.len(), 1);
        let def = &program.definitions[0];
        assert!(def.stack_effect.is_some());
    }

    #[test]
    fn test_parse_if_then() {
        let program = parse_program(": abs ( n -- |n| ) dup 0 < IF negate THEN ;").unwrap();
        assert_eq!(program.definitions.len(), 1);
    }

    #[test]
    fn test_parse_begin_until() {
        let program = parse_program(": countdown BEGIN dup . 1 - dup 0 = UNTIL drop ;").unwrap();
        assert_eq!(program.definitions.len(), 1);
    }

    #[test]
    fn test_deeply_nested_definitions() {
        // Test 15+ levels of nested IF-THEN structures
        let source = ": deep-nest ( n -- result )
            dup 0 > IF
                dup 1 > IF
                    dup 2 > IF
                        dup 3 > IF
                            dup 4 > IF
                                dup 5 > IF
                                    dup 6 > IF
                                        dup 7 > IF
                                            dup 8 > IF
                                                dup 9 > IF
                                                    dup 10 > IF
                                                        dup 11 > IF
                                                            dup 12 > IF
                                                                dup 13 > IF
                                                                    dup 14 > IF
                                                                        100
                                                                    THEN
                                                                THEN
                                                            THEN
                                                        THEN
                                                    THEN
                                                THEN
                                            THEN
                                        THEN
                                    THEN
                                THEN
                            THEN
                        THEN
                    THEN
                THEN
            THEN
        ;";
        let program = parse_program(source).unwrap();
        assert_eq!(program.definitions.len(), 1);
        assert_eq!(program.definitions[0].name, "deep-nest");
    }

    #[test]
    fn test_malformed_if_missing_then() {
        // IF without THEN should produce error
        // The semicolon terminates the definition, so parser sees unterminated IF
        let result = parse_program(": broken-if 5 0 > IF 10");
        assert!(result.is_err());
        // The error will be either "Unterminated IF" or "Unterminated definition"
        if let Err(ForthError::ParseError { message, .. }) = result {
            assert!(
                message.contains("Unterminated") || message.contains("IF"),
                "Expected error about unterminated construct, got: {}",
                message
            );
        } else {
            panic!("Expected ParseError for unterminated IF");
        }
    }

    #[test]
    fn test_unicode_in_strings() {
        // Unicode characters in string literals
        // This Forth implementation uses regular double quotes, not S"
        let program = parse_program(r#": greet " Hello 世界 🌍 " ;"#).unwrap();
        assert_eq!(program.definitions.len(), 1);
        // Verify the string literal contains unicode
        let def = &program.definitions[0];
        assert!(!def.body.is_empty());
        // Check that the string word contains unicode
        if let Some(Word::StringLiteral(s)) = def.body.first() {
            assert!(s.contains("世界"));
            assert!(s.contains("🌍"));
        }
    }

    #[test]
    fn test_very_long_identifier() {
        // Identifier >256 chars - should still parse correctly
        let long_name = "a".repeat(300);
        let source = format!(": {} 42 ;", long_name);
        let program = parse_program(&source).unwrap();
        assert_eq!(program.definitions.len(), 1);
        assert_eq!(program.definitions[0].name.len(), 300);
    }

    #[test]
    fn test_nested_comments() {
        // Nested comments - outer comment should consume inner
        // The lexer supports nested parenthesis comments
        let program = parse_program(": test-comment ( outer ( inner ) outer ) 42 ;").unwrap();
        assert_eq!(program.definitions.len(), 1);
        let def = &program.definitions[0];
        // Since the comment contains nested parens, it's treated as a regular comment,
        // not a stack effect comment (which requires -- separator)
        assert_eq!(def.body.len(), 1); // Just the 42

        // Test with proper stack effect that has nested structure
        let program2 = parse_program(": test-stack ( n ( inner ) -- result ) 42 ;").unwrap();
        assert_eq!(program2.definitions.len(), 1);
    }

    #[test]
    fn test_unbalanced_stack_comments() {
        // Stack comment with mismatched input/output counts
        // Parser should still accept it (semantic validation is separate)
        let program = parse_program(": unbalanced ( a b -- c d e f g ) + ;").unwrap();
        assert_eq!(program.definitions.len(), 1);
        let def = &program.definitions[0];
        assert!(def.stack_effect.is_some());
        if let Some(ref effect) = def.stack_effect {
            assert_eq!(effect.inputs.len(), 2);
            assert_eq!(effect.outputs.len(), 5);
        }
    }

    #[test]
    fn test_immediate_word_compilation() {
        // IMMEDIATE word after semicolon
        let program = parse_program(": test-immediate 42 ; IMMEDIATE").unwrap();
        assert_eq!(program.definitions.len(), 1);
        let def = &program.definitions[0];
        assert!(def.immediate, "Word should be marked as IMMEDIATE");
    }

    #[test]
    fn test_unterminated_definition() {
        // Definition without semicolon should error
        let result = parse_program(": no-semicolon 42");
        assert!(result.is_err());
        if let Err(ForthError::ParseError { message, .. }) = result {
            assert!(message.contains("Unterminated definition"));
        } else {
            panic!("Expected ParseError for unterminated definition");
        }
    }

    #[test]
    fn test_multiple_definitions_with_comments() {
        // Multiple definitions with various comment styles
        let source = r#"
            : first ( n -- n+1 ) 1 + ;
            : second ( n -- n*2 ) 2 * ;
            : third ( a b -- sum ) + ;
        "#;
        let program = parse_program(source).unwrap();
        assert_eq!(program.definitions.len(), 3);
        assert_eq!(program.definitions[0].name, "first");
        assert_eq!(program.definitions[1].name, "second");
        assert_eq!(program.definitions[2].name, "third");
    }
}
