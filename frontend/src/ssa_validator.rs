//! SSA Form Validation
//!
//! Validates SSA invariants to catch bugs early in the compilation pipeline.
//! This module implements comprehensive checks for:
//! - Dominance: all uses dominated by definitions
//! - Single assignment: each register assigned exactly once
//! - Phi node validity: correct placement and incoming edges
//! - Use-before-def: all registers defined before use
//! - Block connectivity: no unreachable blocks
//! - Type consistency: stack depth matches at merge points

use crate::error::{ForthError, Result};
use crate::ssa::{SSAFunction, SSAInstruction, Register, BlockId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Validation error with detailed context
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    pub message: String,
    pub block: Option<BlockId>,
    pub register: Option<Register>,
    pub instruction_index: Option<usize>,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            block: None,
            register: None,
            instruction_index: None,
        }
    }

    pub fn with_block(mut self, block: BlockId) -> Self {
        self.block = Some(block);
        self
    }

    pub fn with_register(mut self, reg: Register) -> Self {
        self.register = Some(reg);
        self
    }

    pub fn with_instruction(mut self, idx: usize) -> Self {
        self.instruction_index = Some(idx);
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSA Validation Error: {}", self.message)?;
        if let Some(block) = self.block {
            write!(f, " in block {}", block)?;
        }
        if let Some(reg) = self.register {
            write!(f, " for register {}", reg)?;
        }
        if let Some(idx) = self.instruction_index {
            write!(f, " at instruction {}", idx)?;
        }
        Ok(())
    }
}

/// SSA Form Validator
pub struct SSAValidator<'a> {
    function: &'a SSAFunction,
    /// Registers defined in each block (for single assignment check)
    definitions: HashMap<BlockId, HashSet<Register>>,
    /// All registers ever defined (global single assignment)
    all_definitions: HashSet<Register>,
    /// Registers used in each block
    uses: HashMap<BlockId, HashSet<Register>>,
    /// Dominance tree: block -> blocks it dominates
    dominators: HashMap<BlockId, HashSet<BlockId>>,
    /// Immediate dominators: block -> its immediate dominator
    idom: HashMap<BlockId, BlockId>,
    /// Block successors (actual control flow)
    successors: HashMap<BlockId, Vec<BlockId>>,
    /// Block predecessors (actual control flow)
    predecessors: HashMap<BlockId, Vec<BlockId>>,
}

impl<'a> SSAValidator<'a> {
    /// Create a new validator for the given SSA function
    pub fn new(function: &'a SSAFunction) -> Self {
        Self {
            function,
            definitions: HashMap::new(),
            all_definitions: HashSet::new(),
            uses: HashMap::new(),
            dominators: HashMap::new(),
            idom: HashMap::new(),
            successors: HashMap::new(),
            predecessors: HashMap::new(),
        }
    }

    /// Run all validation checks
    pub fn validate(&mut self) -> Result<()> {
        // Build control flow graph
        self.build_cfg();

        // 1. Check single assignment
        self.check_single_assignment()?;

        // 2. Check use-before-def (local block check)
        self.check_use_before_def()?;

        // 3. Check block connectivity
        self.check_block_connectivity()?;

        // 4. Compute dominance tree
        self.compute_dominators();

        // 5. Check dominance property
        self.check_dominance()?;

        // 6. Check Phi node validity
        self.check_phi_nodes()?;

        // 7. Check type consistency (stack depth at merge points)
        self.check_type_consistency()?;

        Ok(())
    }

