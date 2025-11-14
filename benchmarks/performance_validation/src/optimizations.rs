/// Optimization levels for Fast Forth
///
/// Different optimization strategies to measure and validate

use serde::{Deserialize, Serialize};
use std::fmt;

/// Optimization levels available in Fast Forth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// No optimizations (baseline)
    None,
    /// Aggressive inlining
    Inlining,
    /// Profile-guided optimization with superinstructions
    PGO,
    /// All optimizations enabled
    Aggressive,
}

impl OptimizationLevel {
    /// Get all optimization levels in order
    pub fn all() -> Vec<Self> {
        vec![
            Self::None,
            Self::Inlining,
            Self::PGO,
            Self::Aggressive,
        ]
    }

    /// Get compiler flags for this optimization level
    pub fn compiler_flags(&self) -> Vec<String> {
        match self {
            Self::None => vec![
                "-O0".to_string(),
            ],
            Self::Inlining => vec![
                "-O1".to_string(),
                "--inline-aggressive".to_string(),
            ],
            Self::PGO => vec![
                "-O2".to_string(),
                "--inline-aggressive".to_string(),
                "--pgo".to_string(),
                "--superinstructions".to_string(),
            ],
            Self::Aggressive => vec![
                "-O3".to_string(),
                "--inline-aggressive".to_string(),
                "--pgo".to_string(),
                "--superinstructions".to_string(),
                "--whole-program".to_string(),
                "--lto".to_string(),
            ],
        }
    }

    /// Get expected speedup factor over baseline
    pub fn expected_speedup(&self) -> f64 {
        match self {
            Self::None => 1.0,
            Self::Inlining => 1.15,
            Self::PGO => 1.40,
            Self::Aggressive => 1.60,
        }
    }

    /// Get description
    pub fn description(&self) -> &str {
        match self {
            Self::None => "No optimizations (baseline)",
            Self::Inlining => "Aggressive inlining",
            Self::PGO => "Profile-guided optimization with superinstructions",
            Self::Aggressive => "All optimizations enabled",
        }
    }
}

impl fmt::Display for OptimizationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "Baseline"),
            Self::Inlining => write!(f, "Inlining"),
            Self::PGO => write!(f, "PGO"),
            Self::Aggressive => write!(f, "Aggressive"),
        }
    }
}

/// Optimization strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStrategy {
    pub level: OptimizationLevel,
    pub enable_inlining: bool,
    pub inline_threshold: usize,
    pub enable_pgo: bool,
    pub enable_superinstructions: bool,
    pub enable_whole_program: bool,
    pub enable_lto: bool,
}

impl OptimizationStrategy {
    /// Create strategy from optimization level
    pub fn from_level(level: OptimizationLevel) -> Self {
        match level {
            OptimizationLevel::None => Self {
                level,
                enable_inlining: false,
                inline_threshold: 0,
                enable_pgo: false,
                enable_superinstructions: false,
                enable_whole_program: false,
                enable_lto: false,
            },
            OptimizationLevel::Inlining => Self {
                level,
                enable_inlining: true,
                inline_threshold: 100,
                enable_pgo: false,
                enable_superinstructions: false,
                enable_whole_program: false,
                enable_lto: false,
            },
            OptimizationLevel::PGO => Self {
                level,
                enable_inlining: true,
                inline_threshold: 100,
                enable_pgo: true,
                enable_superinstructions: true,
                enable_whole_program: false,
                enable_lto: false,
            },
            OptimizationLevel::Aggressive => Self {
                level,
                enable_inlining: true,
                inline_threshold: 200,
                enable_pgo: true,
                enable_superinstructions: true,
                enable_whole_program: true,
                enable_lto: true,
            },
        }
    }

    /// Get enabled optimizations as list
    pub fn enabled_optimizations(&self) -> Vec<String> {
        let mut opts = Vec::new();

        if self.enable_inlining {
            opts.push(format!("Inlining (threshold={})", self.inline_threshold));
        }
        if self.enable_pgo {
            opts.push("Profile-Guided Optimization".to_string());
        }
        if self.enable_superinstructions {
            opts.push("Superinstructions".to_string());
        }
        if self.enable_whole_program {
            opts.push("Whole-Program Optimization".to_string());
        }
        if self.enable_lto {
            opts.push("Link-Time Optimization".to_string());
        }

        opts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimization_levels() {
        assert_eq!(OptimizationLevel::None.expected_speedup(), 1.0);
        assert!(OptimizationLevel::Aggressive.expected_speedup() > 1.5);
    }

    #[test]
    fn test_optimization_strategy() {
        let strategy = OptimizationStrategy::from_level(OptimizationLevel::Aggressive);
        assert!(strategy.enable_inlining);
        assert!(strategy.enable_pgo);
        assert!(strategy.enable_lto);
    }
}
