//! Stack Caching Optimizer
//!
//! Keeps the top N stack items in registers for dramatic performance improvements.
//! This is the most impactful optimization for stack-based code, typically
//! achieving 2-3x speedup on stack-heavy operations.
//!
//! # Algorithm
//!
//! 1. Track stack depth at each instruction
//! 2. Allocate registers for top N items (typically 3: TOS, NOS, 3OS)
//! 3. Transform instructions to use cached registers
//! 4. Insert flush/reload instructions at call boundaries
//!
//! # Register Allocation
//!
//! ```text
//! Stack:  [... | 3OS | NOS | TOS]
//! Regs:        r2    r1    r0
//! ```
//!
//! # Example Transformation
//!
//! Before:
//! ```forth
//! 1 2 + dup *
//! ```
//!
//! After (with stack caching):
//! ```assembly
