//! Stack Caching - Register Allocation for Stack Values
//!
//! Keep the top N stack elements in registers to minimize memory operations.
//! This is one of the key optimizations for Forth performance.

use crate::error::{BackendError, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::types::IntType;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue};
use inkwell::AddressSpace;
use std::collections::VecDeque;

/// Stack cache keeps top N stack items in registers
pub struct StackCache<'ctx> {
    context: &'ctx Context,

    /// Number of stack elements to keep in registers
    cache_depth: usize,

    /// Cached stack values (top of stack is at back)
    cached_values: VecDeque<BasicValueEnum<'ctx>>,

    /// Stack pointer (for spilled values)
    stack_ptr: Option<PointerValue<'ctx>>,

    /// Cell type (i64)
    cell_type: IntType<'ctx>,
}

impl<'ctx> StackCache<'ctx> {
    /// Create a new stack cache
    pub fn new(context: &'ctx Context, cache_depth: usize) -> Self {
        Self {
            context,
            cache_depth,
            cached_values: VecDeque::with_capacity(cache_depth),
            stack_ptr: None,
            cell_type: context.i64_type(),
        }
    }

    /// Initialize stack pointer
    pub fn init_stack_ptr(&mut self, builder: &Builder<'ctx>) -> Result<()> {
        // Allocate stack space
        let stack_size = self.context.i64_type().const_int(256, false);
        let stack_array = builder.build_array_alloca(self.cell_type, stack_size, "data_stack")
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        self.stack_ptr = Some(stack_array);
        Ok(())
    }

    /// Push a value onto the cached stack
    pub fn push(&mut self, builder: &Builder<'ctx>, value: BasicValueEnum<'ctx>) -> Result<()> {
        if self.cached_values.len() >= self.cache_depth {
            // Cache is full, spill oldest value to memory
            self.spill_to_memory(builder)?;
        }

        self.cached_values.push_back(value);
        Ok(())
    }

    /// Pop a value from the cached stack
    pub fn pop(&mut self, builder: &Builder<'ctx>) -> Result<BasicValueEnum<'ctx>> {
        if let Some(value) = self.cached_values.pop_back() {
            Ok(value)
        } else {
            // Cache is empty, load from memory
            self.load_from_memory(builder)
        }
    }

    /// Peek at top of stack without removing
    pub fn peek(&self, offset: usize) -> Result<BasicValueEnum<'ctx>> {
        let idx = self.cached_values.len()
            .checked_sub(1 + offset)
            .ok_or_else(|| BackendError::RegisterAllocationFailed("Stack underflow".to_string()))?;

        self.cached_values
            .get(idx)
            .copied()
            .ok_or_else(|| BackendError::RegisterAllocationFailed("Value not in cache".to_string()))
    }

    /// Duplicate top of stack
    pub fn dup(&mut self, builder: &Builder<'ctx>) -> Result<()> {
        let top = self.peek(0)?;
        self.push(builder, top)
    }

    /// Drop top of stack
    pub fn drop(&mut self) -> Result<()> {
        self.cached_values.pop_back()
            .ok_or_else(|| BackendError::RegisterAllocationFailed("Stack underflow".to_string()))?;
        Ok(())
    }

    /// Swap top two stack elements
    pub fn swap(&mut self) -> Result<()> {
        if self.cached_values.len() < 2 {
            return Err(BackendError::RegisterAllocationFailed("Need at least 2 elements for swap".to_string()));
        }

        let len = self.cached_values.len();
        self.cached_values.swap(len - 1, len - 2);
        Ok(())
    }

    /// Over - copy second element to top
    pub fn over(&mut self, builder: &Builder<'ctx>) -> Result<()> {
        let second = self.peek(1)?;
        self.push(builder, second)
    }

    /// Rotate top three elements
    pub fn rot(&mut self) -> Result<()> {
        if self.cached_values.len() < 3 {
            return Err(BackendError::RegisterAllocationFailed("Need at least 3 elements for rot".to_string()));
        }

        let len = self.cached_values.len();
        // ( a b c -- b c a )
        let temp = self.cached_values[len - 3];
        self.cached_values[len - 3] = self.cached_values[len - 2];
        self.cached_values[len - 2] = self.cached_values[len - 1];
        self.cached_values[len - 1] = temp;
        Ok(())
    }

    /// Spill oldest cached value to memory
    fn spill_to_memory(&mut self, builder: &Builder<'ctx>) -> Result<()> {
        let stack_ptr = self.stack_ptr
            .ok_or_else(|| BackendError::RegisterAllocationFailed("Stack pointer not initialized".to_string()))?;

        let value = self.cached_values.pop_front()
            .ok_or_else(|| BackendError::RegisterAllocationFailed("No values to spill".to_string()))?;

        // Store to stack memory
        builder.build_store(stack_ptr, value)
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(())
    }

