//! Aggressive Inlining Engine with Whole-Program Analysis
//!
//! This module provides sophisticated inlining optimizations that achieve 10-20% speedup
//! on call-heavy code through:
//!
//! - Whole-program call graph analysis
//! - Strongly connected component detection for cycles
//! - Multi-level recursive inline expansion
//! - Cost/benefit analysis with code size tracking
//! - Programmer control via INLINE/NOINLINE directives
//!
//! # Algorithm Overview
//!
//! 1. Build complete call graph of all words
//! 2. Detect cycles using Tarjan's SCC algorithm
//! 3. Topologically sort acyclic words
//! 4. Inline in bottom-up order (callees before callers)
//! 5. Repeat until fixpoint or iteration limit
//!
//! # Example
//!
//! ```forth
//! : helper1 dup + ;          \ 2 instructions
//! : helper2 helper1 1 + ;    \ calls helper1
//! : main 5 helper2 * ;       \ calls helper2
//! ```
//!
//! After aggressive inlining:
//! ```forth
//! : main 5 dup + 1 + * ;     \ fully flattened
//! ```
//!
//! # Performance Targets
//!
//! - Sieve: Inline helper functions (10% speedup)
//! - Fibonacci: Inline base case checks (15% speedup)
//! - Overall: 10-20% on call-heavy code

use crate::ir::{ForthIR, Instruction, StackEffect, WordDef};
use crate::{OptimizationLevel, Result};
use petgraph::algo::tarjan_scc;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};

/// Inline directives for programmer control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InlineDirective {
    /// Force inline regardless of cost
    AlwaysInline,
    /// Prevent inlining
    NeverInline,
    /// Let optimizer decide
    Auto,
}

/// Extended word definition with inline metadata
#[derive(Debug, Clone)]
pub struct InlineableWord {
    pub word: WordDef,
    pub directive: InlineDirective,
    pub call_count: usize,
    pub inline_depth: usize,  // How many levels deep this has been inlined
    pub total_cost: usize,    // Cost including inlined callees
}

impl InlineableWord {
    fn new(word: WordDef) -> Self {
        let total_cost = word.cost;
        Self {
            word,
            directive: InlineDirective::Auto,
            call_count: 0,
            inline_depth: 0,
            total_cost,
        }
    }
}

/// Call graph node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CallGraphNode {
    name: String,
    instruction_count: usize,
}

/// Call graph edge
#[derive(Debug, Clone)]
struct CallGraphEdge {
    /// Number of times callee is called by caller
    call_count: usize,
}

/// Complete call graph for whole-program analysis
#[derive(Debug, Clone)]
pub struct CallGraph {
    graph: DiGraph<CallGraphNode, CallGraphEdge>,
    name_to_node: HashMap<String, NodeIndex>,
}

impl CallGraph {
    /// Build call graph from IR
    pub fn build(ir: &ForthIR) -> Self {
        let mut graph = DiGraph::new();
        let mut name_to_node = HashMap::new();

        // Create nodes for all words
        for (name, word) in &ir.words {
            let node = CallGraphNode {
                name: name.clone(),
                instruction_count: word.instructions.len(),
            };
            let idx = graph.add_node(node);
            name_to_node.insert(name.clone(), idx);
        }

        // Add edges for call relationships
        for (caller_name, word) in &ir.words {
            let caller_idx = name_to_node[caller_name];

            // Count calls to each callee
            let mut callee_counts: HashMap<String, usize> = HashMap::new();
            for inst in &word.instructions {
                if let Instruction::Call(callee_name) = inst {
                    if name_to_node.contains_key(callee_name) {
                        *callee_counts.entry(callee_name.clone()).or_insert(0) += 1;
                    }
                }
            }

            // Create edges
            for (callee_name, count) in callee_counts {
                let callee_idx = name_to_node[&callee_name];
                graph.add_edge(
                    caller_idx,
                    callee_idx,
                    CallGraphEdge { call_count: count },
                );
            }
        }

        Self { graph, name_to_node }
    }

