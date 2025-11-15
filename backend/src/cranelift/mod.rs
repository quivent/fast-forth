//! Cranelift Backend for Fast Forth
//!
//! This module provides fast compilation through Cranelift code generator.
//! Trade-off: 100x faster compilation (50ms vs 2-5min) with slightly lower
//! runtime performance (70-85% of C vs LLVM's 85-110% of C).
//!
//! **Use Case**: Development builds (-O0, -O1) for fast iteration
//! **Not for**: Production releases (use LLVM with -O2, -O3)

mod compiler;
mod translator;

pub use compiler::{CraneliftBackend, CraneliftCompiler};
pub use translator::SSATranslator;

use crate::error::{BackendError, Result};
use fastforth_frontend::ssa::{SSAFunction, SSAInstruction, Register, BlockId};

/// Compilation settings for Cranelift
#[derive(Debug, Clone, Copy)]
pub struct CraneliftSettings {
    /// Optimization level (0 = no opts, 1 = basic opts)
    pub opt_level: u8,
    /// Enable debug info generation
    pub debug_info: bool,
    /// Target triple (defaults to host)
    pub target_triple: Option<&'static str>,
}

impl Default for CraneliftSettings {
    fn default() -> Self {
        Self {
            opt_level: 0,
            debug_info: false,
            target_triple: None,
        }
    }
}

impl CraneliftSettings {
    /// Create settings for development builds (fast compilation)
    pub fn development() -> Self {
        Self {
            opt_level: 0,
            debug_info: true,
            target_triple: None,
        }
    }

    /// Create settings for optimized development builds
    pub fn optimized_dev() -> Self {
        Self {
            opt_level: 1,
            debug_info: true,
            target_triple: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = CraneliftSettings::default();
        assert_eq!(settings.opt_level, 0);
        assert!(!settings.debug_info);
    }

    #[test]
    fn test_development_settings() {
        let settings = CraneliftSettings::development();
        assert_eq!(settings.opt_level, 0);
        assert!(settings.debug_info);
    }
}
