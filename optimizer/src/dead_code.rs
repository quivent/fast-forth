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
        // First pass: remove trivial operations
        let mut result = self.remove_trivial_ops(instructions);

        // Perform liveness analysis on simplified code
        let liveness = self.analyze_liveness(&result);

        // Remove dead instructions
        let mut filtered = Vec::new();
        for (i, inst) in result.iter().enumerate() {
            if self.should_keep(inst, i, &liveness) {
                filtered.push(inst.clone());
            }
        }

        Ok(filtered)
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

        // Calculate final stack depth to mark final values as live
        let mut final_stack_depth = 0i32;
        for inst in instructions.iter() {
            let effect = inst.stack_effect();
            final_stack_depth += effect.produced as i32 - effect.consumed as i32;
        }

        // Backward pass to propagate liveness
        let mut changed = true;
        while changed {
            changed = false;
            let mut stack_depth = final_stack_depth; // Start with final depth

            for (i, inst) in instructions.iter().enumerate().rev() {
                let effect = inst.stack_effect();

                // If this instruction produces values and stack is needed, it's live
                if effect.produced > 0 && stack_depth > 0 {
                    if !live.contains(&i) {
                        live.insert(i);
                        changed = true;
                    }
                }

                // Update stack depth (going backwards)
                stack_depth -= effect.produced as i32;
                stack_depth += effect.consumed as i32;
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
                // dup drop -> (remove both)
                [Instruction::Dup, Instruction::Drop, ..] => {
                    i += 2; // Skip both
                }

                // swap swap -> (remove both)
                [Instruction::Swap, Instruction::Swap, ..] => {
                    i += 2;
                }

                // over drop drop -> (remove all)
                [Instruction::Over, Instruction::Drop, Instruction::Drop, ..] => {
                    i += 3;
                }

                // Nop -> (remove)
                [Instruction::Nop, ..] => {
                    i += 1;
                }

                // literal drop -> (remove both if no side effects)
                [Instruction::Literal(_), Instruction::Drop, ..] => {
                    i += 2;
                }

                // Binary operation followed by drop -> remove all 3 (if literals)
                [Instruction::Literal(_), Instruction::Literal(_), op, Instruction::Drop, ..]
                    if matches!(op, Instruction::Add | Instruction::Sub | Instruction::Mul
                                  | Instruction::Div | Instruction::Mod | Instruction::And
                                  | Instruction::Or | Instruction::Xor) => {
                    i += 4; // Skip all 4
                }

                // Keep instruction
                _ => {
                    result.push(instructions[i].clone());
                    i += 1;
                }
            }
        }

        result
    }

    /// Get statistics about dead code elimination
    pub fn get_stats(&self, before: &ForthIR, after: &ForthIR) -> EliminationStats {
        let before_count = before.instruction_count();
        let after_count = after.instruction_count();

        EliminationStats {
            before_instructions: before_count,
            after_instructions: after_count,
            instructions_eliminated: before_count.saturating_sub(after_count),
            reduction_percent: if before_count > 0 {
                ((before_count - after_count) as f64 / before_count as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

impl Default for DeadCodeEliminator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct EliminationStats {
    pub before_instructions: usize,
    pub after_instructions: usize,
    pub instructions_eliminated: usize,
    pub reduction_percent: f64,
}

impl std::fmt::Display for EliminationStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Before: {} instructions\n\
             After: {} instructions\n\
             Eliminated: {} instructions\n\
             Reduction: {:.1}%",
            self.before_instructions,
            self.after_instructions,
            self.instructions_eliminated,
            self.reduction_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eliminate_dup_drop() {
        let eliminator = DeadCodeEliminator::new();
        let mut ir = ForthIR::new();
        ir.main = vec![
            Instruction::Literal(5),
            Instruction::Dup,
            Instruction::Drop,
        ];

        let optimized = eliminator.eliminate(&ir).unwrap();

        // dup drop should be eliminated, leaving just the literal
        assert_eq!(optimized.main.len(), 1);
        assert!(matches!(optimized.main[0], Instruction::Literal(5)));
    }

    #[test]
    fn test_eliminate_swap_swap() {
        let eliminator = DeadCodeEliminator::new();
        let ir = ForthIR::parse("1 2 swap swap +").unwrap();
        let optimized = eliminator.eliminate(&ir).unwrap();

        // swap swap should be eliminated
        let swap_count = optimized
            .main
            .iter()
            .filter(|i| matches!(i, Instruction::Swap))
            .count();
        assert_eq!(swap_count, 0);
    }

    #[test]
    fn test_eliminate_literal_drop() {
        let eliminator = DeadCodeEliminator::new();
        let mut ir = ForthIR::new();
        ir.main = vec![
            Instruction::Literal(42),
            Instruction::Drop,
            Instruction::Literal(5),
        ];

        let optimized = eliminator.eliminate(&ir).unwrap();

        // Literal followed by drop should be eliminated
        assert_eq!(optimized.main.len(), 1);
        assert!(matches!(optimized.main[0], Instruction::Literal(5)));
    }

    #[test]
    fn test_keep_side_effects() {
        let eliminator = DeadCodeEliminator::new();
        let mut ir = ForthIR::new();
        ir.main = vec![
            Instruction::Literal(42),
            Instruction::Literal(100),
            Instruction::Store, // Side effect!
        ];

        let optimized = eliminator.eliminate(&ir).unwrap();

        // Store has side effect, should be kept
        let has_store = optimized
            .main
            .iter()
            .any(|i| matches!(i, Instruction::Store));
        assert!(has_store);
    }

    #[test]
    fn test_eliminate_unused_computation() {
        let eliminator = DeadCodeEliminator::new();
        let mut ir = ForthIR::new();
        ir.main = vec![
            Instruction::Literal(1),
            Instruction::Literal(2),
            Instruction::Add,
            Instruction::Drop,
            Instruction::Literal(5),
        ];

        let optimized = eliminator.eliminate(&ir).unwrap();

        // The computation 1 2 + is unused (dropped), should be eliminated
        assert!(optimized.main.len() <= 2);
    }

    #[test]
    fn test_elimination_stats() {
        let eliminator = DeadCodeEliminator::new();
        let ir = ForthIR::parse("1 2 3 dup drop swap swap").unwrap();
        let optimized = eliminator.eliminate(&ir).unwrap();

        let stats = eliminator.get_stats(&ir, &optimized);
        assert!(stats.instructions_eliminated > 0);
    }
}
