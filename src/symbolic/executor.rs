//! Symbolic Executor
//!
//! Executes Forth code symbolically to analyze behavior

use super::{SymbolicValue, SymbolicStack, SymbolicError, Result};
use super::symbolic_value::{BinaryOperator, UnaryOperator};
use fastforth_frontend::{Word, Definition, Program};
use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;

/// Result of symbolic execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub initial_stack: Vec<String>,
    pub final_stack: Vec<String>,
    pub operations_count: usize,
    pub simplified: bool,
}

/// Symbolic executor for Forth code
pub struct SymbolicExecutor {
    stack: SymbolicStack,
    definitions: FxHashMap<String, Definition>,
    operations_count: usize,
    max_operations: usize,
}

impl SymbolicExecutor {
    pub fn new() -> Self {
        Self {
            stack: SymbolicStack::new(),
            definitions: FxHashMap::default(),
            operations_count: 0,
            max_operations: 10000,
        }
    }

    /// Execute a program symbolically
    pub fn execute_program(&mut self, program: &Program) -> Result<ExecutionResult> {
        // Register all definitions
        for def in &program.definitions {
            self.definitions.insert(def.name.clone(), def.clone());
        }

        // Execute top-level code
        for word in &program.top_level_code {
            self.execute_word(word)?;
        }

        Ok(self.get_result())
    }

    /// Execute a single word
    pub fn execute_word(&mut self, word: &Word) -> Result<()> {
        self.operations_count += 1;
        if self.operations_count > self.max_operations {
            return Err(SymbolicError::ExecutionLimitExceeded);
        }

        match word {
            Word::IntLiteral(n) => {
                self.stack.push(SymbolicValue::concrete(*n));
                Ok(())
            }

            Word::WordRef { name, .. } => {
                self.execute_builtin(name)
            }

            Word::If { then_branch, else_branch } => {
                let condition = self.stack.pop()
                    .ok_or(SymbolicError::StackUnderflow { required: 1, available: 0 })?;

                // Execute both branches and create conditional values
                let mut then_executor = self.clone();
                for word in then_branch {
                    then_executor.execute_word(word)?;
                }

                let mut else_executor = self.clone();
                if let Some(else_words) = else_branch {
                    for word in else_words {
                        else_executor.execute_word(word)?;
                    }
                }

                // Merge stacks with conditional values
                self.merge_conditional_stacks(condition, &then_executor.stack, &else_executor.stack);
                Ok(())
            }

            _ => Err(SymbolicError::UnsupportedOperation(format!("{:?}", word))),
        }
    }

    /// Execute a builtin word
    fn execute_builtin(&mut self, name: &str) -> Result<()> {
        match name {
            // Arithmetic
            "+" => self.binary_op(BinaryOperator::Add),
            "-" => self.binary_op(BinaryOperator::Sub),
            "*" => self.binary_op(BinaryOperator::Mul),
            "/" => self.binary_op(BinaryOperator::Div),
            "mod" => self.binary_op(BinaryOperator::Mod),

            // Comparison
            "<" => self.binary_op(BinaryOperator::Lt),
            ">" => self.binary_op(BinaryOperator::Gt),
            "=" => self.binary_op(BinaryOperator::Eq),
            "<=" => self.binary_op(BinaryOperator::Lte),
            ">=" => self.binary_op(BinaryOperator::Gte),
            "<>" => self.binary_op(BinaryOperator::Neq),

            // Logical
            "and" => self.binary_op(BinaryOperator::And),
            "or" => self.binary_op(BinaryOperator::Or),
            "not" => self.unary_op(UnaryOperator::Not),

            // Unary
            "negate" => self.unary_op(UnaryOperator::Negate),
            "abs" => self.unary_op(UnaryOperator::Abs),

            // Stack manipulation
            "dup" => {
                self.stack.dup()
                    .ok_or(SymbolicError::StackUnderflow { required: 1, available: 0 })?;
                Ok(())
            }
            "drop" => {
                self.stack.pop()
                    .ok_or(SymbolicError::StackUnderflow { required: 1, available: 0 })?;
                Ok(())
            }
            "swap" => {
                self.stack.swap()
                    .ok_or(SymbolicError::StackUnderflow { required: 2, available: self.stack.depth() })?;
                Ok(())
            }
            "over" => {
                self.stack.over()
                    .ok_or(SymbolicError::StackUnderflow { required: 2, available: self.stack.depth() })?;
                Ok(())
            }
            "rot" => {
                self.stack.rot()
                    .ok_or(SymbolicError::StackUnderflow { required: 3, available: self.stack.depth() })?;
                Ok(())
            }

            // User-defined words
            name => {
                if let Some(def) = self.definitions.get(name).cloned() {
                    for word in &def.body {
                        self.execute_word(word)?;
                    }
                    Ok(())
                } else {
                    Err(SymbolicError::UnknownWord(name.to_string()))
                }
            }
        }
    }