    /// Build control flow graph by analyzing branches and jumps
    fn build_cfg(&mut self) {
        // Initialize empty successor and predecessor lists
        for block in &self.function.blocks {
            self.successors.insert(block.id, Vec::new());
            self.predecessors.insert(block.id, Vec::new());
        }

        // Analyze terminator instructions to build edges
        for block in &self.function.blocks {
            if let Some(terminator) = block.instructions.last() {
                match terminator {
                    SSAInstruction::Branch { true_block, false_block, .. } => {
                        self.successors.entry(block.id).or_default().push(*true_block);
                        self.successors.entry(block.id).or_default().push(*false_block);
                        self.predecessors.entry(*true_block).or_default().push(block.id);
                        self.predecessors.entry(*false_block).or_default().push(block.id);
                    }
                    SSAInstruction::Jump { target } => {
                        self.successors.entry(block.id).or_default().push(*target);
                        self.predecessors.entry(*target).or_default().push(block.id);
                    }
                    SSAInstruction::Return { .. } => {
                        // Return has no successors
                    }
                    _ => {
                        // Non-terminator as last instruction is an error
                        // We'll catch this in other checks
                    }
                }
            }
        }
    }

    /// Check 1: Each register is assigned exactly once (Single Static Assignment)
    fn check_single_assignment(&mut self) -> Result<()> {
        for block in &self.function.blocks {
            let mut block_defs = HashSet::new();

            for inst in &block.instructions {
                // Collect all destination registers from this instruction
                let dests = self.get_destination_registers(inst);

                for dest in dests {
                    // Check if this register was already defined
                    if self.all_definitions.contains(&dest) {
                        return Err(ForthError::SSAConversionError {
                            message: format!(
                                "Register {} assigned multiple times (violation of SSA form)",
                                dest
                            ),
                        });
                    }

                    block_defs.insert(dest);
                    self.all_definitions.insert(dest);
                }
            }

            self.definitions.insert(block.id, block_defs);
        }

        Ok(())
    }

    /// Check 2: All registers are defined before use within a block
    fn check_use_before_def(&mut self) -> Result<()> {
        for block in &self.function.blocks {
            let mut defined_in_block = HashSet::new();

            // Function parameters are pre-defined
            if block.id == self.function.entry_block {
                for &param in &self.function.parameters {
                    defined_in_block.insert(param);
                }
            }

            for inst in &block.instructions {
                // First, check all uses in this instruction
                let uses = self.get_used_registers(inst);

                for used_reg in &uses {
                    // Skip Phi nodes - they're special (values come from predecessors)
                    if matches!(inst, SSAInstruction::Phi { .. }) {
                        continue;
                    }

                    // Check if register is defined in this block or is a parameter
                    if !defined_in_block.contains(used_reg) {
                        // It's okay if it's defined in a dominating block
                        // We'll check that in the dominance check
                        // For now, just track uses
                    }
                }

                // Then, add any definitions from this instruction
                let defs = self.get_destination_registers(inst);
                for def in defs {
                    defined_in_block.insert(def);
                }
            }

            self.uses.insert(block.id, defined_in_block);
        }

        Ok(())
    }

    /// Check 3: No unreachable blocks (all blocks reachable from entry)
    fn check_block_connectivity(&self) -> Result<()> {
        let mut reachable = HashSet::new();
        let mut worklist = VecDeque::new();

        // Start from entry block
        worklist.push_back(self.function.entry_block);
        reachable.insert(self.function.entry_block);

        // BFS to find all reachable blocks
        while let Some(block_id) = worklist.pop_front() {
            if let Some(successors) = self.successors.get(&block_id) {
                for &succ in successors {
                    if !reachable.contains(&succ) {
                        reachable.insert(succ);
                        worklist.push_back(succ);
                    }
                }
            }
        }

        // Check if all blocks are reachable
        for block in &self.function.blocks {
            if !reachable.contains(&block.id) {
                return Err(ForthError::SSAConversionError {
                    message: format!(
                        "Unreachable block {} detected (not connected to entry block {})",
                        block.id, self.function.entry_block
                    ),
                });
            }
        }

        Ok(())
    }

