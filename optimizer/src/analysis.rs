//! Data Flow Analysis for Stack-Based Code
//!
//! Provides various analysis passes for optimization:
//! - Stack depth tracking
//! - Reaching definitions
//! - Use-def chains
//! - Control flow graph construction

use crate::ir::{ForthIR, Instruction, WordDef};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

/// Stack depth at each program point
#[derive(Debug, Clone)]
pub struct StackDepthAnalysis {
    /// Depth at each instruction index
    pub depths: Vec<i32>,
    /// Maximum depth reached
    pub max_depth: i32,
}

impl StackDepthAnalysis {
    /// Analyze stack depths in a sequence
    pub fn analyze(instructions: &[Instruction]) -> Self {
        let mut depths = Vec::with_capacity(instructions.len());
        let mut current_depth = 0i32;
        let mut max_depth = 0i32;

        for inst in instructions {
            depths.push(current_depth);

            let effect = inst.stack_effect();
            current_depth -= effect.consumed as i32;
            current_depth += effect.produced as i32;
            max_depth = max_depth.max(current_depth);
        }

        Self { depths, max_depth }
    }

    /// Get depth at instruction index
    pub fn get_depth(&self, index: usize) -> Option<i32> {
        self.depths.get(index).copied()
    }
}

/// Control Flow Graph
#[derive(Debug, Clone)]
pub struct CFG {
    /// Graph of basic blocks
    pub graph: DiGraph<BasicBlock, EdgeType>,
    /// Entry node
    pub entry: NodeIndex,
    /// Exit nodes
    pub exits: Vec<NodeIndex>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    /// Instructions in this block
    pub instructions: Vec<Instruction>,
    /// Start index in original sequence
    pub start_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Unconditional,
    Conditional,
    Fallthrough,
}

impl CFG {
    /// Build CFG from instruction sequence
    pub fn build(instructions: &[Instruction]) -> Self {
        let mut graph = DiGraph::new();
        let mut blocks = Vec::new();
        let mut current_block = Vec::new();
        let mut block_start = 0;

        // Split into basic blocks
        for (i, inst) in instructions.iter().enumerate() {
            current_block.push(inst.clone());

            // End block on control flow or label
            let is_terminator = matches!(
                inst,
                Instruction::Branch(_)
                    | Instruction::BranchIf(_)
                    | Instruction::BranchIfNot(_)
                    | Instruction::Return
            );

            let is_label = matches!(inst, Instruction::Label(_));

            if is_terminator || is_label || i == instructions.len() - 1 {
                blocks.push(BasicBlock {
                    instructions: current_block.clone(),
                    start_index: block_start,
                });
                current_block.clear();
                block_start = i + 1;
            }
        }

        // Create nodes
        let nodes: Vec<_> = blocks.iter().map(|b| graph.add_node(b.clone())).collect();

        let entry = nodes.first().copied().unwrap_or_else(|| {
            graph.add_node(BasicBlock {
                instructions: vec![],
                start_index: 0,
            })
        });

        // TODO: Add edges based on control flow
        // This is simplified for now

        Self {
            graph,
            entry,
            exits: vec![],
        }
    }

    /// Get basic block at index
    pub fn get_block(&self, index: NodeIndex) -> Option<&BasicBlock> {
        self.graph.node_weight(index)
    }
}

/// Reaching definitions analysis
#[derive(Debug, Clone)]
pub struct ReachingDefinitions {
    /// Definitions that reach each instruction
    pub reaching: Vec<HashSet<usize>>,
}

impl ReachingDefinitions {
    /// Analyze reaching definitions
    pub fn analyze(instructions: &[Instruction]) -> Self {
        let mut reaching = vec![HashSet::new(); instructions.len()];

        // Simple forward dataflow analysis
        for (i, inst) in instructions.iter().enumerate() {
            if i > 0 {
                // Propagate from previous instruction
                reaching[i] = reaching[i - 1].clone();
            }

            // Add new definition if this instruction produces values
            let effect = inst.stack_effect();
            if effect.produced > 0 {
                reaching[i].insert(i);
            }
        }

        Self { reaching }
    }

    /// Get definitions reaching an instruction
    pub fn get_reaching(&self, index: usize) -> Option<&HashSet<usize>> {
        self.reaching.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_depth_analysis() {
        let instructions = vec![
            Instruction::Literal(1),   // depth: 0 -> 1
            Instruction::Literal(2),   // depth: 1 -> 2
            Instruction::Add,          // depth: 2 -> 1
        ];

        let analysis = StackDepthAnalysis::analyze(&instructions);

        assert_eq!(analysis.get_depth(0), Some(0));
        assert_eq!(analysis.get_depth(1), Some(1));
        assert_eq!(analysis.get_depth(2), Some(2));
        assert_eq!(analysis.max_depth, 2);
    }

    #[test]
    fn test_cfg_construction() {
        let instructions = vec![
            Instruction::Literal(1),
            Instruction::BranchIf(5),
            Instruction::Literal(2),
        ];

        let cfg = CFG::build(&instructions);

        // Should have at least one basic block
        assert!(cfg.graph.node_count() > 0);
    }

    #[test]
    fn test_reaching_definitions() {
        let instructions = vec![
            Instruction::Literal(1),   // def 0
            Instruction::Literal(2),   // def 1
            Instruction::Add,          // uses defs 0, 1
        ];

        let analysis = ReachingDefinitions::analyze(&instructions);

        // At instruction 2, definitions 0 and 1 should reach
        let reaching = analysis.get_reaching(2).unwrap();
        assert!(reaching.contains(&0));
        assert!(reaching.contains(&1));
    }
}
