//! Simple Forth engine for testing and REPL
//!
//! Provides a simple interface for executing Forth code and inspecting the stack

use crate::{Compiler, CompilationMode, OptimizationLevel, Result};
use std::collections::HashMap;
use std::fmt;

/// Simple Forth execution engine for testing
pub struct ForthEngine {
    compiler: Compiler,
    stack: Vec<i64>,
    return_stack: Vec<i64>,
    memory: HashMap<i64, i64>,
    variables: HashMap<String, i64>,
    constants: HashMap<String, i64>,
    values: HashMap<String, i64>,
    next_addr: i64,
    base: i64,
    output: String,
}

impl ForthEngine {
    /// Create a new Forth engine
    pub fn new() -> Self {
        Self {
            compiler: Compiler::new(OptimizationLevel::Standard),
            stack: Vec::new(),
            return_stack: Vec::new(),
            memory: HashMap::new(),
            variables: HashMap::new(),
            constants: HashMap::new(),
            values: HashMap::new(),
            next_addr: 0x1000, // Start memory addresses at 0x1000
            base: 10,
            output: String::new(),
        }
    }

    /// Evaluate Forth code
    pub fn eval(&mut self, code: &str) -> Result<()> {
        // Parse simple stack operations for testing
        // This is a minimal interpreter for differential testing
        let tokens: Vec<&str> = code.split_whitespace().collect();

        for token in tokens {
            match token.to_uppercase().as_str() {
                // Numbers
                s if s.parse::<i64>().is_ok() => {
                    self.stack.push(s.parse().unwrap());
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
                        return Err(crate::error::CompileError::RuntimeError("Division by zero".to_string()));
                    }
                    self.stack.push(a / b);
                }
                "MOD" => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    if b == 0 {
                        return Err(crate::error::CompileError::RuntimeError("Modulo by zero".to_string()));
                    }
                    self.stack.push(a % b);
                }
                "/MOD" => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    if b == 0 {
                        return Err(crate::error::CompileError::RuntimeError("Division by zero".to_string()));
                    }
                    self.stack.push(a % b);  // remainder
                    self.stack.push(a / b);  // quotient
                }
                // Stack manipulation
                "DUP" => {
                    let a = self.peek()?;
                    self.stack.push(a);
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
                        return Err(crate::error::CompileError::RuntimeError("Stack underflow".to_string()));
                    }
                    let a = self.stack[self.stack.len() - 2];
                    self.stack.push(a);
                }
                "ROT" => {
                    // ( a b c -- b c a )
                    if self.stack.len() < 3 {
                        return Err(crate::error::CompileError::RuntimeError("Stack underflow".to_string()));
                    }
                    let c = self.pop()?;
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(b);
                    self.stack.push(c);
                    self.stack.push(a);
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
                "2DUP" => {
                    // ( a b -- a b a b )
                    if self.stack.len() < 2 {
                        return Err(crate::error::CompileError::RuntimeError("Stack underflow".to_string()));
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
                    if self.stack.len() < 4 {
                        return Err(crate::error::CompileError::RuntimeError("Stack underflow".to_string()));
                    }
                    let d = self.pop()?;
                    let c = self.pop()?;
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(c);
                    self.stack.push(d);
                    self.stack.push(a);
                    self.stack.push(b);
                }
                // Comparison
                "=" => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.stack.push(if a == b { -1 } else { 0 });
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
                "0<" => {
                    let a = self.pop()?;
                    self.stack.push(if a < 0 { -1 } else { 0 });
                }
                "0>" => {
                    let a = self.pop()?;
                    self.stack.push(if a > 0 { -1 } else { 0 });
                }
                // Logical
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
                "." => {
                    // Print and drop (for GForth compatibility)
                    let val = self.pop()?;
                    self.output.push_str(&format!("{} ", val));
                }

                // PRIORITY 1: Memory Operations
                "!" => {
                    // ( val addr -- ) Store value at address
                    let addr = self.pop()?;
                    let val = self.pop()?;
                    self.memory.insert(addr, val);
                }
                "@" => {
                    // ( addr -- val ) Fetch value from address
                    let addr = self.pop()?;
                    let val = *self.memory.get(&addr).unwrap_or(&0);
                    self.stack.push(val);
                }
                "+!" => {
                    // ( n addr -- ) Add n to value at addr
                    let addr = self.pop()?;
                    let n = self.pop()?;
                    let current = *self.memory.get(&addr).unwrap_or(&0);
                    self.memory.insert(addr, current + n);
                }

                // PRIORITY 2: Advanced Stack Operations (Return Stack)
                ">R" => {
                    // ( n -- ) Move from data stack to return stack
                    let val = self.pop()?;
                    self.return_stack.push(val);
                }
                "R>" => {
                    // ( -- n ) Move from return stack to data stack
                    let val = self.return_stack.pop().ok_or_else(|| {
                        crate::error::CompileError::RuntimeError("Return stack underflow".to_string())
                    })?;
                    self.stack.push(val);
                }
                "R@" => {
                    // ( -- n ) Copy from return stack to data stack
                    let val = self.return_stack.last().copied().ok_or_else(|| {
                        crate::error::CompileError::RuntimeError("Return stack underflow".to_string())
                    })?;
                    self.stack.push(val);
                }
                "2>R" => {
                    // ( n1 n2 -- ) Move two cells to return stack
                    let n2 = self.pop()?;
                    let n1 = self.pop()?;
                    self.return_stack.push(n1);
                    self.return_stack.push(n2);
                }
                "2R>" => {
                    // ( -- n1 n2 ) Move two cells from return stack
                    let n2 = self.return_stack.pop().ok_or_else(|| {
                        crate::error::CompileError::RuntimeError("Return stack underflow".to_string())
                    })?;
                    let n1 = self.return_stack.pop().ok_or_else(|| {
                        crate::error::CompileError::RuntimeError("Return stack underflow".to_string())
                    })?;
                    self.stack.push(n1);
                    self.stack.push(n2);
                }
                "2R@" => {
                    // ( -- n1 n2 ) Copy two cells from return stack
                    if self.return_stack.len() < 2 {
                        return Err(crate::error::CompileError::RuntimeError("Return stack underflow".to_string()));
                    }
                    let len = self.return_stack.len();
                    let n1 = self.return_stack[len - 2];
                    let n2 = self.return_stack[len - 1];
                    self.stack.push(n1);
                    self.stack.push(n2);
                }

                // PRIORITY 4: Base Conversion
                "DECIMAL" => {
                    self.base = 10;
                }
                "HEX" => {
                    self.base = 16;
                }
                "BINARY" => {
                    self.base = 2;
                }
                "OCTAL" => {
                    self.base = 8;
                }

                // Handle variable/constant/value references
                _ => {
                    let upper_token = token.to_uppercase();
                    // Check if it's a variable reference
                    if let Some(&addr) = self.variables.get(&upper_token) {
                        self.stack.push(addr);
                    }
                    // Check if it's a constant reference
                    else if let Some(&val) = self.constants.get(&upper_token) {
                        self.stack.push(val);
                    }
                    // Check if it's a value reference
                    else if let Some(&val) = self.values.get(&upper_token) {
                        self.stack.push(val);
                    }
                    // Ignore unknown words for now
                    // In a real implementation, this would error
                }
            }
        }

        Ok(())
    }

    /// Get the current stack
    pub fn stack(&self) -> &[i64] {
        &self.stack
    }

    /// Get the return stack
    pub fn return_stack(&self) -> &[i64] {
        &self.return_stack
    }

    /// Clear the stack
    pub fn clear_stack(&mut self) {
        self.stack.clear();
    }

    /// Get and clear output
    pub fn take_output(&mut self) -> String {
        std::mem::take(&mut self.output)
    }

    /// Get output without clearing
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Define a variable
    /// Returns the address of the variable
    pub fn define_variable(&mut self, name: &str) -> i64 {
        let addr = self.next_addr;
        self.next_addr += 8; // Increment by cell size
        self.variables.insert(name.to_uppercase(), addr);
        self.memory.insert(addr, 0); // Initialize to zero
        addr
    }

    /// Define a constant
    pub fn define_constant(&mut self, name: &str, value: i64) {
        self.constants.insert(name.to_uppercase(), value);
    }

    /// Define a value
    pub fn define_value(&mut self, name: &str, value: i64) {
        self.values.insert(name.to_uppercase(), value);
    }

    /// Update a value (TO word)
    pub fn update_value(&mut self, name: &str, new_value: i64) -> Result<()> {
        if let Some(val) = self.values.get_mut(&name.to_uppercase()) {
            *val = new_value;
            Ok(())
        } else {
            Err(crate::error::CompileError::RuntimeError(
                format!("Value '{}' not defined", name)
            ))
        }
    }

    /// Get memory value at address
    pub fn get_memory(&self, addr: i64) -> i64 {
        *self.memory.get(&addr).unwrap_or(&0)
    }

    /// Set memory value at address
    pub fn set_memory(&mut self, addr: i64, value: i64) {
        self.memory.insert(addr, value);
    }

    /// Get constants map (for debugging)
    pub fn constants(&self) -> &HashMap<String, i64> {
        &self.constants
    }

    /// Get values map (for debugging)
    pub fn values_map(&self) -> &HashMap<String, i64> {
        &self.values
    }

    fn pop(&mut self) -> Result<i64> {
        self.stack.pop().ok_or_else(|| {
            crate::error::CompileError::RuntimeError("Stack underflow".to_string())
        })
    }

    fn peek(&self) -> Result<i64> {
        self.stack.last().copied().ok_or_else(|| {
            crate::error::CompileError::RuntimeError("Stack underflow".to_string())
        })
    }
}

impl Default for ForthEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ForthEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForthEngine")
            .field("stack", &self.stack)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut engine = ForthEngine::new();
        engine.eval("5 10 +").unwrap();
        assert_eq!(engine.stack(), &[15]);
    }

    #[test]
    fn test_stack_operations() {
        let mut engine = ForthEngine::new();
        engine.eval("5 10 SWAP").unwrap();
        assert_eq!(engine.stack(), &[10, 5]);
    }

    #[test]
    fn test_dup() {
        let mut engine = ForthEngine::new();
        engine.eval("5 DUP").unwrap();
        assert_eq!(engine.stack(), &[5, 5]);
    }
}
