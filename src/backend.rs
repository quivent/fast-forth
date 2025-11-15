//! Backend selection and management
//!
//! This module provides the logic for selecting between different compilation backends
//! based on optimization level:
//! - Cranelift: Fast compilation (50ms), good runtime (70-85% of C) - Default for -O0/-O1
//! - LLVM: Slow compilation (2-5min), excellent runtime (85-110% of C) - Default for -O2/-O3

use crate::error::{CompileError, Result};
use fastforth_frontend::ssa::SSAFunction;
use fastforth_optimizer::{ForthIR, OptimizationLevel};

#[cfg(feature = "cranelift")]
use backend::cranelift::{CraneliftCompiler, CraneliftSettings};

/// Backend type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Cranelift backend - Fast compilation, good runtime
    Cranelift,
    /// LLVM backend - Slow compilation, excellent runtime
    LLVM,
}

/// Backend selection strategy
pub struct BackendSelector;

impl BackendSelector {
    /// Select backend based on optimization level
    ///
    /// - O0, O1, O2: Cranelift (fast compilation, good performance)
    /// - O3: LLVM (slow compilation, maximum performance)
    pub fn select_backend(opt_level: OptimizationLevel) -> BackendType {
        match opt_level {
            OptimizationLevel::None | OptimizationLevel::Basic | OptimizationLevel::Standard => {
                // For development and standard builds, prioritize fast compilation
                #[cfg(feature = "cranelift")]
                return BackendType::Cranelift;

                #[cfg(not(feature = "cranelift"))]
                BackendType::LLVM
            }
            OptimizationLevel::Aggressive => {
                // For maximum optimization, use LLVM
                BackendType::LLVM
            }
        }
    }

    /// Check if backend is available
    pub fn is_available(backend: BackendType) -> bool {
        match backend {
            BackendType::Cranelift => cfg!(feature = "cranelift"),
            BackendType::LLVM => cfg!(feature = "llvm"),
        }
    }

    /// Get backend name
    pub fn backend_name(backend: BackendType) -> &'static str {
        match backend {
            BackendType::Cranelift => "Cranelift",
            BackendType::LLVM => "LLVM",
        }
    }

    /// Get expected compile time range
    pub fn expected_compile_time(backend: BackendType) -> &'static str {
        match backend {
            BackendType::Cranelift => "10-50ms",
            BackendType::LLVM => "2-5 minutes",
        }
    }

    /// Get expected runtime performance
    pub fn expected_runtime_performance(backend: BackendType) -> &'static str {
        match backend {
            BackendType::Cranelift => "70-85% of C",
            BackendType::LLVM => "85-110% of C",
        }
    }
}

/// Unified backend interface
pub struct Backend {
    backend_type: BackendType,
    #[cfg(feature = "cranelift")]
    cranelift: Option<CraneliftCompiler>,
}

impl Backend {
    /// Create a new backend with automatic selection based on optimization level
    pub fn new(opt_level: OptimizationLevel) -> Result<Self> {
        let backend_type = BackendSelector::select_backend(opt_level);

        if !BackendSelector::is_available(backend_type) {
            return Err(CompileError::BackendError(format!(
                "{} backend not available (feature not enabled)",
                BackendSelector::backend_name(backend_type)
            )));
        }

        Self::with_backend(backend_type, opt_level)
    }

    /// Create a backend with explicit backend selection
    pub fn with_backend(backend_type: BackendType, opt_level: OptimizationLevel) -> Result<Self> {
        match backend_type {
            #[cfg(feature = "cranelift")]
            BackendType::Cranelift => {
                let settings = match opt_level {
                    OptimizationLevel::None => CraneliftSettings::development(),
                    OptimizationLevel::Basic => CraneliftSettings::optimized_dev(),
                    OptimizationLevel::Standard => CraneliftSettings::maximum(),
                    OptimizationLevel::Aggressive => {
                        return Err(CompileError::BackendError(
                            "Cranelift maximum is -O2 (speed_and_size). Use LLVM for -O3.".to_string()
                        ))
                    }
                };

                let compiler = CraneliftCompiler::with_settings(settings)
                    .map_err(|e| CompileError::BackendError(format!("Cranelift init failed: {}", e)))?;

                Ok(Self {
                    backend_type,
                    cranelift: Some(compiler),
                })
            }

            #[cfg(not(feature = "cranelift"))]
            BackendType::Cranelift => {
                Err(CompileError::BackendError(
                    "Cranelift backend not available (compile with --features cranelift)".to_string()
                ))
            }

            BackendType::LLVM => {
                // LLVM backend is the fallback - return stub for now
                Ok(Self {
                    backend_type,
                    #[cfg(feature = "cranelift")]
                    cranelift: None,
                })
            }
        }
    }