    /// Execute a binary operation
    fn binary_op(&mut self, op: BinaryOperator) -> Result<()> {
        let (a, b) = self.stack.pop2()
            .ok_or(SymbolicError::StackUnderflow { required: 2, available: self.stack.depth() })?;

        let result = SymbolicValue::binary_op(op, a, b).simplify();
        self.stack.push(result);
        Ok(())
    }

    /// Execute a unary operation
    fn unary_op(&mut self, op: UnaryOperator) -> Result<()> {
        let val = self.stack.pop()
            .ok_or(SymbolicError::StackUnderflow { required: 1, available: 0 })?;

        let result = SymbolicValue::unary_op(op, val).simplify();
        self.stack.push(result);
        Ok(())
    }

    /// Merge conditional stacks
    fn merge_conditional_stacks(
        &mut self,
        condition: SymbolicValue,
        then_stack: &SymbolicStack,
        else_stack: &SymbolicStack,
    ) {
        // For simplicity, use the then branch values
        // In a full implementation, create conditional symbolic values
        for val in then_stack.get_stack() {
            self.stack.push(val.clone());
        }
    }

    /// Get the execution result
    pub fn get_result(&self) -> ExecutionResult {
        ExecutionResult {
            initial_stack: vec![],
            final_stack: self.stack.get_stack().iter().map(|v| format!("{}", v)).collect(),
            operations_count: self.operations_count,
            simplified: true,
        }
    }

    /// Initialize stack with symbolic inputs
    pub fn initialize_inputs(&mut self, count: usize) {
        for i in 0..count {
            let var = SymbolicValue::variable(format!("in"), i);
            self.stack.push(var);
        }
    }
}

impl Clone for SymbolicExecutor {
    fn clone(&self) -> Self {
        Self {
            stack: self.stack.clone(),
            definitions: self.definitions.clone(),
            operations_count: self.operations_count,
            max_operations: self.max_operations,
        }
    }
}

impl Default for SymbolicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastforth_frontend::parse_program;

    #[test]
    fn test_execute_arithmetic() {
        let mut executor = SymbolicExecutor::new();
        executor.stack.push(SymbolicValue::concrete(2));
        executor.stack.push(SymbolicValue::concrete(3));

        executor.execute_builtin("+").unwrap();

        let result = executor.stack.pop().unwrap();
        assert_eq!(result, SymbolicValue::concrete(5));
    }

    #[test]
    fn test_execute_dup() {
        let mut executor = SymbolicExecutor::new();
        executor.stack.push(SymbolicValue::concrete(42));

        executor.execute_builtin("dup").unwrap();

        assert_eq!(executor.stack.depth(), 2);
        let val1 = executor.stack.pop().unwrap();
        let val2 = executor.stack.pop().unwrap();
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_symbolic_square() {
        let program = parse_program(": square dup * ; square").unwrap();
        let mut executor = SymbolicExecutor::new();

        executor.initialize_inputs(1);
        executor.execute_program(&program).unwrap();

        let result = executor.get_result();
        assert_eq!(result.final_stack.len(), 1);
        assert!(result.final_stack[0].contains("*"));
    }
}