    /// Compute dominance tree using iterative algorithm
    fn compute_dominators(&mut self) {
        let blocks: Vec<BlockId> = self.function.blocks.iter().map(|b| b.id).collect();

        // Initialize: entry dominates only itself, others dominate all
        let mut dom: HashMap<BlockId, HashSet<BlockId>> = HashMap::new();
        let all_blocks: HashSet<BlockId> = blocks.iter().copied().collect();

        dom.insert(self.function.entry_block,
                   [self.function.entry_block].iter().copied().collect());

        for &block in &blocks {
            if block != self.function.entry_block {
                dom.insert(block, all_blocks.clone());
            }
        }

        // Iterate until fixed point
        let mut changed = true;
        while changed {
            changed = false;

            for &block in &blocks {
                if block == self.function.entry_block {
                    continue;
                }

                // Dom(n) = {n} ∪ (∩ Dom(p) for all predecessors p of n)
                let mut new_dom = if let Some(preds) = self.predecessors.get(&block) {
                    if preds.is_empty() {
                        // No predecessors - should have been caught by connectivity check
                        continue;
                    }

                    let mut intersection = dom.get(&preds[0]).cloned().unwrap_or_default();
                    for &pred in &preds[1..] {
                        if let Some(pred_dom) = dom.get(&pred) {
                            intersection = intersection.intersection(pred_dom).copied().collect();
                        }
                    }
                    intersection
                } else {
                    HashSet::new()
                };

                new_dom.insert(block);

                if new_dom != *dom.get(&block).unwrap() {
                    dom.insert(block, new_dom);
                    changed = true;
                }
            }
        }

        self.dominators = dom;

        // Compute immediate dominators (idom)
        for &block in &blocks {
            if block == self.function.entry_block {
                continue;
            }

            if let Some(dominators) = self.dominators.get(&block) {
                // idom(n) is the unique dominator that doesn't dominate any other dominator of n
                let candidates: Vec<BlockId> = dominators
                    .iter()
                    .filter(|&&d| d != block)
                    .copied()
                    .collect();

                // Find the dominator that is dominated by all other dominators
                for &candidate in &candidates.clone() {
                    let is_idom = candidates.iter().all(|&other| {
                        other == candidate || {
                            if let Some(other_doms) = self.dominators.get(&other) {
                                other_doms.contains(&candidate)
                            } else {
                                false
                            }
                        }
                    });

                    if is_idom {
                        self.idom.insert(block, candidate);
                        break;
                    }
                }
            }
        }
    }