    /// Get the backend type being used
    pub fn backend_type(&self) -> BackendType {
        self.backend_type
    }

    /// Compile an SSA function to native code
    pub fn compile_function(&mut self, ssa_func: &SSAFunction, name: &str) -> Result<*const u8> {
        match self.backend_type {
            #[cfg(feature = "cranelift")]
            BackendType::Cranelift => {
                if let Some(ref mut compiler) = self.cranelift {
                    compiler.compile(ssa_func, name)
                        .map_err(|e| CompileError::BackendError(format!("Cranelift compilation failed: {}", e)))
                } else {
                    Err(CompileError::BackendError("Cranelift not initialized".to_string()))
                }
            }

            #[cfg(not(feature = "cranelift"))]
            BackendType::Cranelift => {
                Err(CompileError::BackendError("Cranelift not available".to_string()))
            }

            BackendType::LLVM => {
                // LLVM compilation logic would go here
                Err(CompileError::BackendError("LLVM compilation not yet integrated with unified backend".to_string()))
            }
        }
    }

    /// Get information about the backend
    pub fn info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: self.backend_type,
            name: BackendSelector::backend_name(self.backend_type),
            compile_time: BackendSelector::expected_compile_time(self.backend_type),
            runtime_performance: BackendSelector::expected_runtime_performance(self.backend_type),
        }
    }
}

/// Information about a backend
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub backend_type: BackendType,
    pub name: &'static str,
    pub compile_time: &'static str,
    pub runtime_performance: &'static str,
}

impl std::fmt::Display for BackendInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (compile: {}, runtime: {})",
            self.name, self.compile_time, self.runtime_performance
        )
    }
}

// Legacy stubs for compatibility
pub struct LLVMBackend;

impl LLVMBackend {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_llvm_ir(&self, _ir: &ForthIR) -> Result<String> {
        Err(CompileError::CodeGenError(
            "LLVM backend not yet implemented".to_string(),
        ))
    }

    pub fn compile_to_object(&self, _ir: &ForthIR, _output_path: &str) -> Result<()> {
        Err(CompileError::CodeGenError(
            "Object compilation not yet implemented".to_string(),
        ))
    }
}

impl Default for LLVMBackend {
    fn default() -> Self {
        Self::new()
    }
}

pub struct JITExecutor;

impl JITExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, _ir: &ForthIR) -> Result<i64> {
        Err(CompileError::CodeGenError(
            "JIT execution not yet implemented".to_string(),
        ))
    }
}

impl Default for JITExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_selection() {
        // O0/O1/O2 should select Cranelift (if available)
        let backend = BackendSelector::select_backend(OptimizationLevel::None);
        #[cfg(feature = "cranelift")]
        assert_eq!(backend, BackendType::Cranelift);
        #[cfg(not(feature = "cranelift"))]
        assert_eq!(backend, BackendType::LLVM);

        let backend = BackendSelector::select_backend(OptimizationLevel::Standard);
        #[cfg(feature = "cranelift")]
        assert_eq!(backend, BackendType::Cranelift);
        #[cfg(not(feature = "cranelift"))]
        assert_eq!(backend, BackendType::LLVM);

        // O3 should always select LLVM
        let backend = BackendSelector::select_backend(OptimizationLevel::Aggressive);
        assert_eq!(backend, BackendType::LLVM);
    }

    #[test]
    fn test_backend_availability() {
        #[cfg(feature = "cranelift")]
        assert!(BackendSelector::is_available(BackendType::Cranelift));
    }
}
