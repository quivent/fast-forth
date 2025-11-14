//! Stack Caching Optimizer
//!
//! Keeps the top N stack items in registers for dramatic performance improvements.
//! This is the most impactful optimization for stack-based code, typically
//! achieving 2-3x speedup on stack-heavy operations.
//!
//! # Algorithm
//!
//! 1. Track stack depth at each instruction
//! 2. Allocate registers for top N items (typically 3: TOS, NOS, 3OS)
//! 3. Transform instructions to use cached registers
//! 4. Insert flush/reload instructions at call boundaries
//!
//! # Register Allocation
//!
//! ```text
//! Stack:  [... | 3OS | NOS | TOS]
//! Regs:        r2    r1    r0
//! ```
//!
//! # Example Transformation
//!
//! Before:
//! ```forth
//! 1 2 + dup *
//! ```
//!
//! After (with stack caching):
//! ```assembly
//! mov r0, 1        ; literal -> r0 (TOS)
//! mov r1, 2        ; literal -> r1 (NOS), r0 -> r2 (3OS)
//! add r0, r1       ; + consumes r0,r1 -> r0
//! mov r1, r0       ; dup: r0 -> r1
//! mul r0, r1       ; * consumes r0,r1 -> r0
//! ```

use crate::ir::{ForthIR, Instruction, WordDef};
use crate::{OptimizerError, Result};
use smallvec::{SmallVec, smallvec};
use std::collections::HashMap;

/// Register identifier for stack cache
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Register(pub u8);

impl Register {
    pub const TOS: Self = Register(0);  // Top of stack
    pub const NOS: Self = Register(1);  // Next on stack
    pub const THIRD: Self = Register(2); // Third on stack
}

/// Stack cache state at a program point
#[derive(Debug, Clone, PartialEq)]
pub struct CacheState {
    /// Number of items currently cached in registers
    pub cached_depth: u8,
    /// Total stack depth (including non-cached items)
    pub total_depth: i32,
}

impl CacheState {
    pub fn new(cache_size: u8) -> Self {
        Self {
            cached_depth: 0,
            total_depth: 0,
        }
    }

    /// Check if stack item at given depth is cached
    pub fn is_cached(&self, depth: u8) -> bool {
        depth < self.cached_depth
    }

    /// Get register for stack item at depth
    pub fn get_register(&self, depth: u8) -> Option<Register> {
        if self.is_cached(depth) {
            Some(Register(depth))
        } else {
            None
        }
    }
}

/// Stack caching optimizer
pub struct StackCacheOptimizer {
    /// Number of stack items to cache in registers
    cache_size: u8,
}

impl StackCacheOptimizer {
    pub fn new(cache_size: u8) -> Self {
        assert!(cache_size > 0 && cache_size <= 8, "Cache size must be 1-8");
        Self { cache_size }
    }

    /// Optimize IR with stack caching
    pub fn optimize(&self, ir: &ForthIR) -> Result<ForthIR> {
        let mut optimized = ir.clone();

        // Optimize main sequence
        optimized.main = self.optimize_sequence(&ir.main)?;

        // Optimize each word
        for (name, word) in ir.words.iter() {
            let optimized_word = self.optimize_word(word)?;
            optimized.words.insert(name.clone(), optimized_word);
        }

        Ok(optimized)
    }

    /// Optimize a word definition
    fn optimize_word(&self, word: &WordDef) -> Result<WordDef> {
        let mut optimized = word.clone();
        optimized.instructions = self.optimize_sequence(&word.instructions)?;
        optimized.update();
        Ok(optimized)
    }

    /// Optimize a sequence of instructions
    fn optimize_sequence(&self, instructions: &[Instruction]) -> Result<Vec<Instruction>> {
        let mut result = Vec::with_capacity(instructions.len());
        let mut state = CacheState::new(self.cache_size);

        for inst in instructions {
            // Apply stack caching transformation
            let transformed = self.transform_instruction(inst, &mut state)?;
            result.extend(transformed);
        }

        // Flush cache at end if needed
        if state.cached_depth > 0 {
            result.push(Instruction::FlushCache);
        }

        Ok(result)
    }