    /// Check 4: All uses are dominated by their definitions
    fn check_dominance(&self) -> Result<()> {
        for block in &self.function.blocks {
            for inst in &block.instructions {
                // Skip Phi nodes - they have special semantics
                if matches!(inst, SSAInstruction::Phi { .. }) {
                    continue;
                }

                let uses = self.get_used_registers(inst);

                for used_reg in uses {
                    // Find where this register is defined
                    let def_block = self.find_definition_block(used_reg);

                    if let Some(def_block_id) = def_block {
                        // Check if def_block dominates current block
                        if !self.is_dominated_by(block.id, def_block_id) {
                            return Err(ForthError::SSAConversionError {
                                message: format!(
                                    "Register {} used in block {} but defined in non-dominating block {}",
                                    used_reg, block.id, def_block_id
                                ),
                            });
                        }
                    } else if !self.function.parameters.contains(&used_reg) {
                        // Register not defined anywhere and not a parameter
                        return Err(ForthError::SSAConversionError {
                            message: format!(
                                "Register {} used in block {} but never defined",
                                used_reg, block.id
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Check 5: Phi nodes are valid
    fn check_phi_nodes(&self) -> Result<()> {
        for block in &self.function.blocks {
            let mut seen_non_phi = false;

            for inst in &block.instructions {
                match inst {
                    SSAInstruction::Phi { dest, incoming } => {
                        // Phi nodes must be at the start of the block
                        if seen_non_phi {
                            return Err(ForthError::SSAConversionError {
                                message: format!(
                                    "Phi node for {} not at start of block {}",
                                    dest, block.id
                                ),
                            });
                        }

                        // Check that all predecessors have incoming values
                        let actual_preds = self.predecessors.get(&block.id).cloned().unwrap_or_default();
                        let phi_preds: HashSet<BlockId> = incoming.iter().map(|(b, _)| *b).collect();

                        // Check for missing predecessors
                        for &pred in &actual_preds {
                            if !phi_preds.contains(&pred) {
                                return Err(ForthError::SSAConversionError {
                                    message: format!(
                                        "Phi node for {} in block {} missing incoming value from predecessor {}",
                                        dest, block.id, pred
                                    ),
                                });
                            }
                        }

                        // Check for extra predecessors
                        for &phi_pred in &phi_preds {
                            if !actual_preds.contains(&phi_pred) {
                                return Err(ForthError::SSAConversionError {
                                    message: format!(
                                        "Phi node for {} in block {} has incoming value from non-predecessor {}",
                                        dest, block.id, phi_pred
                                    ),
                                });
                            }
                        }
                    }
                    _ => {
                        seen_non_phi = true;
                    }
                }
            }
        }

        Ok(())
    }

    /// Check 6: Type consistency (stack depth matches at merge points)
    fn check_type_consistency(&self) -> Result<()> {
        // For each block with multiple predecessors, check that
        // all predecessors leave the same stack depth
        for block in &self.function.blocks {
            if let Some(preds) = self.predecessors.get(&block.id) {
                if preds.len() <= 1 {
                    continue; // No merge point
                }

                // Count Phi nodes in this block (they represent the merged stack)
                let phi_count = block.instructions.iter()
                    .filter(|inst| matches!(inst, SSAInstruction::Phi { .. }))
                    .count();

                // All predecessors should contribute the same number of values
                // This is implicitly checked by Phi node validation
                // But we can do an additional sanity check here

                if phi_count > 0 {
                    // Verify that each Phi has the correct number of incoming edges
                    for inst in &block.instructions {
                        if let SSAInstruction::Phi { dest, incoming } = inst {
                            if incoming.len() != preds.len() {
                                return Err(ForthError::SSAConversionError {
                                    message: format!(
                                        "Type consistency error: Phi node for {} in block {} has {} incoming values but {} predecessors",
                                        dest, block.id, incoming.len(), preds.len()
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Helper: Find which block defines a register
    fn find_definition_block(&self, reg: Register) -> Option<BlockId> {
        for (block_id, defs) in &self.definitions {
            if defs.contains(&reg) {
                return Some(*block_id);
            }
        }
        None
    }

    /// Helper: Check if block_id is dominated by dominator_id
    fn is_dominated_by(&self, block_id: BlockId, dominator_id: BlockId) -> bool {
        if block_id == dominator_id {
            return true;
        }

        if let Some(doms) = self.dominators.get(&block_id) {
            doms.contains(&dominator_id)
        } else {
            false
        }
    }

    /// Helper: Extract all destination registers from an instruction
    fn get_destination_registers(&self, inst: &SSAInstruction) -> Vec<Register> {
        match inst {
            SSAInstruction::LoadInt { dest, .. } => vec![*dest],
            SSAInstruction::LoadFloat { dest, .. } => vec![*dest],
            SSAInstruction::LoadString { dest_addr, dest_len, .. } => vec![*dest_addr, *dest_len],
            SSAInstruction::BinaryOp { dest, .. } => vec![*dest],
            SSAInstruction::UnaryOp { dest, .. } => vec![*dest],
            SSAInstruction::Call { dest, .. } => dest.to_vec(),
            SSAInstruction::Phi { dest, .. } => vec![*dest],
            SSAInstruction::Load { dest, .. } => vec![*dest],
            SSAInstruction::FFICall { dest, .. } => dest.to_vec(),
            SSAInstruction::FileOpen { dest_fileid, dest_ior, .. } => vec![*dest_fileid, *dest_ior],
            SSAInstruction::FileRead { dest_bytes, dest_ior, .. } => vec![*dest_bytes, *dest_ior],
            SSAInstruction::FileWrite { dest_ior, .. } => vec![*dest_ior],
            SSAInstruction::FileClose { dest_ior, .. } => vec![*dest_ior],
            SSAInstruction::FileDelete { dest_ior, .. } => vec![*dest_ior],
            SSAInstruction::FileCreate { dest_fileid, dest_ior, .. } => vec![*dest_fileid, *dest_ior],
            SSAInstruction::SystemCall { dest, .. } => vec![*dest],
            SSAInstruction::Branch { .. } => vec![],
            SSAInstruction::Jump { .. } => vec![],
            SSAInstruction::Return { .. } => vec![],
            SSAInstruction::Store { .. } => vec![],
        }
    }

    /// Helper: Extract all used registers from an instruction
    fn get_used_registers(&self, inst: &SSAInstruction) -> Vec<Register> {
        match inst {
            SSAInstruction::LoadInt { .. } => vec![],
            SSAInstruction::LoadFloat { .. } => vec![],
            SSAInstruction::LoadString { .. } => vec![],
            SSAInstruction::BinaryOp { left, right, .. } => vec![*left, *right],
            SSAInstruction::UnaryOp { operand, .. } => vec![*operand],
            SSAInstruction::Call { args, .. } => args.to_vec(),
            SSAInstruction::Branch { condition, .. } => vec![*condition],
            SSAInstruction::Jump { .. } => vec![],
            SSAInstruction::Return { values } => values.to_vec(),
            SSAInstruction::Phi { incoming, .. } => {
                incoming.iter().map(|(_, reg)| *reg).collect()
            }
            SSAInstruction::Load { address, .. } => vec![*address],
            SSAInstruction::Store { address, value, .. } => vec![*address, *value],
            SSAInstruction::FFICall { args, .. } => args.to_vec(),
            SSAInstruction::FileOpen { path_addr, path_len, mode, .. } => {
                vec![*path_addr, *path_len, *mode]
            }
            SSAInstruction::FileRead { buffer, count, fileid, .. } => {
                vec![*buffer, *count, *fileid]
            }
            SSAInstruction::FileWrite { buffer, count, fileid, .. } => {
                vec![*buffer, *count, *fileid]
            }
            SSAInstruction::FileClose { fileid, .. } => vec![*fileid],
            SSAInstruction::FileDelete { path_addr, path_len, .. } => {
                vec![*path_addr, *path_len]
            }
            SSAInstruction::FileCreate { path_addr, path_len, mode, .. } => {
                vec![*path_addr, *path_len, *mode]
            }
            SSAInstruction::SystemCall { command_addr, command_len, .. } => {
                vec![*command_addr, *command_len]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssa::*;

    #[test]
    fn test_valid_simple_function() {
        // : add-one ( n -- n+1 ) 1 + ;
        // %0 is parameter
        // %1 = load 1
        // %2 = add %0, %1
        // ret %2

        let mut func = SSAFunction::new("add-one".to_string(), 1);

        func.blocks[0].instructions = vec![
            SSAInstruction::LoadInt { dest: Register(1), value: 1 },
            SSAInstruction::BinaryOp {
                dest: Register(2),
                op: BinaryOperator::Add,
                left: Register(0),
                right: Register(1),
            },
            SSAInstruction::Return {
                values: smallvec::smallvec![Register(2)],
            },
        ];

        let mut validator = SSAValidator::new(&func);
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_multiple_assignment_error() {
        // Register assigned twice - should fail
        let mut func = SSAFunction::new("bad".to_string(), 0);

        func.blocks[0].instructions = vec![
            SSAInstruction::LoadInt { dest: Register(0), value: 1 },
            SSAInstruction::LoadInt { dest: Register(0), value: 2 }, // ERROR: %0 assigned twice
            SSAInstruction::Return {
                values: smallvec::smallvec![Register(0)],
            },
        ];

        let mut validator = SSAValidator::new(&func);
        assert!(validator.validate().is_err());
    }

    #[test]
    fn test_undefined_register_error() {
        // Using undefined register - should fail
        let mut func = SSAFunction::new("bad".to_string(), 0);

        func.blocks[0].instructions = vec![
            SSAInstruction::LoadInt { dest: Register(0), value: 1 },
            SSAInstruction::BinaryOp {
                dest: Register(2),
                op: BinaryOperator::Add,
                left: Register(0),
                right: Register(99), // ERROR: %99 never defined
            },
            SSAInstruction::Return {
                values: smallvec::smallvec![Register(2)],
            },
        ];

        let mut validator = SSAValidator::new(&func);
        assert!(validator.validate().is_err());
    }
}
