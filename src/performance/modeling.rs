//! Performance modeling and prediction
//!
//! Provides analytical models to predict:
//! - Execution speed (vs C baseline)
//! - Compile time
//! - Binary size
//! - Memory usage

use crate::error::{CompileError, Result};
use fastforth_optimizer::{ForthIR, Instruction};
use serde::{Deserialize, Serialize};

/// Performance target specification
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PerformanceTarget {
    /// Target execution speed relative to C (0.0-1.0+)
    /// 0.9 means 90% of C performance
    pub speed_ratio: f64,

    /// Maximum compile time in milliseconds
    pub max_compile_time_ms: Option<u64>,

    /// Maximum binary size in bytes
    pub max_binary_size: Option<usize>,

    /// Maximum memory usage in bytes
    pub max_memory_usage: Option<usize>,
}

impl PerformanceTarget {
    /// Create a new performance target with just speed ratio
    pub fn new(speed_ratio: f64) -> Self {
        Self {
            speed_ratio,
            max_compile_time_ms: None,
            max_binary_size: None,
            max_memory_usage: None,
        }
    }

    /// Set maximum compile time
    pub fn with_compile_time(mut self, ms: u64) -> Self {
        self.max_compile_time_ms = Some(ms);
        self
    }

    /// Set maximum binary size
    pub fn with_binary_size(mut self, bytes: usize) -> Self {
        self.max_binary_size = Some(bytes);
        self
    }

    /// Set maximum memory usage
    pub fn with_memory_usage(mut self, bytes: usize) -> Self {
        self.max_memory_usage = Some(bytes);
        self
    }
}

/// Performance prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    /// Predicted execution speed relative to C
    pub speed_ratio: f64,

    /// Predicted compile time in milliseconds
    pub compile_time_ms: u64,

    /// Predicted binary size in bytes
    pub binary_size: usize,

    /// Predicted memory usage in bytes
    pub memory_usage: usize,

    /// Predicted branch prediction hit rate
    pub branch_prediction_rate: f64,

    /// Detailed breakdown by operation type
    pub breakdown: OperationBreakdown,
}

impl PerformancePrediction {
    /// Check if prediction meets the target
    pub fn meets_target(&self, target: &PerformanceTarget) -> bool {
        // Check speed ratio
        if self.speed_ratio < target.speed_ratio {
            return false;
        }

        // Check compile time if specified
        if let Some(max_time) = target.max_compile_time_ms {
            if self.compile_time_ms > max_time {
                return false;
            }
        }

        // Check binary size if specified
        if let Some(max_size) = target.max_binary_size {
            if self.binary_size > max_size {
                return false;
            }
        }

        // Check memory usage if specified
        if let Some(max_mem) = target.max_memory_usage {
            if self.memory_usage > max_mem {
                return false;
            }
        }

        true
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Performance: {:.2}x C | Compile: {}ms | Binary: {} bytes | Memory: {} bytes",
            self.speed_ratio,
            self.compile_time_ms,
            self.binary_size,
            self.memory_usage
        )
    }
}

/// Breakdown of operations by type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperationBreakdown {
    pub arithmetic_ops: usize,
    pub memory_ops: usize,
    pub branch_ops: usize,
    pub call_ops: usize,
    pub stack_ops: usize,
    pub total_ops: usize,
}

/// Performance model for predicting execution characteristics
pub struct PerformanceModel {
    /// Operation costs in CPU cycles
    operation_costs: OperationCosts,
}

impl PerformanceModel {
    /// Create a new performance model with default costs
    pub fn new() -> Self {
        Self {
            operation_costs: OperationCosts::default(),
        }
    }

    /// Predict performance for the given IR
    pub fn predict(&self, ir: &ForthIR) -> Result<PerformancePrediction> {
        let breakdown = self.analyze_operations(ir);
        let total_cycles = self.estimate_cycles(&breakdown);

        // Model execution speed
        // Assume baseline C implementation takes 1 cycle per operation
        // Fast Forth with optimizations should be competitive
        let c_baseline_cycles = breakdown.total_ops as f64;
        let speed_ratio = c_baseline_cycles / total_cycles.max(1.0);

        // Model compile time (roughly 0.1ms per instruction with optimization)
        let compile_time_ms = (breakdown.total_ops as f64 * 0.1) as u64;

        // Model binary size (roughly 8 bytes per instruction on x86-64)
        let binary_size = breakdown.total_ops * 8;

        // Model memory usage (stack + heap)
        let stack_depth = self.estimate_stack_depth(ir);
        let memory_usage = stack_depth * 8 + 1024; // Stack + overhead

        // Model branch prediction (simple heuristic)
        let branch_prediction_rate = if breakdown.branch_ops > 0 {
            0.85 // Typical modern CPU branch prediction rate
        } else {
            1.0 // No branches, 100% predictable
        };

        Ok(PerformancePrediction {
            speed_ratio,
            compile_time_ms,
            binary_size,
            memory_usage,
            branch_prediction_rate,
            breakdown,
        })
    }

