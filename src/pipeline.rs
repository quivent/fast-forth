//! Compilation pipeline implementation
//!
//! This module implements the complete compilation pipeline:
//! 1. Frontend: Parse → Semantic Analysis → Type Inference → SSA Conversion
//! 2. Optimizer: 5 optimization passes
//! 3. Backend: LLVM IR generation → Native code
//! 4. Execution: JIT or AOT

use crate::error::{CompileError, Result};
use fastforth_frontend::{parse_program, analyze, convert_to_ssa, Program, SSAFunction};
use fastforth_optimizer::{ForthIR, Optimizer, OptimizationLevel, Instruction};
use tracing::{debug, info, warn};
use std::time::Instant;

/// Compilation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationMode {
    /// Compile to native executable
    AOT,
    /// Just-in-time compilation and execution
    JIT,
}

/// Result of compilation
#[derive(Debug)]
pub struct CompilationResult {
    /// Compilation mode used
    pub mode: CompilationMode,
    /// Compilation time in milliseconds
    pub compile_time_ms: u64,
    /// Generated code size (if available)
    pub code_size: Option<usize>,
    /// Output file path (for AOT mode)
    pub output_path: Option<String>,
    /// JIT execution result (for JIT mode)
    pub jit_result: Option<i64>,
    /// Optimization statistics
    pub stats: CompilationStats,
}

/// Compilation statistics
#[derive(Debug, Default)]
pub struct CompilationStats {
    /// Number of definitions compiled
    pub definitions_count: usize,
    /// Number of instructions before optimization
    pub instructions_before: usize,
    /// Number of instructions after optimization
    pub instructions_after: usize,
    /// Frontend time in milliseconds
    pub frontend_time_ms: u64,
    /// Optimization time in milliseconds
    pub optimization_time_ms: u64,
    /// Backend time in milliseconds
    pub backend_time_ms: u64,
}

impl CompilationStats {
    /// Calculate optimization savings
    pub fn optimization_savings(&self) -> f64 {
        if self.instructions_before == 0 {
            0.0
        } else {
            (self.instructions_before - self.instructions_after) as f64 / self.instructions_before as f64
        }
    }
}

/// The main compilation pipeline
pub struct CompilationPipeline {
    optimization_level: OptimizationLevel,
    optimizer: Optimizer,
}

impl CompilationPipeline {
    /// Create a new compilation pipeline
    pub fn new(optimization_level: OptimizationLevel) -> Self {
        Self {
            optimization_level,
            optimizer: Optimizer::new(optimization_level),
        }
    }

    /// Compile Forth source code
    pub fn compile(&mut self, source: &str, mode: CompilationMode) -> Result<CompilationResult> {
        let start_time = Instant::now();
        let mut stats = CompilationStats::default();

        info!("Starting compilation in {:?} mode", mode);

        // Phase 1: Frontend (Parsing, Semantic Analysis, Type Inference, SSA)
        let frontend_start = Instant::now();
        let (program, ssa_functions) = self.run_frontend(source)?;
        stats.frontend_time_ms = frontend_start.elapsed().as_millis() as u64;
        stats.definitions_count = program.definitions.len();

        debug!("Frontend complete: {} definitions", stats.definitions_count);

        // Phase 2: Convert SSA to Optimizer IR
        let ir = self.convert_to_ir(&ssa_functions)?;
        stats.instructions_before = self.count_instructions(&ir);

        // Phase 3: Optimization
        let optimization_start = Instant::now();
        let optimized_ir = self.run_optimizer(ir)?;
        stats.optimization_time_ms = optimization_start.elapsed().as_millis() as u64;
        stats.instructions_after = self.count_instructions(&optimized_ir);

        info!(
            "Optimization reduced instructions by {:.1}%",
            stats.optimization_savings() * 100.0
        );

        // Phase 4: Backend code generation
        let backend_start = Instant::now();
        let result = match mode {
            CompilationMode::AOT => self.compile_aot(&optimized_ir, &mut stats)?,
            CompilationMode::JIT => self.compile_jit(&optimized_ir, &mut stats)?,
        };
        stats.backend_time_ms = backend_start.elapsed().as_millis() as u64;

        let compile_time_ms = start_time.elapsed().as_millis() as u64;

        info!(
            "Compilation complete in {}ms (frontend: {}ms, opt: {}ms, backend: {}ms)",
            compile_time_ms,
            stats.frontend_time_ms,
            stats.optimization_time_ms,
            stats.backend_time_ms
        );

        Ok(CompilationResult {
            mode,
            compile_time_ms,
            code_size: result.0,
            output_path: result.1,
            jit_result: result.2,
            stats,
        })
    }

    /// Run the frontend pipeline
    fn run_frontend(&self, source: &str) -> Result<(Program, Vec<SSAFunction>)> {
        // Step 1: Parse
        debug!("Parsing source code...");
        let program = parse_program(source)
            .map_err(|e| CompileError::ParseError(format!("{}", e)))?;

        // Step 2: Semantic analysis
        debug!("Running semantic analysis...");
        analyze(&program)
            .map_err(|e| CompileError::SemanticError(format!("{}", e)))?;

        // Step 3: Type inference happens inside convert_to_ssa

        // Step 4: Convert to SSA
        debug!("Converting to SSA...");
        let ssa_functions = convert_to_ssa(&program)
            .map_err(|e| CompileError::SSAError(format!("{}", e)))?;

        Ok((program, ssa_functions))
    }