    /// Load value from memory into cache
    fn load_from_memory(&mut self, builder: &Builder<'ctx>) -> Result<BasicValueEnum<'ctx>> {
        let stack_ptr = self.stack_ptr
            .ok_or_else(|| BackendError::RegisterAllocationFailed("Stack pointer not initialized".to_string()))?;

        let loaded = builder.build_load(self.cell_type, stack_ptr, "stack_load")
            .map_err(|e| BackendError::CodeGenError(e.to_string()))?;

        Ok(loaded)
    }

    /// Get current cache depth
    pub fn depth(&self) -> usize {
        self.cached_values.len()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cached_values.clear();
    }

    /// Get all cached values (for debugging)
    pub fn get_cached_values(&self) -> &VecDeque<BasicValueEnum<'ctx>> {
        &self.cached_values
    }

    /// Flush all cached values to memory
    pub fn flush_to_memory(&mut self, builder: &Builder<'ctx>) -> Result<()> {
        while !self.cached_values.is_empty() {
            self.spill_to_memory(builder)?;
        }
        Ok(())
    }

    /// Optimize stack operations sequence
    pub fn optimize_sequence(&mut self, builder: &Builder<'ctx>, operations: &[StackOp]) -> Result<()> {
        for op in operations {
            match op {
                StackOp::Push(val) => self.push(builder, *val)?,
                StackOp::Pop => { self.cached_values.pop_back(); },
                StackOp::Dup => self.dup(builder)?,
                StackOp::Drop => self.drop()?,
                StackOp::Swap => self.swap()?,
                StackOp::Over => self.over(builder)?,
                StackOp::Rot => self.rot()?,
            }
        }
        Ok(())
    }
}

/// Stack operation types
#[derive(Debug, Clone, Copy)]
pub enum StackOp<'ctx> {
    Push(BasicValueEnum<'ctx>),
    Pop,
    Dup,
    Drop,
    Swap,
    Over,
    Rot,
}

/// Stack cache optimizer - analyzes and optimizes stack operations
pub struct StackCacheOptimizer<'ctx> {
    cache: StackCache<'ctx>,
}

impl<'ctx> StackCacheOptimizer<'ctx> {
    pub fn new(context: &'ctx Context, cache_depth: usize) -> Self {
        Self {
            cache: StackCache::new(context, cache_depth),
        }
    }

    /// Analyze stack depth at each program point
    pub fn analyze_stack_depths(&self, operations: &[StackOp]) -> Vec<usize> {
        let mut depths = Vec::with_capacity(operations.len());
        let mut current_depth: usize = 0;

        for op in operations {
            match op {
                StackOp::Push(_) => current_depth += 1,
                StackOp::Pop | StackOp::Drop => current_depth = current_depth.saturating_sub(1),
                StackOp::Dup | StackOp::Over => current_depth += 1,
                StackOp::Swap | StackOp::Rot => {}, // No depth change
            }
            depths.push(current_depth);
        }

        depths
    }

    /// Determine optimal cache depth for a sequence of operations
    pub fn optimal_cache_depth(&self, operations: &[StackOp]) -> usize {
        let depths = self.analyze_stack_depths(operations);
        let max_depth = depths.iter().max().copied().unwrap_or(0);

        // Use min of max depth and practical cache limit (8 registers)
        max_depth.min(8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_cache_creation() {
        let context = Context::create();
        let cache = StackCache::new(&context, 4);
        assert_eq!(cache.depth(), 0);
        assert_eq!(cache.cache_depth, 4);
    }

    #[test]
    fn test_stack_depth_analysis() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 4);

        // Simulate: 1 2 + (push, push, pop, pop, push)
        let i64_type = context.i64_type();
        let one = i64_type.const_int(1, false);
        let two = i64_type.const_int(2, false);

        let ops = vec![
            StackOp::Push(one.into()),
            StackOp::Push(two.into()),
            StackOp::Pop,
            StackOp::Pop,
            StackOp::Push(one.into()),
        ];

        let depths = optimizer.analyze_stack_depths(&ops);
        assert_eq!(depths, vec![1, 2, 1, 0, 1]);
    }

    #[test]
    fn test_optimal_cache_depth() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 8);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        // Create sequence that uses 6 stack slots
        let ops = vec![
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
        ];

        let optimal = optimizer.optimal_cache_depth(&ops);
        assert_eq!(optimal, 6);
    }
}
