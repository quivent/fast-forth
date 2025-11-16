//! Test utilities for ANS Forth compliance testing
//!
//! Provides a simplified ForthEngine interface for testing standard Forth operations.

/// Simple Forth engine for testing
///
/// Provides a stack-based interface for testing ANS Forth compliance
pub struct ForthEngine {
    stack: Vec<i64>,
    output: String,
}

impl ForthEngine {
    /// Create a new Forth engine
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            output: String::new(),
        }
    }

    /// Evaluate a Forth expression and update the stack
    ///
    /// This is a simplified interpreter for testing purposes
    pub fn eval(&mut self, source: &str) -> Result<(), String> {
        // Simple token-based interpreter for testing
        for token in source.split_whitespace() {
            self.eval_token(token)?;
        }
        Ok(())
    }

    fn eval_token(&mut self, token: &str) -> Result<(), String> {
        match token {
            // Stack manipulation
            "DUP" => {
                let val = self.pop()?;
                self.stack.push(val);
                self.stack.push(val);
            }
            "DROP" => {
                self.pop()?;
            }
            "SWAP" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(b);
                self.stack.push(a);
            }
            "OVER" => {
                // ( a b -- a b a )
                if self.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let val = self.stack[self.stack.len() - 2];
                self.stack.push(val);
            }
            "ROT" => {
                // ( a b c -- b c a )
                let c = self.pop()?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(b);
                self.stack.push(c);
                self.stack.push(a);
            }
            "-ROT" => {
                // ( a b c -- c a b )
                let c = self.pop()?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(c);
                self.stack.push(a);
                self.stack.push(b);
            }
            "2DUP" => {
                // ( a b -- a b a b )
                if self.stack.len() < 2 {
                    return Err("Stack underflow".to_string());
                }
                let b = self.stack[self.stack.len() - 1];
                let a = self.stack[self.stack.len() - 2];
                self.stack.push(a);
                self.stack.push(b);
            }
            "2DROP" => {
                self.pop()?;
                self.pop()?;
            }
            "2SWAP" => {
                // ( a b c d -- c d a b )
                let d = self.pop()?;
                let c = self.pop()?;
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(c);
                self.stack.push(d);
                self.stack.push(a);
                self.stack.push(b);
            }
            "2OVER" => {
                // ( a b c d -- a b c d a b )
                if self.stack.len() < 4 {
                    return Err("Stack underflow".to_string());
                }
                let b = self.stack[self.stack.len() - 4];
                let a = self.stack[self.stack.len() - 3];
                self.stack.push(b);
                self.stack.push(a);
            }
            "?DUP" => {
                let val = self.pop()?;
                self.stack.push(val);
                if val != 0 {
                    self.stack.push(val);
                }
            }
            "NIP" => {
                // ( a b -- b )
                let b = self.pop()?;
                self.pop()?;
                self.stack.push(b);
            }
            "TUCK" => {
                // ( a b -- b a b )
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(b);
            }

            // Arithmetic
            "+" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a + b);
            }
            "-" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a - b);
            }
            "*" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a * b);
            }
            "/" => {
                let b = self.pop()?;
                let a = self.pop()?;
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(a / b);
            }
            "MOD" => {
                let b = self.pop()?;
                let a = self.pop()?;
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(a % b);
            }
            "/MOD" => {
                let b = self.pop()?;
                let a = self.pop()?;
                if b == 0 {
                    return Err("Division by zero".to_string());
                }
                self.stack.push(a % b);  // remainder
                self.stack.push(a / b);  // quotient
            }
            "1+" => {
                let a = self.pop()?;
                self.stack.push(a + 1);
            }
            "1-" => {
                let a = self.pop()?;
                self.stack.push(a - 1);
            }
            "2*" => {
                let a = self.pop()?;
                self.stack.push(a * 2);
            }
            "2/" => {
                let a = self.pop()?;
                self.stack.push(a / 2);
            }
            "NEGATE" => {
                let a = self.pop()?;
                self.stack.push(-a);
            }
            "ABS" => {
                let a = self.pop()?;
                self.stack.push(a.abs());
            }
            "MIN" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.min(b));
            }
            "MAX" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a.max(b));
            }

            // Comparison
            "=" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(if a == b { -1 } else { 0 });
            }
            "<>" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(if a != b { -1 } else { 0 });
            }
            "<" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(if a < b { -1 } else { 0 });
            }
            ">" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(if a > b { -1 } else { 0 });
            }
            "<=" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(if a <= b { -1 } else { 0 });
            }
            ">=" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(if a >= b { -1 } else { 0 });
            }
            "0=" => {
                let a = self.pop()?;
                self.stack.push(if a == 0 { -1 } else { 0 });
            }
            "0<>" => {
                let a = self.pop()?;
                self.stack.push(if a != 0 { -1 } else { 0 });
            }
            "0<" => {
                let a = self.pop()?;
                self.stack.push(if a < 0 { -1 } else { 0 });
            }
            "0>" => {
                let a = self.pop()?;
                self.stack.push(if a > 0 { -1 } else { 0 });
            }

            // Logic
            "AND" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a & b);
            }
            "OR" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a | b);
            }
            "XOR" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a ^ b);
            }
            "INVERT" => {
                let a = self.pop()?;
                self.stack.push(!a);
            }
            "NOT" => {
                let a = self.pop()?;
                self.stack.push(if a == 0 { -1 } else { 0 });
            }
            "LSHIFT" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a << b);
            }
            "RSHIFT" => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.stack.push(a >> b);
            }

            // Stack depth operations
            "DEPTH" => {
                self.stack.push(self.stack.len() as i64);
            }

            // Output operations (simplified)
            "." => {
                let val = self.pop()?;
                self.output.push_str(&format!("{} ", val));
            }
            "EMIT" => {
                let val = self.pop()?;
                if let Some(ch) = char::from_u32(val as u32) {
                    self.output.push(ch);
                }
            }
            "CR" => {
                self.output.push('\n');
            }
            "SPACE" => {
                self.output.push(' ');
            }
            "SPACES" => {
                let n = self.pop()?;
                for _ in 0..n {
                    self.output.push(' ');
                }
            }

            // Try to parse as number
            _ => {
                if let Ok(num) = token.parse::<i64>() {
                    self.stack.push(num);
                } else if token.starts_with('-') && token.len() > 1 {
                    // Handle negative numbers
                    if let Ok(num) = token.parse::<i64>() {
                        self.stack.push(num);
                    } else {
                        return Err(format!("Unknown word: {}", token));
                    }
                } else {
                    return Err(format!("Unknown word: {}", token));
                }
            }
        }
        Ok(())
    }

    fn pop(&mut self) -> Result<i64, String> {
        self.stack.pop().ok_or_else(|| "Stack underflow".to_string())
    }

    /// Get the current stack contents
    pub fn stack(&self) -> &[i64] {
        &self.stack
    }

    /// Get the output generated so far
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Clear the stack
    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }

    /// Clear the output
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Clear both stack and output
    pub fn clear(&mut self) {
        self.clear_stack();
        self.clear_output();
    }
}

impl Default for ForthEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forth_engine_basic() {
        let mut engine = ForthEngine::new();
        engine.eval("5 10 +").unwrap();
        assert_eq!(engine.stack(), &[15]);
    }

    #[test]
    fn test_forth_engine_stack_ops() {
        let mut engine = ForthEngine::new();
        engine.eval("5 DUP").unwrap();
        assert_eq!(engine.stack(), &[5, 5]);
    }

    #[test]
    fn test_forth_engine_underflow() {
        let mut engine = ForthEngine::new();
        let result = engine.eval("DUP");
        assert!(result.is_err());
    }
}