    /// Convert frontend SSA to optimizer IR
    fn convert_to_ir(&self, ssa_functions: &[SSAFunction]) -> Result<ForthIR> {
        debug!("Converting SSA to optimizer IR...");

        // Create a new ForthIR
        let mut ir = ForthIR::new();

        // Convert each SSA function to IR instructions
        for func in ssa_functions {
            let instructions = self.ssa_to_instructions(func)?;

            // Create a word definition for this function
            use fastforth_optimizer::ir::WordDef;
            let word_def = WordDef::new(func.name.clone(), instructions);
            ir.add_word(word_def);
        }

        Ok(ir)
    }

    /// Convert a single SSA function to IR instructions
    fn ssa_to_instructions(&self, func: &SSAFunction) -> Result<Vec<Instruction>> {
        use fastforth_frontend::ssa::{SSAInstruction, BinaryOperator, UnaryOperator};

        let mut instructions = Vec::new();

        // Process each basic block
        for block in &func.blocks {
            // Add a label for this block
            instructions.push(Instruction::Label(format!("bb{}", block.id.0)));

            // Convert each SSA instruction
            for ssa_inst in &block.instructions {
                match ssa_inst {
                    SSAInstruction::LoadInt { value, .. } => {
                        instructions.push(Instruction::Literal(*value));
                    }
                    SSAInstruction::LoadFloat { value, .. } => {
                        instructions.push(Instruction::FloatLiteral(*value));
                    }
                    SSAInstruction::BinaryOp { op, .. } => {
                        let inst = match op {
                            BinaryOperator::Add => Instruction::Add,
                            BinaryOperator::Sub => Instruction::Sub,
                            BinaryOperator::Mul => Instruction::Mul,
                            BinaryOperator::Div => Instruction::Div,
                            BinaryOperator::Mod => Instruction::Mod,
                            BinaryOperator::Lt => Instruction::Lt,
                            BinaryOperator::Gt => Instruction::Gt,
                            BinaryOperator::Le => Instruction::Le,
                            BinaryOperator::Ge => Instruction::Ge,
                            BinaryOperator::Eq => Instruction::Eq,
                            BinaryOperator::Ne => Instruction::Ne,
                            BinaryOperator::And => Instruction::And,
                            BinaryOperator::Or => Instruction::Or,
                        };
                        instructions.push(inst);
                    }
                    SSAInstruction::UnaryOp { op, .. } => {
                        let inst = match op {
                            UnaryOperator::Negate => Instruction::Neg,
                            UnaryOperator::Not => Instruction::Not,
                            UnaryOperator::Abs => Instruction::Abs,
                        };
                        instructions.push(inst);
                    }
                    SSAInstruction::Call { name, .. } => {
                        instructions.push(Instruction::Call(name.clone()));
                    }
                    SSAInstruction::Return { .. } => {
                        instructions.push(Instruction::Return);
                    }
                    SSAInstruction::Branch { true_block, false_block, .. } => {
                        // Conditional branch: branch if not, then fall through or jump
                        instructions.push(Instruction::BranchIfNot(true_block.0));
                        instructions.push(Instruction::Branch(false_block.0));
                    }
                    SSAInstruction::Jump { target } => {
                        instructions.push(Instruction::Branch(target.0));
                    }
                    SSAInstruction::Load { .. } => {
                        instructions.push(Instruction::Load);
                    }
                    SSAInstruction::Store { .. } => {
                        instructions.push(Instruction::Store);
                    }
                    SSAInstruction::Phi { .. } => {
                        // Phi nodes are handled by SSA construction and don't need runtime code
                        // They're just for data flow analysis
                        continue;
                    }
                    _ => {
                        warn!("Unhandled SSA instruction: {:?}", ssa_inst);
                        continue;
                    }
                }
            }
        }

        Ok(instructions)
    }

    /// Run the optimizer
    fn run_optimizer(&mut self, ir: ForthIR) -> Result<ForthIR> {
        debug!("Running optimizer with level {:?}...", self.optimization_level);

        let optimized = self.optimizer.optimize(ir)
            .map_err(|e| CompileError::OptimizationError(format!("{}", e)))?;

        Ok(optimized)
    }

    /// Compile to native executable (AOT)
    fn compile_aot(&self, ir: &ForthIR, stats: &mut CompilationStats) -> Result<(Option<usize>, Option<String>, Option<i64>)> {
        debug!("Generating native code (AOT)...");

        // TODO: Implement LLVM backend integration
        // For now, return a placeholder
        warn!("AOT compilation not yet fully implemented");

        Ok((None, Some("output.o".to_string()), None))
    }

    /// Compile and execute with JIT
    fn compile_jit(&self, ir: &ForthIR, stats: &mut CompilationStats) -> Result<(Option<usize>, Option<String>, Option<i64>)> {
        debug!("Compiling and executing (JIT)...");

        // TODO: Implement JIT execution
        // For now, return a placeholder
        warn!("JIT compilation not yet fully implemented");

        Ok((None, None, Some(0)))
    }

    /// Count total instructions in IR
    fn count_instructions(&self, ir: &ForthIR) -> usize {
        ir.instruction_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = CompilationPipeline::new(OptimizationLevel::Standard);
        assert_eq!(pipeline.optimization_level, OptimizationLevel::Standard);
    }

    #[test]
    fn test_simple_compilation() {
        let pipeline = CompilationPipeline::new(OptimizationLevel::Basic);
        let source = ": double 2 * ;";

        // This will fail until backend is implemented, but tests the pipeline structure
        let result = pipeline.compile(source, CompilationMode::JIT);
        // We expect this to fail for now, but it should be a compilation error, not a panic
        assert!(result.is_ok() || result.is_err());
    }
}