    /// Analyze operations in the IR
    fn analyze_operations(&self, ir: &ForthIR) -> OperationBreakdown {
        let mut breakdown = OperationBreakdown::default();

        for word in ir.words.values() {
            for inst in &word.instructions {
                breakdown.total_ops += 1;

                match inst {
                    // Arithmetic operations
                    Instruction::Add | Instruction::Sub |
                    Instruction::Mul | Instruction::Div |
                    Instruction::Mod | Instruction::Neg |
                    Instruction::Abs => {
                        breakdown.arithmetic_ops += 1;
                    }

                    // Memory operations
                    Instruction::Load | Instruction::Store => {
                        breakdown.memory_ops += 1;
                    }

                    // Branch operations
                    Instruction::Branch(_) |
                    Instruction::BranchIfNot(_) => {
                        breakdown.branch_ops += 1;
                    }

                    // Call operations
                    Instruction::Call(_) | Instruction::Return => {
                        breakdown.call_ops += 1;
                    }

                    // Stack operations
                    Instruction::Dup | Instruction::Drop |
                    Instruction::Swap | Instruction::Over |
                    Instruction::Rot => {
                        breakdown.stack_ops += 1;
                    }

                    _ => {}
                }
            }
        }

        breakdown
    }

    /// Estimate total CPU cycles for operations
    fn estimate_cycles(&self, breakdown: &OperationBreakdown) -> f64 {
        let mut total_cycles = 0.0;

        total_cycles += breakdown.arithmetic_ops as f64 * self.operation_costs.arithmetic;
        total_cycles += breakdown.memory_ops as f64 * self.operation_costs.memory;
        total_cycles += breakdown.branch_ops as f64 * self.operation_costs.branch;
        total_cycles += breakdown.call_ops as f64 * self.operation_costs.call;
        total_cycles += breakdown.stack_ops as f64 * self.operation_costs.stack;

        total_cycles
    }

    /// Estimate maximum stack depth
    fn estimate_stack_depth(&self, ir: &ForthIR) -> usize {
        // Simple heuristic: count maximum stack operations in any word
        let mut max_depth = 0;

        for word in ir.words.values() {
            let mut depth: usize = 0;
            for inst in &word.instructions {
                match inst {
                    Instruction::Literal(_) | Instruction::FloatLiteral(_) => depth += 1,
                    Instruction::Dup => depth += 1,
                    Instruction::Drop => depth = depth.saturating_sub(1),
                    Instruction::Add | Instruction::Sub |
                    Instruction::Mul | Instruction::Div => depth = depth.saturating_sub(1),
                    _ => {}
                }
                max_depth = max_depth.max(depth);
            }
        }

        max_depth.max(16) // Minimum stack depth
    }
}

impl Default for PerformanceModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Operation costs in CPU cycles
#[derive(Debug, Clone)]
struct OperationCosts {
    arithmetic: f64,
    memory: f64,
    branch: f64,
    call: f64,
    stack: f64,
}

impl Default for OperationCosts {
    fn default() -> Self {
        Self {
            arithmetic: 1.0,   // Fast arithmetic operations
            memory: 3.0,       // Memory access has latency
            branch: 2.0,       // Branch prediction overhead
            call: 5.0,         // Function call overhead
            stack: 0.5,        // Stack operations are optimized to registers
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_target_creation() {
        let target = PerformanceTarget::new(0.9);
        assert_eq!(target.speed_ratio, 0.9);
        assert_eq!(target.max_compile_time_ms, None);
    }

    #[test]
    fn test_performance_target_builder() {
        let target = PerformanceTarget::new(0.95)
            .with_compile_time(100)
            .with_binary_size(1024)
            .with_memory_usage(2048);

        assert_eq!(target.speed_ratio, 0.95);
        assert_eq!(target.max_compile_time_ms, Some(100));
        assert_eq!(target.max_binary_size, Some(1024));
        assert_eq!(target.max_memory_usage, Some(2048));
    }

    #[test]
    fn test_performance_model_creation() {
        let model = PerformanceModel::new();
        assert_eq!(model.operation_costs.arithmetic, 1.0);
    }
}