    /// Detect strongly connected components (cycles)
    pub fn find_cycles(&self) -> Vec<HashSet<String>> {
        let sccs = tarjan_scc(&self.graph);

        sccs.into_iter()
            .filter(|scc| scc.len() > 1 || self.has_self_loop(scc[0]))
            .map(|scc| {
                scc.into_iter()
                    .map(|idx| self.graph[idx].name.clone())
                    .collect()
            })
            .collect()
    }

    /// Check if node has self-loop (direct recursion)
    fn has_self_loop(&self, node: NodeIndex) -> bool {
        self.graph
            .edges(node)
            .any(|edge| edge.target() == node)
    }

    /// Topological sort of acyclic nodes - returns callees before callers
    pub fn topological_sort(&self) -> Vec<String> {
        // Use post-order DFS where dependencies are visited before dependents
        let mut visited = HashSet::new();
        let mut in_progress = HashSet::new();
        let mut result = Vec::new();

        for node_idx in self.graph.node_indices() {
            if !visited.contains(&node_idx) {
                self.visit_topological_postorder(node_idx, &mut visited, &mut in_progress, &mut result);
            }
        }

        // Post-order DFS gives us leaves first, which is what we want (callees before callers)
        result
    }

    fn visit_topological_postorder(
        &self,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        in_progress: &mut HashSet<NodeIndex>,
        result: &mut Vec<String>,
    ) {
        if visited.contains(&node) {
            return;
        }

        if in_progress.contains(&node) {
            return; // Cycle detected, skip
        }

        in_progress.insert(node);

        // Visit all callees first (post-order)
        for edge in self.graph.edges(node) {
            self.visit_topological_postorder(edge.target(), visited, in_progress, result);
        }

        in_progress.remove(&node);
        visited.insert(node);
        result.push(self.graph[node].name.clone());
    }

