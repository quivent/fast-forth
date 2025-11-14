//! Stack cache optimization tests

#[cfg(feature = "llvm")]
mod stack_cache_tests {
    use backend::codegen::stack_cache::{StackCache, StackCacheOptimizer, StackOp};
    use inkwell::context::Context;

    #[test]
    fn test_stack_cache_creation() {
        let context = Context::create();
        let cache = StackCache::new(&context, 4);
        assert_eq!(cache.depth(), 0);
    }

    #[test]
    fn test_stack_depth_analysis() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 4);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        // Simulate: PUSH PUSH POP POP PUSH
        let ops = vec![
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Pop,
            StackOp::Pop,
            StackOp::Push(val.into()),
        ];

        let depths = optimizer.analyze_stack_depths(&ops);
        assert_eq!(depths, vec![1, 2, 1, 0, 1]);
    }

    #[test]
    fn test_optimal_cache_depth_small() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 8);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        let ops = vec![
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
        ];

        let optimal = optimizer.optimal_cache_depth(&ops);
        assert_eq!(optimal, 3);
    }

    #[test]
    fn test_optimal_cache_depth_large() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 8);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        // Create sequence that would need 10 slots
        let mut ops = vec![];
        for _ in 0..10 {
            ops.push(StackOp::Push(val.into()));
        }

        let optimal = optimizer.optimal_cache_depth(&ops);
        // Should be capped at 8 (max practical cache depth)
        assert_eq!(optimal, 8);
    }

    #[test]
    fn test_stack_depth_with_operations() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 4);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        // Simulate: PUSH PUSH DUP (should be 1, 2, 3)
        let ops = vec![
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Dup,
        ];

        let depths = optimizer.analyze_stack_depths(&ops);
        assert_eq!(depths, vec![1, 2, 3]);
    }

    #[test]
    fn test_stack_depth_with_drop() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 4);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        // Simulate: PUSH PUSH DROP (should be 1, 2, 1)
        let ops = vec![
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Drop,
        ];

        let depths = optimizer.analyze_stack_depths(&ops);
        assert_eq!(depths, vec![1, 2, 1]);
    }

    #[test]
    fn test_stack_depth_with_over() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 4);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        // Simulate: PUSH PUSH OVER (should be 1, 2, 3)
        let ops = vec![
            StackOp::Push(val.into()),
            StackOp::Push(val.into()),
            StackOp::Over,
        ];

        let depths = optimizer.analyze_stack_depths(&ops);
        assert_eq!(depths, vec![1, 2, 3]);
    }

    #[test]
    fn test_stack_depth_underflow_protection() {
        let context = Context::create();
        let optimizer = StackCacheOptimizer::new(&context, 4);

        let i64_type = context.i64_type();
        let val = i64_type.const_int(42, false);

        // Simulate: POP (underflow protection should give 0)
        let ops = vec![
            StackOp::Pop,
        ];

        let depths = optimizer.analyze_stack_depths(&ops);
        assert_eq!(depths, vec![0]);
    }
}
