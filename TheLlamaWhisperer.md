# The Llama Whisperer: Fast-Forth JIT Compiler Development Guide

**For Agents Working on This Codebase**

Last Updated: 2025-11-15
Status: Active Development - Recursion Implementation In Progress

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Concepts](#core-concepts)
3. [Development Workflows](#development-workflows)
4. [The Recursion Challenge](#the-recursion-challenge)
5. [Debugging Strategies](#debugging-strategies)
6. [Performance Considerations](#performance-considerations)
7. [Agent Collaboration Patterns](#agent-collaboration-patterns)
8. [Known Issues & Solutions](#known-issues--solutions)
9. [Quick Reference](#quick-reference)

---

## Architecture Overview

### The Big Picture

Fast-Forth is a **JIT compiler for Forth** that uses Cranelift as its backend. The architecture follows a classic compilation pipeline:

```
Forth Source → Parser → SSA IR → Cranelift IR → Native Code → Execution
```

### Directory Structure

```
fast-forth/
├── frontend/          # Parser & SSA generation
│   └── src/
│       ├── parser.rs  # Forth → AST
│       └── ssa.rs     # AST → SSA IR
├── backend/           # JIT compilation backends
│   └── src/
│       └── cranelift/ # Cranelift JIT (primary backend)
│           ├── compiler.rs   # Two-pass compilation orchestration
│           └── translator.rs # SSA → Cranelift IR translation
├── cli/               # Command-line interface
│   └── execute.rs     # JIT execution entry point
└── src/               # Root crate (legacy pipeline)
    └── main.rs        # CLI binary with Execute command
```

### Key Files & Their Roles

| File | Purpose | Lines | Critical For |
|------|---------|-------|--------------|
| `cli/execute.rs` | Modern JIT execution path | ~195 | Actual execution, testing |
| `backend/src/cranelift/compiler.rs` | Compilation orchestration | ~210 | Two-pass API, finalization |
| `backend/src/cranelift/translator.rs` | SSA→Cranelift translation | ~600 | IR generation, function calls |
| `frontend/src/ssa.rs` | Forth→SSA conversion | ~800 | SSA form generation |
| `src/main.rs` | CLI binary | ~400 | Execute command handler |

---

## Core Concepts

### 1. Stack-Based Calling Convention

Forth is stack-based, but Cranelift expects registers. The bridge:

```rust
// Function signature: fn(stack_ptr: *mut i64) -> *mut i64
//   - Input: pointer to base of data stack
//   - Output: updated pointer (after pushing results)
```

**Stack Layout:**
```
Stack: [arg0, arg1, ..., argN, <-- stack_ptr points here]
       Low addresses          High addresses
```

**Critical Pattern:**
```rust
// Load arguments from stack
let arg1 = builder.ins().load(types::I64, MemFlags::new(), sp, -16);
let arg0 = builder.ins().load(types::I64, MemFlags::new(), sp, -8);

// Push results onto stack
builder.ins().store(MemFlags::new(), result, sp, 0);
let eight = builder.ins().iconst(types::I64, 8);
sp = builder.ins().iadd(sp, eight);  // Move stack pointer up
```

### 2. Two-Pass Compilation (For Function Calls & Recursion)

**Why Two Passes?**
- Functions can call each other (mutual recursion)
- Functions can call themselves (self-recursion)
- Cranelift needs function declarations before generating calls

**The Pattern:**

```rust
// Pass 1: Declare all functions
backend.declare_all_functions(&functions)?;

// Pass 2: Compile each function body
for (name, func) in &functions {
    backend.compile_function(func, name)?;
}

// Pass 3: Finalize (link everything)
backend.finalize_all()?;
```

**CRITICAL:** Do NOT call `finalize_definitions()` between individual function compilations! This breaks recursion because Cranelift can't link incomplete call graphs.

### 3. SSA Form & Dominance

**The Challenge:**
Forth's `if/then/else` creates merge points where values from different branches converge.

**Bad (Causes Verifier Errors):**
```
block1:
    v13 = imul v1, v12    // Only defined here!
    jump block2

block2:                    // Reachable from block1 AND block3
    store v13, sp         // ERROR: v13 not available from block3!

block3:
    v16 = iconst 1
    jump block2
```

**Good (Proper SSA):**
```
block1:
    v13 = imul v1, v12
    jump block2(v13)      // Pass as parameter

block2(vresult: i64):     // Block parameter
    store vresult, sp

block3:
    v16 = iconst 1
    jump block2(v16)      // Pass different value
```

**Current Status:** translator.rs lines 296 & 300 use empty parameter arrays `&[]` — this is the root cause of recursion failures.

---

## Development Workflows

### Building & Testing

```bash
# Build with Cranelift backend
cargo build --features cranelift

# Test basic execution
./target/debug/fastforth execute "42"
# Expected: Result: 42

# Test function calls
./target/debug/fastforth execute ": double 2 * ; 5 double"
# Expected: Result: 10

# Test recursion (CURRENTLY BROKEN)
./target/debug/fastforth execute ": factorial dup 1 > if dup 1 - factorial * else drop 1 then ; 5 factorial"
# Expected: Result: 120
# Actual: Verifier error (SSA dominance violation)
```

### Debugging Cranelift IR

The verbose debug output in `compiler.rs` (lines 119-140) prints:
1. Generated Cranelift IR before verification
2. Detailed verifier errors with exact instruction numbers
3. Function display after errors

**Reading Verifier Errors:**
```
inst16 (store.i64 v13, v11): uses value v13 from non-dominating inst14
         ^^^^^^^^^^^^^^       ^^^^^^^^^^         ^^^^^^^^^^^^^^^^^^^
         What instruction     What's wrong       Where v13 comes from
```

### Common Build Errors

**"Backend not available"**
- Missing `--features cranelift`
- Solution: Always build with `cargo build --features cranelift`

**"Function not declared"**
- Trying to call a function before Pass 1 (declare_all_functions)
- Check compilation order in execute.rs

**"Verifier errors"**
- SSA form violation (dominance, block parameters, etc.)
- Look at the printed IR and verifier output
- Usually means values cross block boundaries without parameters

---

## The Recursion Challenge

### Current Status: BLOCKED

**Problem:** Recursive functions fail Cranelift verification with SSA dominance errors.

**Root Cause:** The if/then/else construct in recursive Forth functions creates merge points where:
1. The "then" branch computes a value (e.g., `n * factorial(n-1)`)
2. The "else" branch computes a different value (e.g., `1`)
3. Both branches jump to a merge block that tries to store the result
4. The translator doesn't use block parameters to pass these values

**What We Tried:**

1. ✅ **Two-pass compilation** - Implemented successfully
2. ✅ **Pre-importing function references** - Added to translator.rs:66-71
3. ❌ **Deferred finalization** - Already in place, not the issue
4. ❌ **Direct fix attempts** - Block parameter implementation needed

### The Solution Path

**Three approaches researched:**

1. **Quick Fix (Recommended First):** Fix SSA dominance by using block parameters
   - Modify translator.rs lines 296 & 300
   - Pass values through jump/brif instructions
   - Estimated effort: 2-4 hours

2. **Production Architecture:** Function table with indirect calls
   - Eliminates all circular dependency issues
   - Enables runtime function replacement (good for REPL)
   - Estimated effort: 1-2 days

3. **Advanced:** Tail-call optimization
   - Only helps tail-recursive cases
   - Complex to implement
   - Defer until after basic recursion works

### Research Documents Created

- `docs/CRANELIFT_RECURSION_RESEARCH.md` - Comprehensive research (13KB)
- `docs/CRANELIFT_RECURSION_QUICKSTART.md` - Quick reference (8KB)

**Key Finding:** Cranelift supports recursion natively through `declare_func_in_func()` — no special handling needed IF the SSA form is correct.

---

## Debugging Strategies

### 1. Verifier Errors = IR Problems

When you see "Verifier errors", the generated Cranelift IR violates SSA form rules. Always:

1. Look at the printed IR (enabled in compiler.rs:119-122)
2. Find the failing instruction number
3. Trace back to where that value was defined
4. Check if all paths to the merge point define that value

### 2. Function Call Debugging

```rust
// In translator.rs, add before/after Call handling:
eprintln!("[CALL] Function: {}, args: {:?}, dest: {:?}", name, args, dest);
eprintln!("[CALL] Func_refs available: {:?}", self.func_refs.keys());
```

### 3. Stack Pointer Tracking

Stack corruption is silent and deadly. Add validation:

```rust
// After each stack operation
let stack_depth = unsafe { sp.offset_from(stack_base) };
assert!(stack_depth >= 0 && stack_depth < STACK_SIZE);
eprintln!("[STACK] Depth: {}, SP: {:?}", stack_depth, sp);
```

### 4. Integration Test Pattern

```rust
#[test]
fn test_recursive_factorial() {
    let result = execute_program(
        ": factorial dup 1 > if dup 1 - factorial * else drop 1 then ; 5 factorial",
        false
    );
    assert!(result.is_ok(), "Compilation failed: {:?}", result);
    assert_eq!(result.unwrap(), 120, "5! should be 120");
}
```

---

## Performance Considerations

### Cranelift vs LLVM Trade-offs

| Aspect | Cranelift | LLVM |
|--------|-----------|------|
| Compile time | ~50ms | ~2-5min |
| Runtime speed | 70-90% of C | 85-110% of C |
| Binary size | Smaller | Larger |
| Use case | Development, iteration | Production, maximum performance |

### Fast Iteration Workflow

```bash
# Development cycle (fast compilation)
cargo build --features cranelift && ./target/debug/fastforth execute "test code"

# Release build (with debug symbols for profiling)
cargo build --release --features cranelift

# Production build (maximum optimization - future LLVM backend)
# cargo build --release --features llvm  # Not yet implemented
```

### Optimization Opportunities (Future Work)

1. **Inline small functions** - Avoid call overhead for 1-2 instruction functions
2. **Constant folding** - Evaluate pure expressions at compile time
3. **Dead code elimination** - Remove unreachable branches
4. **Register allocation hints** - Help Cranelift make better choices

Current optimization level: `-O1` (speed) in CraneliftSettings

---

## Agent Collaboration Patterns

### Multi-Agent Research Pattern (Used Successfully)

**Scenario:** Complex problem requiring multiple perspectives

**Pattern:**
```markdown
/ci-team-sdk "Research recursion in Cranelift from 3 angles in parallel:
1. @developer - Add verbose error output to see exact verifier complaints
2. @researcher - Find Cranelift documentation and examples for recursion
3. @architect - Design alternative approaches (function tables, trampolines, etc.)"
```

**Benefits:**
- Parallel execution (3x faster than sequential)
- Multiple perspectives catch blind spots
- Cross-validation of findings

**Results from our session:**
- Developer found: SSA dominance violation at inst16
- Researcher found: Cranelift recursion examples, pre-import pattern
- Architect designed: 3 alternative architectures with trade-offs

### Task Decomposition

**Good decomposition:**
```
1. Fix printing issue ✓
   ├─ Identify root cause (old vs new pipeline)
   ├─ Expose jit_execute function
   ├─ Update Execute command
   └─ Test and verify

2. Fix recursion (IN PROGRESS)
   ├─ Add verbose error output ✓
   ├─ Research Cranelift recursion ✓
   ├─ Design solutions ✓
   └─ Implement SSA dominance fix (NEXT)
```

### Communication Patterns

**Between agents:**
- Use markdown files in `docs/` for knowledge transfer
- Create Quick Reference guides for rapid onboarding
- Document "current status" and "next steps" clearly

**With user:**
- Show working examples before explaining
- Provide concrete test commands
- Explain trade-offs, not just solutions

---

## Known Issues & Solutions

### Issue 1: Execute Command Printed "0" Instead of "10"

**Symptoms:**
```bash
./target/debug/fastforth execute ": double 2 * ; 5 double"
# Output: 0  (WRONG)
```

**Root Cause:** `src/main.rs` Execute command used old `compiler.compile_string()` pipeline instead of new `jit_execute()` path.

**Solution:** ✅ FIXED
- Modified src/main.rs:345-356 to call `backend::cranelift::jit_execute()`
- Added `jit_execute()` function to backend/src/cranelift/mod.rs
- Added "Result: " label to output

**Files Changed:**
- src/main.rs (Execute command handler)
- backend/src/cranelift/mod.rs (new jit_execute function)
- backend/Cargo.toml (added anyhow dependency)
- root Cargo.toml (made backend non-optional)

### Issue 2: Recursive Functions Fail Verification

**Symptoms:**
```bash
./target/debug/fastforth execute ": factorial dup 1 > if dup 1 - factorial * else drop 1 then ; 5 factorial"
# Error: Verifier errors - inst16 (store.i64 v13, v11): uses value v13 from non-dominating inst14
```

**Root Cause:** SSA dominance violation in if/then/else merge points. Values from "then" branch not available in merge block when coming from "else" branch.

**Status:** ⚠️ IN PROGRESS

**Solution Path:**
1. Modify translator.rs:296 & 300 to use block parameters
2. Pass branch results through jump/brif instructions
3. Accept parameters in merge blocks

**Alternative:** Function table with indirect calls (more robust, slight performance cost)

### Issue 3: Non-Recursive Function Calls Work Perfectly

**Status:** ✅ WORKING

Function calls between different functions work correctly:
```bash
./target/debug/fastforth execute ": triple 3 * ; : nine 3 triple ; nine"
# Result: 9 ✓
```

This proves:
- Two-pass compilation works
- Function declarations work
- Function imports work
- Call instruction generation works

**Only recursion (self-calls) triggers the SSA issue.**

---

## Quick Reference

### Essential Commands

```bash
# Build
cargo build --features cranelift

# Test basic
./target/debug/fastforth execute "42"

# Test function call
./target/debug/fastforth execute ": double 2 * ; 5 double"

# Run all tests
cargo test --features cranelift

# Check for warnings
cargo clippy --features cranelift
```

### Key Code Locations

```
Execute command entry:     src/main.rs:345-356
JIT execution:             cli/execute.rs:9-158
  ├─ Parse:               line 17
  ├─ SSA conversion:      line 37
  ├─ Declare functions:   line 75
  ├─ Compile functions:   line 80-100
  ├─ Finalize:            line 104
  └─ Execute:             line 120-156

Cranelift backend:         backend/src/cranelift/
  ├─ Compiler:            compiler.rs
  │   ├─ declare_all:     line 87
  │   ├─ compile_func:    line 100
  │   └─ finalize_all:    line 160
  └─ Translator:          translator.rs
      ├─ translate:       line 62
      ├─ Branch:          line 283
      ├─ Jump:            line 299
      └─ Call:            line 327 (builtins), 505 (user funcs)
```

### Debugging Checklist

When something breaks:

1. ☐ Is it a compilation error or runtime error?
2. ☐ Does it work without recursion?
3. ☐ What does the Cranelift IR look like?
4. ☐ Are there verifier errors? Which instruction?
5. ☐ Is the stack pointer tracking correctly?
6. ☐ Are all functions declared before compilation?
7. ☐ Is finalize called after ALL compilations?

### Architecture Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-11-15 | Use Cranelift as primary backend | 100x faster compile vs LLVM, good for iteration |
| 2025-11-15 | Two-pass compilation | Required for function calls and recursion |
| 2025-11-15 | Stack-based calling convention | Natural fit for Forth, simple to implement |
| 2025-11-15 | Pre-import all functions | Enables recursion via declare_func_in_func |

---

## Next Agent: Start Here

⚠️ **CRITICAL: GIT STATE RESET - Work was lost, needs restoration!**

**Current Status**: The repository experienced a git checkout during this session that reverted files to a simplified state. The following work from the previous session needs to be restored before fixing the SSA dominance issue:

**Lost Work to Restore**:
1. `backend/src/cranelift/mod.rs::jit_execute()` function - see `cli/execute.rs` lines 8-164 for reference implementation
2. `src/main.rs` line 347: Execute command calling `backend::cranelift::jit_execute()`
3. `backend/Cargo.toml`: Add `anyhow = "1.0"` dependency
4. Root `Cargo.toml`: Make backend non-optional (remove `optional = true`)

**Current Objective:** Fix recursive function support (AFTER restoring lost work above)

**The Problem:** Values cross block boundaries without proper SSA block parameters in if/then/else constructs.

**The Good News:** The current `translator.rs` in git ALREADY uses Cranelift's Variable API (lines 20-26) which automatically handles Phi nodes! This is partially solved.

**The Challenge:** The simplified translator lost the two-pass compilation infrastructure (function_ids, module, func_refs) needed for function calls and recursion.

**Two Paths Forward**:

**Option A - Quick Restore (RECOMMENDED for immediate progress)**:
1. Copy jit_execute() implementation from cli/execute.rs to backend/src/cranelift/mod.rs
2. Update dependencies as listed above
3. Verify basic execution works: `./target/debug/fastforth execute "42"` → `Result: 42`
4. Test recursion to see if Variables API already fixes SSA issue
5. If still fails, proceed to Option B

**Option B - Complete Translator Rewrite (if Option A still has SSA issues)**:
Modify backend/src/cranelift/translator.rs:
1. Restore function call support (function_ids, module parameters)
2. The Variable API is already there - keep it!
3. Ensure all register_map operations use Variables
4. Test: `: factorial dup 1 > if dup 1 - factorial * else drop 1 then ; 5 factorial`

**Expected Result:**
```bash
./target/debug/fastforth execute ": factorial dup 1 > if dup 1 - factorial * else drop 1 then ; 5 factorial"
# Should output: Result: 120
```

**Resources:**
- Cranelift docs: docs/CRANELIFT_RECURSION_QUICKSTART.md
- Current IR output: Run test and check stderr
- Working examples: Non-recursive function calls

**Test Plan:**
1. Simple recursion: factorial
2. Complex recursion: fibonacci
3. Mutual recursion: even/odd predicates
4. Deeply nested: recursive tree traversal

Good luck! The framework is solid, just needs proper SSA form. 🚀

---

## Deep Insights: What I Learned the Hard Way

### On Understanding Cranelift's Mental Model

Cranelift isn't just "LLVM but faster" - it has a fundamentally different philosophy. LLVM optimizes aggressively and assumes you know what you're doing. Cranelift **verifies everything** and fails fast. This is actually a feature, not a bug. When you get a verifier error, Cranelift is telling you "your SSA form is mathematically invalid" - it's not being pedantic, it's catching bugs that would become security vulnerabilities or silent corruption in production.

The key insight: **think in data flow, not control flow**. When you write `if/then/else` in Forth, you're thinking "execute this branch OR that branch". But Cranelift wants you to think "produce this VALUE or that VALUE, then USE the value". Every merge point is asking: "where does this value come from on ALL possible paths?" If the answer isn't clear, you need block parameters to make it explicit. It's like showing your work in math class - Cranelift won't accept "trust me, the value will be there."

### The Recursion Puzzle: A Case Study in Systematic Debugging

We spent hours chasing the recursion bug. Here's what we learned: **the problem was never recursion itself**. Non-recursive function calls worked perfectly. The two-pass compilation was solid. Pre-importing functions was correct. The issue was that recursive functions forced us to confront a deeper problem we'd been ignoring: improper SSA form in conditional branches.

Non-recursive code got lucky - the merge points happened to work because the compiler didn't need to verify as strictly. But recursion made the verifier path more complex, and suddenly the SSA violations became fatal. This is a pattern you'll see in compilers: a bug hides in simple cases and only shows up when complexity increases. The fix isn't "make recursion work" - it's "fix SSA form everywhere, which will make recursion work as a side effect."

### When to Use Multi-Agent Collaboration (And When Not To)

The 3-agent parallel research approach worked beautifully for the recursion problem because we had: (1) a well-defined problem, (2) multiple valid approaches to explore, and (3) independent research domains (implementation, documentation, architecture). The agents didn't step on each other's toes because each had a distinct mandate.

When NOT to parallelize: Don't spawn multiple agents to "try different fixes" on the same code file. They'll conflict. Don't parallelize exploratory debugging - one agent carefully tracing through IR is better than three agents guessing. Parallelize research and design, serialize implementation and testing. Think of it like pair programming: discuss in parallel, code in serial.

### The Stack Pointer Dance: Forth's Secret Handshake

Managing the stack pointer is deceptively simple until it isn't. The pattern is: load arguments going backward (sp-16, sp-8), push results going forward (sp, sp+8). But here's the gotcha: **who owns the stack pointer?** Each function receives it, modifies it, and returns the new value. This means you can't cache it - it's a hot potato that gets passed around.

In the Return instruction (translator.rs:304-324), we carefully track `current_sp` as we push each result. This isn't just bookkeeping - it's the contract between functions. If you mess this up, the stack pointer drifts, and suddenly function arguments are garbage. The symptom? Tests pass individually but fail when chained. The fix? Always use the returned stack pointer, never try to "fix up" the offset yourself.

### Reading Cranelift IR Like a Detective

When you see the printed IR, don't just scan it - read it like you're debugging assembly. Each line has a purpose. `v13 = imul.i64 v1, v12` means "multiply v1 and v12, store in v13". The verifier error `uses value v13 from non-dominating inst14` is telling you: "you're using v13 in this instruction, but v13 comes from instruction 14, which doesn't dominate this point in the CFG."

The dominance rule is simple: **instruction A dominates instruction B if every path from entry to B goes through A**. If there's ANY path that skips A but still reaches B, A doesn't dominate. In our factorial case, block1 (with v13 = imul) doesn't dominate block2 (the merge point) because block3 can also reach block2. The solution isn't to "make block1 dominate" - it's to pass v13 as a parameter so block2 doesn't depend on block1's internal state.

### The Two Mental Models You Need

**Model 1: Forth's Execution Model** - Stack-based, words consume and produce values, control flow via if/then/else. This is what the user sees and what the frontend parser understands.

**Model 2: Cranelift's SSA Model** - Value-based, every value has exactly one definition point, control flow creates merge points that need phi nodes (or block parameters). This is what the backend must produce.

The translator (translator.rs) is the bridge between these models. When it fails, it's because you're thinking in Model 1 but Cranelift demands Model 2. The symptom: "it's obvious where the value comes from!" (Model 1 thinking) vs "the CFG has multiple paths with different definitions" (Model 2 reality). Always think: "if I'm at this merge point, which value do I use, and where did it come from on EACH possible path?"

---

*This document is living documentation. Update it as you discover new patterns, solve problems, or encounter edge cases.*
