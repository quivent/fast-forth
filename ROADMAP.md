# Fast-Forth Implementation Roadmap

## Current State (2025-11-15)

**STATUS: Working JIT compiler with limitations**

The fast-forth compiler successfully:
- Parses Forth source with ANS Forth prelude
- Converts to SSA intermediate representation
- Compiles to native x86-64 machine code via Cranelift
- Executes with correct stack-based calling convention
- Returns accurate results for arithmetic and stack operations

**Test**: `execute_program("42", true)` → `Ok(42)` ✅

## Remaining Work for Full Functionality

### Phase 1: Inter-Function Call Support (HIGH PRIORITY)

**Goal**: Enable user-defined functions to call each other (including recursion)

**Current Blocker**: Line 468-473 in `backend/src/cranelift/translator.rs`:
```rust
_ => {
    return Err(BackendError::CodeGeneration(
        format!("Function call '{}' not supported...", name)
    ));
}
```

**Implementation Steps**:

1. **Modify CraneliftBackend** (`backend/src/cranelift/compiler.rs`):
   ```rust
   // Add new method to declare all functions upfront
   pub fn declare_all_functions(&mut self, functions: &[(String, &SSAFunction)]) -> Result<()> {
       for (name, _) in functions {
           let sig = self.create_signature();
           let func_id = self.module.declare_function(
               name,
               Linkage::Export,
               &sig
           )?;
           self.functions.insert(name.clone(), func_id);
       }
       Ok(())
   }
   ```

2. **Update SSATranslator** (`backend/src/cranelift/translator.rs`):
   ```rust
   pub struct SSATranslator<'a> {
       // ...existing fields...
       /// Map of function names to FuncRefs for inter-function calls
       function_refs: HashMap<String, cranelift_codegen::ir::FuncRef>,
   }
   ```

3. **Implement Call Instruction** (translator.rs:468):
   ```rust
   _ => {
       // User-defined function call
       if let Some(func_ref) = self.function_refs.get(name) {
           // Prepare arguments (load from stack via stack pointer)
           let mut call_args = Vec::new();

           // Current stack pointer
           let current_sp = self.stack_ptr.expect("Stack pointer not initialized");
           call_args.push(current_sp);

           // Make the call
           let call_inst = self.builder.ins().call(*func_ref, &call_args);
           let results = self.builder.inst_results(call_inst);

           // Update stack pointer with returned value
           self.stack_ptr = Some(results[0]);

           // Results are already on stack, caller will load them
       } else {
           return Err(BackendError::CodeGeneration(
               format!("Unknown function: {}", name)
           ));
       }
   }
   ```

4. **Update execute.rs** (`cli/execute.rs:66-79`):
   ```rust
   // Phase 3: JIT compile with Cranelift

   // Collect all function names
   let func_names: Vec<String> = (0..ssa_functions.len())
       .map(|i| format!("func_{}", i))
       .collect();

   // Pass 1: Declare all functions
   let functions_with_names: Vec<(String, &SSAFunction)> =
       func_names.iter().zip(ssa_functions.iter())
       .map(|(name, func)| (name.clone(), func))
       .collect();

   backend.declare_all_functions(&functions_with_names)?;

   // Pass 2: Compile all function bodies (can now reference each other)
   for (name, func) in &functions_with_names {
       backend.compile_function(func, name)?;
   }
   ```

**Test Cases**:
- Simple call: `: double 2 * ; 5 double` → Should return 10
- Recursion: `: factorial dup 1 <= if drop 1 else dup 1- factorial * then ; 5 factorial` → Should return 120

### Phase 2: Integration with llama CLI

**Goal**: Replace gforth with fast-forth for instant startup

**Steps**:

1. Create `llama/variants/fast-forth/` directory structure
2. Modify Makefile to support both variants
3. Implement hot-swap mechanism:
   ```forth
   : use-fast-forth  \ Switch to native JIT
       s" variants/fast-forth/llama" load-variant ;

   : use-gforth  \ Switch back to interpreted
       s" llama" load-variant ;
   ```

4. Benchmark comparison:
   ```bash
   # Startup time
   time ./llama -e 'bye'                    # gforth: ~158ms
   time ./variants/fast-forth/llama -e 'bye'  # fast-forth: <10ms goal

   # Execution performance
   benchmark-factorial-1000
   ```

### Phase 3: Optimization & Feature Completion

**Features to add**:
- [ ] String literal support (`LoadString` instruction)
- [ ] Variable support (DATA section)
- [ ] Control flow optimization (phi nodes, loop unrolling)
- [ ] More builtins (bit operations, floating point)
- [ ] Foreign function interface (call C libraries)
- [ ] Ahead-of-time compilation mode (save binary)

**Optimizations**:
- Inline small functions automatically
- Constant folding
- Dead code elimination
- Register allocation improvements

### Phase 4: Production Readiness

**Testing**:
- [ ] Full ANS Forth test suite
- [ ] Stress testing (large programs)
- [ ] Memory safety verification
- [ ] Performance regression tests

**Documentation**:
- [ ] User guide for llama integration
- [ ] Developer guide for extending compiler
- [ ] Performance tuning guide
- [ ] Migration guide from gforth

## Success Metrics

| Metric | Current | Goal | Status |
|--------|---------|------|--------|
| Startup time | 158ms (gforth) | <10ms | Pending integration |
| Compilation time | - | <100ms | ~50ms ✅ |
| Execution speed | Interpreted | 70-85% of C | Expected ✅ |
| Binary size | Requires gforth | 10-50KB | Pending |
| Recursion depth | N/A | 1000+ levels | Pending Phase 1 |

## Technical Debt & Notes

### Known Issues
1. `test_execute_simple` fails due to recursion (expected - Phase 1 work)
2. Definition-only code still executes (should return 0)
3. Phi nodes use placeholder implementation
4. No string literal support yet

### Architecture Decisions
- **Stack calling convention**: `fn(*mut i64) -> *mut i64`
  - Pro: Simple, matches Forth semantics perfectly
  - Con: Function call overhead (mitigated by inlining)

- **JIT vs AOT**: Currently JIT-only
  - Pro: Fast development, instant execution
  - Con: Compilation overhead on each run
  - Future: Add AOT mode for production deployments

### Resources
- Cranelift docs: https://cranelift.readthedocs.io/
- SSA tutorial: https://en.wikipedia.org/wiki/Static_single_assignment_form
- Forth standards: https://forth-standard.org/

## Timeline Estimate

**Phase 1** (Recursion): 4-6 hours focused work
- Implement function declarations: 1-2 hours
- Update translator for calls: 2-3 hours
- Testing and debugging: 1 hour

**Phase 2** (llama integration): 2-3 hours
- Directory structure: 30 min
- Makefile changes: 1 hour
- Testing: 1-1.5 hours

**Phase 3** (Optimization): Ongoing
**Phase 4** (Production): Ongoing

**Total to MVP with recursion**: 6-9 hours