    /// Transform a single instruction with stack cache awareness
    fn transform_instruction(
        &self,
        inst: &Instruction,
        state: &mut CacheState,
    ) -> Result<SmallVec<[Instruction; 4]>> {
        use Instruction::*;

        let mut result = SmallVec::new();

        match inst {
            // Literals: push to cache
            Literal(v) => {
                result.push(Literal(*v));
                self.push_cache(state);
            }

            FloatLiteral(v) => {
                result.push(FloatLiteral(*v));
                self.push_cache(state);
            }

            // Stack operations with caching
            Dup => {
                if state.cached_depth >= 1 {
                    result.push(CachedDup {
                        depth: state.cached_depth,
                    });
                    self.push_cache(state);
                } else {
                    result.push(Dup);
                    state.total_depth += 1;
                }
            }

            Drop => {
                if state.cached_depth >= 1 {
                    self.pop_cache(state);
                } else {
                    result.push(Drop);
                    state.total_depth -= 1;
                }
            }

            Swap => {
                if state.cached_depth >= 2 {
                    result.push(CachedSwap {
                        depth: state.cached_depth,
                    });
                    // Depth unchanged
                } else {
                    // Need to flush and reload
                    self.flush_cache(&mut result, state);
                    result.push(Swap);
                }
            }

            Over => {
                if state.cached_depth >= 2 {
                    result.push(CachedOver {
                        depth: state.cached_depth,
                    });
                    self.push_cache(state);
                } else {
                    self.flush_cache(&mut result, state);
                    result.push(Over);
                    state.total_depth += 1;
                }
            }

            // Arithmetic: operate on cached values
            Add | Sub | Mul | Div | Mod | And | Or | Xor | Eq | Ne | Lt | Le | Gt | Ge | Shl
            | Shr => {
                if state.cached_depth >= 2 {
                    result.push(inst.clone());
                    self.pop_cache(state); // Binary op: consume 2, produce 1
                } else {
                    self.flush_cache(&mut result, state);
                    result.push(inst.clone());
                    state.total_depth -= 1;
                }
            }

            // Unary operations
            Neg | Abs | Not | ZeroEq | ZeroLt | ZeroGt => {
                if state.cached_depth >= 1 {
                    result.push(inst.clone());
                    // Depth unchanged
                } else {
                    self.flush_cache(&mut result, state);
                    result.push(inst.clone());
                }
            }

            // Superinstructions
            DupAdd | DupMul => {
                if state.cached_depth >= 1 {
                    result.push(inst.clone());
                    // Net effect: consume 1, produce 1 (depth unchanged)
                } else {
                    self.flush_cache(&mut result, state);
                    result.push(inst.clone());
                }
            }

            // Control flow: flush cache
            Call(_) | Return | Branch(_) | BranchIf(_) | BranchIfNot(_) => {
                self.flush_cache(&mut result, state);
                result.push(inst.clone());
            }

            // Memory operations: flush cache for safety
            Store | Store8 | Load | Load8 => {
                self.flush_cache(&mut result, state);
                result.push(inst.clone());
                // Update depth
                let effect = inst.stack_effect();
                state.total_depth += effect.produced as i32 - effect.consumed as i32;
            }

            // Return stack operations: flush cache
            ToR | FromR | RFetch => {
                self.flush_cache(&mut result, state);
                result.push(inst.clone());
                let effect = inst.stack_effect();
                state.total_depth += effect.produced as i32 - effect.consumed as i32;
            }

            // Metadata: pass through
            Comment(_) | Label(_) | Nop | FlushCache => {
                result.push(inst.clone());
            }

            // Already cached instructions: pass through
            CachedDup { .. } | CachedSwap { .. } | CachedOver { .. } => {
                result.push(inst.clone());
            }

            _ => {
                // Default: flush cache and pass through
                self.flush_cache(&mut result, state);
                result.push(inst.clone());
                let effect = inst.stack_effect();
                state.total_depth += effect.produced as i32 - effect.consumed as i32;
            }
        }

        Ok(result)
    }

    /// Push an item to the cache
    fn push_cache(&self, state: &mut CacheState) {
        if state.cached_depth < self.cache_size {
            state.cached_depth += 1;
        }
        state.total_depth += 1;
    }

    /// Pop an item from the cache
    fn pop_cache(&self, state: &mut CacheState) {
        if state.cached_depth > 0 {
            state.cached_depth -= 1;
        }
        state.total_depth -= 1;
    }

    /// Flush cache to memory
    fn flush_cache(&self, result: &mut SmallVec<[Instruction; 4]>, state: &mut CacheState) {
        if state.cached_depth > 0 {
            result.push(Instruction::FlushCache);
            state.cached_depth = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_state() {
        let mut state = CacheState::new(3);
        assert_eq!(state.cached_depth, 0);
        assert_eq!(state.total_depth, 0);

        let optimizer = StackCacheOptimizer::new(3);
        optimizer.push_cache(&mut state);
        assert_eq!(state.cached_depth, 1);
        assert_eq!(state.total_depth, 1);

        optimizer.pop_cache(&mut state);
        assert_eq!(state.cached_depth, 0);
        assert_eq!(state.total_depth, 0);
    }

    #[test]
    fn test_optimize_literals() {
        let optimizer = StackCacheOptimizer::new(3);
        let ir = ForthIR::parse("1 2 3").unwrap();
        let optimized = optimizer.optimize(&ir).unwrap();

        // Should have literals plus flush at end
        assert!(optimized.main.len() >= 3);
    }

    #[test]
    fn test_optimize_dup_add() {
        let optimizer = StackCacheOptimizer::new(3);
        let ir = ForthIR::parse("5 dup +").unwrap();
        let optimized = optimizer.optimize(&ir).unwrap();

        // Should use cached operations
        let has_cached_dup = optimized
            .main
            .iter()
            .any(|i| matches!(i, Instruction::CachedDup { .. }));
        assert!(has_cached_dup);
    }

    #[test]
    fn test_flush_on_call() {
        let optimizer = StackCacheOptimizer::new(3);
        let mut ir = ForthIR::parse("1 2 foo").unwrap();
        // foo is a call
        let optimized = optimizer.optimize(&ir).unwrap();

        // Should have flush before call
        let has_flush = optimized
            .main
            .iter()
            .any(|i| matches!(i, Instruction::FlushCache));
        assert!(has_flush);
    }
}