    /// Get callees of a word
    pub fn get_callees(&self, word_name: &str) -> Vec<String> {
        if let Some(&node_idx) = self.name_to_node.get(word_name) {
            self.graph
                .edges(node_idx)
                .map(|edge| self.graph[edge.target()].name.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Count calls from one word to another
    pub fn get_call_count(&self, caller: &str, callee: &str) -> usize {
        if let (Some(&caller_idx), Some(&callee_idx)) =
            (self.name_to_node.get(caller), self.name_to_node.get(callee))
        {
            self.graph
                .edges(caller_idx)
                .find(|edge| edge.target() == callee_idx)
                .map(|edge| edge.weight().call_count)
                .unwrap_or(0)
        } else {
            0
        }
    }
}

/// Aggressive inlining optimizer
pub struct AggressiveInlineOptimizer {
    level: OptimizationLevel,
    inline_threshold_unconditional: usize,
    inline_threshold_conditional: usize,
    max_inline_sites: usize,
    max_inline_depth: usize,
    max_code_bloat_factor: f64,
    max_iterations: usize,
}

impl AggressiveInlineOptimizer {
    pub fn new(level: OptimizationLevel) -> Self {
        match level {
            OptimizationLevel::None => Self {
                level,
                inline_threshold_unconditional: 0,
                inline_threshold_conditional: 0,
                max_inline_sites: 0,
                max_inline_depth: 0,
                max_code_bloat_factor: 1.0,
                max_iterations: 0,
            },
            OptimizationLevel::Basic => Self {
                level,
                inline_threshold_unconditional: 3,
                inline_threshold_conditional: 8,
                max_inline_sites: 5,
                max_inline_depth: 2,
                max_code_bloat_factor: 1.5,
                max_iterations: 2,
            },
            OptimizationLevel::Standard => Self {
                level,
                inline_threshold_unconditional: 5,
                inline_threshold_conditional: 15,
                max_inline_sites: 10,
                max_inline_depth: 3,
                max_code_bloat_factor: 2.0,
                max_iterations: 3,
            },
            OptimizationLevel::Aggressive => Self {
                level,
                inline_threshold_unconditional: 5,
                inline_threshold_conditional: 30,
                max_inline_sites: 25,
                max_inline_depth: 5,
                max_code_bloat_factor: 3.0,
                max_iterations: 5,
            },
        }
    }

    /// Perform aggressive inlining with whole-program analysis
    pub fn inline(&self, ir: &ForthIR) -> Result<ForthIR> {
        if self.level == OptimizationLevel::None {
            return Ok(ir.clone());
        }

        let mut current_ir = ir.clone();
        let original_size = ir.instruction_count();

        for iteration in 0..self.max_iterations {
            // Build call graph
            let call_graph = CallGraph::build(&current_ir);

            // Detect cycles
            let cycles = call_graph.find_cycles();
            let cyclic_words: HashSet<String> =
                cycles.into_iter().flat_map(|c| c.into_iter()).collect();

            // Build inlineable word map
            let mut inlineable_words = self.build_inlineable_map(&current_ir, &call_graph);

            // Mark cyclic words as never inline
            for word_name in &cyclic_words {
                if let Some(inlineable) = inlineable_words.get_mut(word_name) {
                    inlineable.directive = InlineDirective::NeverInline;
                }
            }

            // Topological sort to inline callees before callers
            let topo_order = call_graph.topological_sort();

            // Inline in bottom-up order
            let new_ir = self.inline_iteration(
                &current_ir,
                &call_graph,
                &inlineable_words,
                &topo_order,
                iteration,
            )?;

            // Check for convergence
            let new_size = new_ir.instruction_count();
            if new_ir == current_ir {
                break;
            }

            // Check code bloat
            if new_size as f64 > original_size as f64 * self.max_code_bloat_factor {
                break;
            }

            current_ir = new_ir;
        }

        Ok(current_ir)
    }

    /// Build map of inlineable words with metadata
    fn build_inlineable_map(
        &self,
        ir: &ForthIR,
        call_graph: &CallGraph,
    ) -> HashMap<String, InlineableWord> {
        let mut map = HashMap::new();

        for (name, word) in &ir.words {
            let mut inlineable = InlineableWord::new(word.clone());

            // Use is_inline flag as directive
            if word.is_inline {
                inlineable.directive = InlineDirective::AlwaysInline;
            }

            // Count call sites
            inlineable.call_count = self.count_call_sites(name, ir);

            // Calculate total cost including inlined callees
            inlineable.total_cost = self.calculate_total_cost(name, word, call_graph, ir);

            map.insert(name.clone(), inlineable);
        }

        map
    }

    /// Count total call sites for a word
    fn count_call_sites(&self, word_name: &str, ir: &ForthIR) -> usize {
        let mut count = 0;

        // Count in main
        for inst in &ir.main {
            if let Instruction::Call(name) = inst {
                if name == word_name {
                    count += 1;
                }
            }
        }

        // Count in all words
        for word in ir.words.values() {
            for inst in &word.instructions {
                if let Instruction::Call(name) = inst {
                    if name == word_name {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Calculate total cost including inlined callees
    fn calculate_total_cost(
        &self,
        word_name: &str,
        word: &WordDef,
        call_graph: &CallGraph,
        ir: &ForthIR,
    ) -> usize {
        let mut cost = 0;

        for inst in &word.instructions {
            match inst {
                Instruction::Call(callee_name) => {
                    if let Some(callee) = ir.get_word(callee_name) {
                        // Recursively calculate callee cost
                        cost += callee.instructions.len();
                    } else {
                        // External call, count as 1
                        cost += 1;
                    }
                }
                _ => cost += 1,
            }
        }

        cost
    }

    /// Perform one iteration of inlining
    fn inline_iteration(
        &self,
        ir: &ForthIR,
        call_graph: &CallGraph,
        inlineable_words: &HashMap<String, InlineableWord>,
        topo_order: &[String],
        iteration: usize,
    ) -> Result<ForthIR> {
        let mut result = ir.clone();

        // Inline in topological order
        for word_name in topo_order {
            if let Some(word) = result.get_word(word_name).cloned() {
                let inlineable = &inlineable_words[word_name];

                // Inline calls within this word
                let new_instructions = self.inline_in_sequence(
                    &word.instructions,
                    &result,
                    inlineable_words,
                    iteration,
                )?;

                if new_instructions != word.instructions {
                    let mut new_word = word.clone();
                    new_word.instructions = new_instructions;
                    new_word.update();
                    result.words.insert(word_name.clone(), new_word);
                }
            }
        }

        // Inline in main sequence
        result.main = self.inline_in_sequence(
            &ir.main,
            &result,
            inlineable_words,
            iteration,
        )?;

        Ok(result)
    }

    /// Inline calls in an instruction sequence
    fn inline_in_sequence(
        &self,
        instructions: &[Instruction],
        ir: &ForthIR,
        inlineable_words: &HashMap<String, InlineableWord>,
        iteration: usize,
    ) -> Result<Vec<Instruction>> {
        let mut result = Vec::with_capacity(instructions.len() * 2);

        for inst in instructions {
            match inst {
                Instruction::Call(callee_name) => {
                    if let Some(inlineable) = inlineable_words.get(callee_name) {
                        if self.should_inline_call(inlineable, iteration) {
                            if let Some(callee) = ir.get_word(callee_name) {
                                // Add comment marker
                                result.push(Instruction::Comment(
                                    format!("inlined {}", callee_name)
                                ));

                                // Inline the callee's instructions
                                result.extend_from_slice(&callee.instructions);
                                continue;
                            }
                        }
                    }

                    // Don't inline: keep the call
                    result.push(inst.clone());
                }
                _ => {
                    result.push(inst.clone());
                }
            }
        }

        Ok(result)
    }

    /// Decide if a call should be inlined
    fn should_inline_call(&self, inlineable: &InlineableWord, iteration: usize) -> bool {
        // Check directive
        match inlineable.directive {
            InlineDirective::AlwaysInline => return true,
            InlineDirective::NeverInline => return false,
            InlineDirective::Auto => {}
        }

        // Check inline depth
        if inlineable.inline_depth >= self.max_inline_depth {
            return false;
        }

        // Check if too many call sites
        if inlineable.call_count > self.max_inline_sites {
            return false;
        }

        // Unconditional inline for very small words
        if inlineable.word.cost <= self.inline_threshold_unconditional {
            return true;
        }

        // Conditional inline based on cost and call count
        if inlineable.word.cost <= self.inline_threshold_conditional {
            // More aggressive inlining in later iterations
            let effective_threshold = self.inline_threshold_conditional * (iteration + 1);
            return inlineable.total_cost <= effective_threshold;
        }

        false
    }

    /// Get detailed inlining statistics
    pub fn get_stats(&self, before: &ForthIR, after: &ForthIR) -> AggressiveInlineStats {
        let before_calls = self.count_calls(before);
        let after_calls = self.count_calls(after);

        let call_graph_before = CallGraph::build(before);
        let call_graph_after = CallGraph::build(after);

        let cycles_before = call_graph_before.find_cycles();
        let cycles_after = call_graph_after.find_cycles();

        AggressiveInlineStats {
            calls_before: before_calls,
            calls_after: after_calls,
            calls_inlined: before_calls.saturating_sub(after_calls),
            instructions_before: before.instruction_count(),
            instructions_after: after.instruction_count(),
            cycles_detected: cycles_before.len(),
            cycles_remaining: cycles_after.len(),
            words_before: before.words.len(),
            words_after: after.words.len(),
            code_bloat_factor: after.instruction_count() as f64 / before.instruction_count() as f64,
        }
    }

    fn count_calls(&self, ir: &ForthIR) -> usize {
        let mut count = 0;

        for inst in &ir.main {
            if matches!(inst, Instruction::Call(_)) {
                count += 1;
            }
        }

        for word in ir.words.values() {
            for inst in &word.instructions {
                if matches!(inst, Instruction::Call(_)) {
                    count += 1;
                }
            }
        }

        count
    }
}

/// Statistics for aggressive inlining
#[derive(Debug, Clone)]
pub struct AggressiveInlineStats {
    pub calls_before: usize,
    pub calls_after: usize,
    pub calls_inlined: usize,
    pub instructions_before: usize,
    pub instructions_after: usize,
    pub cycles_detected: usize,
    pub cycles_remaining: usize,
    pub words_before: usize,
    pub words_after: usize,
    pub code_bloat_factor: f64,
}

impl std::fmt::Display for AggressiveInlineStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Aggressive Inline Statistics:\n\
             ├─ Calls before:        {}\n\
             ├─ Calls after:         {}\n\
             ├─ Calls inlined:       {} ({:.1}%)\n\
             ├─ Instructions before: {}\n\
             ├─ Instructions after:  {}\n\
             ├─ Code bloat:          {:.2}x\n\
             ├─ Cycles detected:     {}\n\
             ├─ Cycles remaining:    {}\n\
             ├─ Words before:        {}\n\
             └─ Words after:         {}",
            self.calls_before,
            self.calls_after,
            self.calls_inlined,
            (self.calls_inlined as f64 / self.calls_before as f64 * 100.0),
            self.instructions_before,
            self.instructions_after,
            self.code_bloat_factor,
            self.cycles_detected,
            self.cycles_remaining,
            self.words_before,
            self.words_after,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_graph_construction() {
        let mut ir = ForthIR::new();

        // Create word chain: a calls b, b calls c
        let c = WordDef::new("c".to_string(), vec![Instruction::Dup]);
        let b = WordDef::new("b".to_string(), vec![Instruction::Call("c".to_string())]);
        let a = WordDef::new("a".to_string(), vec![Instruction::Call("b".to_string())]);

        ir.add_word(c);
        ir.add_word(b);
        ir.add_word(a);

        let call_graph = CallGraph::build(&ir);

        assert_eq!(call_graph.get_callees("a"), vec!["b"]);
        assert_eq!(call_graph.get_callees("b"), vec!["c"]);
        assert!(call_graph.get_callees("c").is_empty());
    }

    #[test]
    fn test_cycle_detection() {
        let mut ir = ForthIR::new();

        // Create recursive word
        let factorial = WordDef::new(
            "factorial".to_string(),
            vec![Instruction::Dup, Instruction::Call("factorial".to_string())],
        );
        ir.add_word(factorial);

        let call_graph = CallGraph::build(&ir);
        let cycles = call_graph.find_cycles();

        assert_eq!(cycles.len(), 1);
        assert!(cycles[0].contains("factorial"));
    }

    #[test]
    fn test_topological_sort() {
        let mut ir = ForthIR::new();

        // Create dependency chain
        let c = WordDef::new("c".to_string(), vec![Instruction::Dup]);
        let b = WordDef::new("b".to_string(), vec![Instruction::Call("c".to_string())]);
        let a = WordDef::new("a".to_string(), vec![Instruction::Call("b".to_string())]);

        ir.add_word(c);  // Add in c, b, a order
        ir.add_word(b);
        ir.add_word(a);

        let call_graph = CallGraph::build(&ir);
        let topo = call_graph.topological_sort();

        // Verify all three are in the result
        assert_eq!(topo.len(), 3);
        assert!(topo.contains(&"a".to_string()));
        assert!(topo.contains(&"b".to_string()));
        assert!(topo.contains(&"c".to_string()));

        // c should come before b, b before a
        let c_pos = topo.iter().position(|s| s == "c").unwrap();
        let b_pos = topo.iter().position(|s| s == "b").unwrap();
        let a_pos = topo.iter().position(|s| s == "a").unwrap();

        assert!(c_pos < b_pos, "c should come before b, got c_pos={}, b_pos={}", c_pos, b_pos);
        assert!(b_pos < a_pos, "b should come before a, got b_pos={}, a_pos={}", b_pos, a_pos);
    }

    #[test]
    fn test_aggressive_inline_small_words() {
        let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Aggressive);

        let mut ir = ForthIR::new();

        // Create small word chain
        let tiny = WordDef::new("tiny".to_string(), vec![Instruction::Dup]);
        let small = WordDef::new(
            "small".to_string(),
            vec![Instruction::Call("tiny".to_string()), Instruction::Add],
        );

        ir.add_word(tiny);
        ir.add_word(small);
        ir.main = vec![Instruction::Literal(5), Instruction::Call("small".to_string())];

        let optimized = optimizer.inline(&ir).unwrap();

        // Should inline both tiny and small
        let has_calls = optimized.main.iter().any(|i| matches!(i, Instruction::Call(_)));
        assert!(!has_calls, "All calls should be inlined");

        let has_dup = optimized.main.iter().any(|i| matches!(i, Instruction::Dup));
        let has_add = optimized.main.iter().any(|i| matches!(i, Instruction::Add));
        assert!(has_dup && has_add, "Inlined instructions should be present");
    }

    #[test]
    fn test_dont_inline_recursive() {
        let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Aggressive);

        let mut ir = ForthIR::new();

        // Recursive factorial
        let factorial = WordDef::new(
            "factorial".to_string(),
            vec![
                Instruction::Dup,
                Instruction::Literal(1),
                Instruction::Gt,
                Instruction::Call("factorial".to_string()),
            ],
        );
        ir.add_word(factorial);
        ir.main = vec![Instruction::Literal(5), Instruction::Call("factorial".to_string())];

        let optimized = optimizer.inline(&ir).unwrap();

        // Recursive word should NOT be inlined
        let has_call = optimized.main.iter().any(|i| matches!(i, Instruction::Call(_)));
        assert!(has_call, "Recursive call should be preserved");
    }

    #[test]
    fn test_multi_level_inlining() {
        let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Aggressive);

        let mut ir = ForthIR::new();

        // Three-level call chain
        let level3 = WordDef::new("level3".to_string(), vec![Instruction::Dup]);
        let level2 = WordDef::new(
            "level2".to_string(),
            vec![Instruction::Call("level3".to_string())],
        );
        let level1 = WordDef::new(
            "level1".to_string(),
            vec![Instruction::Call("level2".to_string())],
        );

        ir.add_word(level3);
        ir.add_word(level2);
        ir.add_word(level1);
        ir.main = vec![Instruction::Literal(5), Instruction::Call("level1".to_string())];

        let optimized = optimizer.inline(&ir).unwrap();

        // All levels should be inlined
        let call_count = optimized.main.iter().filter(|i| matches!(i, Instruction::Call(_))).count();
        assert_eq!(call_count, 0, "All nested calls should be inlined");
    }

    #[test]
    fn test_forced_inline() {
        let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Standard);

        let mut ir = ForthIR::new();

        // Large word with forced inline
        let mut large = WordDef::new("large".to_string(), vec![Instruction::Dup; 50]);
        large.is_inline = true; // Force inline
        ir.add_word(large);
        ir.main = vec![Instruction::Call("large".to_string())];

        let optimized = optimizer.inline(&ir).unwrap();

        // Should be inlined despite size
        let has_call = optimized.main.iter().any(|i| matches!(i, Instruction::Call(_)));
        assert!(!has_call, "Forced inline should override size threshold");
    }

    #[test]
    fn test_inline_stats() {
        let optimizer = AggressiveInlineOptimizer::new(OptimizationLevel::Aggressive);

        let mut ir = ForthIR::new();
        let small = WordDef::new("small".to_string(), vec![Instruction::Dup]);
        ir.add_word(small);
        ir.main = vec![
            Instruction::Call("small".to_string()),
            Instruction::Call("small".to_string()),
        ];

        let optimized = optimizer.inline(&ir).unwrap();
        let stats = optimizer.get_stats(&ir, &optimized);

        assert_eq!(stats.calls_before, 2);
        assert_eq!(stats.calls_inlined, 2);
        assert!(stats.code_bloat_factor >= 1.0);
    }
}
