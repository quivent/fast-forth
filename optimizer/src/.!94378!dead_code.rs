//! Dead Code Elimination for Stack Operations
//!
//! Removes unused stack values and pure operations whose results are never used.
//!
//! # Examples
//!
//! Before:
//! ```forth
//! 1 2 3 drop drop +
//! ```
//!
//! After:
//! ```forth
//! 1 2 +
//! ```
//!
//! Before:
//! ```forth
//! dup drop
//! ```
//!
//! After:
//! ```forth
//! (empty - identity operation)
//! ```

use crate::ir::{ForthIR, Instruction, WordDef};
use crate::Result;
use std::collections::HashSet;

/// Liveness analysis result
#[derive(Debug, Clone)]
struct LivenessInfo {
    /// Instructions that are live (their results are used)
    live_instructions: HashSet<usize>,
}

/// Dead code eliminator
pub struct DeadCodeEliminator {
    /// Enable aggressive elimination
    aggressive: bool,
}

impl DeadCodeEliminator {
    pub fn new() -> Self {
        Self { aggressive: true }
    }

    /// Eliminate dead code in IR
    pub fn eliminate(&self, ir: &ForthIR) -> Result<ForthIR> {
        let mut optimized = ir.clone();

        // Eliminate in main sequence
        optimized.main = self.eliminate_sequence(&ir.main)?;

        // Eliminate in each word
        for (name, word) in ir.words.iter() {
            let optimized_word = self.eliminate_word(word)?;
            optimized.words.insert(name.clone(), optimized_word);
        }

        Ok(optimized)
    }

    /// Eliminate dead code in a word definition
    fn eliminate_word(&self, word: &WordDef) -> Result<WordDef> {
        let mut optimized = word.clone();
        optimized.instructions = self.eliminate_sequence(&word.instructions)?;
        optimized.update();
        Ok(optimized)
    }

    /// Eliminate dead code in an instruction sequence
    fn eliminate_sequence(&self, instructions: &[Instruction]) -> Result<Vec<Instruction>> {
        // Perform liveness analysis
        let liveness = self.analyze_liveness(instructions);

        // Remove dead instructions
        let mut result = Vec::new();

        for (i, inst) in instructions.iter().enumerate() {
            if self.should_keep(inst, i, &liveness) {
                result.push(inst.clone());
            }
        }

        // Additional pass: remove trivial operations
        result = self.remove_trivial_ops(&result);

        Ok(result)
    }

    /// Analyze which instructions are live (their results are used)
    fn analyze_liveness(&self, instructions: &[Instruction]) -> LivenessInfo {
        let mut live = HashSet::new();

        // Simple backward analysis: mark instructions as live if they have side effects
        // or their results are used by live instructions
        for (i, inst) in instructions.iter().enumerate() {
            if !inst.is_pure() {
                // Instructions with side effects are always live
                live.insert(i);
            }
        }

        // Backward pass to propagate liveness
        let mut changed = true;
        while changed {
            changed = false;
            let mut stack_depth = 0i32;

            for (i, inst) in instructions.iter().enumerate().rev() {
                let effect = inst.stack_effect();

                // If this instruction produces values and stack is needed, it's live
                if effect.produced > 0 && stack_depth > 0 {
                    if !live.contains(&i) {
                        live.insert(i);
                        changed = true;
                    }
                }

                // Update stack depth
                stack_depth += effect.consumed as i32;
                stack_depth -= effect.produced as i32;
                stack_depth = stack_depth.max(0);
            }
        }

        LivenessInfo {
            live_instructions: live,
        }
    }

    /// Check if instruction should be kept
    fn should_keep(&self, inst: &Instruction, index: usize, liveness: &LivenessInfo) -> bool {
        use Instruction::*;

        // Always keep instructions with side effects
        if !inst.is_pure() {
            return true;
        }

        // Keep if live
        if liveness.live_instructions.contains(&index) {
            return true;
        }

        // Keep control flow and metadata
        match inst {
            Branch(_) | BranchIf(_) | BranchIfNot(_) | Return | Label(_) | Comment(_) => true,

            // Remove dead pure instructions
            _ if inst.is_pure() => false,

            _ => true,
        }
    }

    /// Remove trivial operations (second pass)
    fn remove_trivial_ops(&self, instructions: &[Instruction]) -> Vec<Instruction> {
        let mut result = Vec::new();
        let mut i = 0;

        while i < instructions.len() {
            match &instructions[i..] {
