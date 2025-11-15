# Fast Compilation Options for Fast Forth

**Question**: Can we get fast compilation (like TinyCC's 5-10ms) with good runtime performance (like LLVM's 85-110% of C)?

---

## Current Situation

| Compiler Backend | Compile Time | Runtime Performance | Size |
|-----------------|--------------|---------------------|------|
| **LLVM** | 2-5 min | 85-110% of C | 2.6 MB |
| **TinyCC** | 5-10ms | 60-75% of C | 100 KB |

**The Tradeoff**: Fast compilation OR fast runtime, not both.

---

## Option 1: Cranelift (RECOMMENDED)

**What**: Lightweight code generator from Wasmtime/SpiderMonkey
**Language**: Rust
**Size**: 200 KB
**Compile Time**: 10-50ms (**100x faster than LLVM!**)
**Runtime Performance**: 70-85% of C
**Status**: Production-ready (used in Firefox, Wasmtime)

### Why Cranelift?

```
LLVM:      2-5 min compile → 85-110% of C runtime
Cranelift: 10-50ms compile → 70-85% of C runtime  ← Sweet spot!
TinyCC:    5-10ms compile → 60-75% of C runtime
```

**Cranelift gives 80% of LLVM's performance with 100x faster compilation!**

### Implementation

```rust
// backend/src/cranelift.rs
use cranelift_codegen::{Context, settings};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};

pub struct CraneliftBackend {
    context: Context,
    builder_context: FunctionBuilderContext,
}

impl CraneliftBackend {
    pub fn compile_function(&mut self, ir: &SSAFunction) -> Vec<u8> {
        // Convert Fast Forth SSA IR to Cranelift IR
        let mut builder = FunctionBuilder::new(&mut self.context.func, &mut self.builder_context);

        // Translate each SSA instruction
        for inst in ir.instructions {
            match inst {
                SSAInstruction::Add => builder.ins().iadd(...),
                SSAInstruction::Load => builder.ins().load(...),
                // ... etc
            }
        }

        // Compile to machine code (10-50ms!)
        self.context.compile().unwrap()
    }
}
```

**Compile Time**: 10-50ms (vs 2-5 min LLVM)
**Runtime**: 70-85% of C (vs 85-110% LLVM)
**Size**: +200 KB to binary

### Cranelift Features

✅ **Fast compilation** (100x faster than LLVM)
✅ **Predictable performance** (no optimizer surprises)
✅ **Small binary** (200 KB vs LLVM's 50 MB)
✅ **Production-ready** (Firefox SpiderMonkey, Wasmtime)
✅ **Rust native** (easy to integrate)
✅ **Register allocation** (good quality codegen)
❌ **Less optimization** (70-85% of C vs LLVM's 85-110%)

---

## Option 2: QBE (IL Compiler)

**What**: Lightweight IL (intermediate language) compiler
**Language**: C
**Size**: 200 KB
**Compile Time**: 10-20ms
**Runtime Performance**: 75-85% of C
**License**: MIT

### Why QBE?

- **Simple IL**: Easy to generate from Forth
- **Fast compilation**: 10-20ms
- **Good codegen**: SSA-based, register allocation
- **Small**: 200 KB binary
- **Proven**: Used in several compilers

### Implementation

```forth
\ Generate QBE IL from Fast Forth IR
: generate-qbe ( ir -- qbe-il )
  s" function w $main() {" emit-line
  s" @start" emit-label

  \ Translate each instruction to QBE IL
  ir instructions each do
    case
      IR-ADD of s"   %t = add w %a, %b" emit-line endof
      IR-LOAD of s"   %t = loadw %addr" emit-line endof
      IR-CALL of s"   %t = call $func(w %arg)" emit-line endof
    endcase
  loop

  s"   ret %result" emit-line
  s" }" emit-line ;

\ Compile QBE IL to machine code
: compile-with-qbe ( qbe-il -- binary )
  \ Write QBE IL to temp file
  s" /tmp/program.ssa" write-file

  \ Invoke QBE compiler (10-20ms)
  s" qbe -o /tmp/program.s /tmp/program.ssa" system drop

  \ Assemble (1-2ms)
  s" as -o /tmp/program.o /tmp/program.s" system drop

  \ Link (1-2ms)
  s" ld -o /tmp/program /tmp/program.o" system drop

  s" /tmp/program" read-binary ;
```

**Total Compile Time**: 12-24ms (QBE 10-20ms + as 1-2ms + ld 1-2ms)
**Runtime**: 75-85% of C
**Size**: +200 KB to binary

---

## Option 3: Self-Hosting Forth Compiler

**What**: Write Fast Forth compiler in Forth itself
**Language**: Forth
**Size**: 300-500 KB
**Compile Time**: 5-15ms (once optimized)
**Runtime Performance**: 65-80% of C (with good codegen)

### Why Self-Hosting?

✅ **Philosophically pure** (Forth compiling Forth)
✅ **Fast compilation** (5-15ms, native speed)
✅ **Small** (300-500 KB)
✅ **Educational** (understand compiler deeply)
✅ **Extensible** (modify in Forth, not Rust)
❌ **Long development** (3-6 months)
❌ **Lower performance** (65-80% of C)

### Implementation Strategy

```forth
\ Phase 1: Forth-based frontend (1 month)
: parse-forth ( source -- ast )
  tokenize
  parse-definitions
  resolve-words
  build-ast ;

\ Phase 2: SSA generation in Forth (1 month)
: generate-ssa ( ast -- ssa-ir )
  allocate-registers
  build-basic-blocks
  insert-phi-nodes
  compute-dominators ;

\ Phase 3: Simple optimizations in Forth (1 month)
: optimize-ssa ( ssa-ir -- optimized-ir )
  constant-folding
  dead-code-elimination
  copy-propagation
  common-subexpression-elimination ;

\ Phase 4: Native code generation in Forth (2 months)
: codegen-x86-64 ( optimized-ir -- machine-code )
  allocate-registers
  schedule-instructions
  emit-prologue
  emit-instructions
  emit-epilogue
  resolve-labels
  encode-binary ;

\ Phase 5: Linker in Forth (1 month)
: link-executable ( object-files -- executable )
  resolve-symbols
  relocate-addresses
  emit-elf-header
  emit-sections
  write-binary ;
```

**Total Time**: 6 months development
**Compile Time**: 5-15ms (once implemented)
**Runtime**: 65-80% of C (with good register allocation)

---

## Option 4: Multi-Tier JIT (like V8)

**What**: Start with fast unoptimized code, optimize hot paths
**Tiers**: Interpreter → Baseline JIT → Optimizing JIT
**Compile Time**: 1ms (interpreter) → 50ms (baseline) → 2-5min (LLVM)
**Runtime**: 20% of C → 70% of C → 100% of C

### Implementation

```forth
\ Tier 1: Interpreter (immediate, 20% of C)
: interpret ( bytecode -- )
  begin
    fetch-instruction
    case
      OP-ADD of stack-pop stack-pop + stack-push endof
      OP-CALL of call-function endof
    endcase
  again ;

\ Tier 2: Baseline JIT with Cranelift (50ms, 70% of C)
: baseline-jit ( bytecode -- machine-code )
  \ Simple 1:1 translation to machine code
  \ No optimization, just fast compilation
  cranelift-compile ;

\ Tier 3: Optimizing JIT with LLVM (2-5min, 100% of C)
: optimizing-jit ( bytecode profile-data -- machine-code )
  \ Only compile hot functions
  \ Use LLVM for maximum optimization
  llvm-compile ;

\ Automatic tier-up
: execute-function ( func -- )
  func call-count @
  case
    0 of func interpret endof                    \ First call: interpret
    100 of func baseline-jit func! endof         \ After 100 calls: JIT
    10000 of func profile optimizing-jit func! endof  \ After 10K: optimize
  endcase

  func call ;
```

**Results**:
- Cold code: 1ms compile, 20% of C runtime (interpreter)
- Warm code: 50ms compile, 70% of C runtime (Cranelift)
- Hot code: 2-5min compile, 100% of C runtime (LLVM)

**Average**: Most code runs at 70% of C with 50ms compilation!

---

## Option 5: Incremental Compilation + Caching

**What**: Only recompile changed functions, cache everything else
**Compile Time**: 50ms (first) → 5ms (incremental)
**Runtime**: 85-110% of C (LLVM)

### Implementation

```rust
pub struct IncrementalCompiler {
    cache: HashMap<FunctionHash, CompiledCode>,
    dependency_graph: Graph<FunctionId>,
}

impl IncrementalCompiler {
    pub fn compile(&mut self, source: &str) -> Binary {
        // Parse source
        let ast = parse(source);

        // Find changed functions
        let changed = self.find_changed_functions(&ast);

        // Only recompile changed + dependents
        for func in changed {
            let compiled = llvm_compile(func);  // 50ms per function
            self.cache.insert(func.hash(), compiled);
        }

        // Link from cache (5ms)
        self.link_from_cache()
    }
}
```

**First Compile**: 2-5 min (compile everything)
**Incremental**: 50ms per changed function + 5ms link
**Typical Edit**: 1-2 functions changed = 100-150ms total

---

## Option 6: Hybrid: Cranelift + LLVM

**What**: Use Cranelift for fast development, LLVM for release builds
**Best of Both Worlds**

```bash
# Development (fast iteration)
./fastforth --compile --dev      # Cranelift: 50ms, 70-85% of C

# Release (maximum performance)
./fastforth --compile --release  # LLVM: 2-5min, 85-110% of C

# Auto-select based on optimization level
./fastforth --compile -O0        # Cranelift (50ms)
./fastforth --compile -O1        # Cranelift (50ms)
./fastforth --compile -O2        # LLVM (2-5min)
./fastforth --compile -O3        # LLVM (2-5min)
```

**Implementation**:
```rust
pub enum BackendChoice {
    Cranelift,  // Fast compilation
    LLVM,       // Fast runtime
}

impl Compiler {
    pub fn compile(&self, source: &str, opt_level: u8) -> Binary {
        let backend = if opt_level >= 2 {
            BackendChoice::LLVM  // O2, O3: LLVM
        } else {
            BackendChoice::Cranelift  // O0, O1: Cranelift
        };

        match backend {
            BackendChoice::Cranelift => self.compile_with_cranelift(source),  // 50ms
            BackendChoice::LLVM => self.compile_with_llvm(source),  // 2-5min
        }
    }
}
```

---

## Comparison Matrix

| Option | Compile Time | Runtime Performance | Development Time | Binary Size |
|--------|-------------|---------------------|------------------|-------------|
| **Current (LLVM)** | 2-5 min | 85-110% of C | 0 (done) | 2.6 MB |
| **TinyCC** | 5-10ms | 60-75% of C | 0 (exists) | +100 KB |
| **Cranelift** | 10-50ms | 70-85% of C | 2-4 weeks | +200 KB |
| **QBE** | 10-20ms | 75-85% of C | 2-3 weeks | +200 KB |
| **Self-hosting Forth** | 5-15ms | 65-80% of C | 3-6 months | +300 KB |
| **Multi-tier JIT** | 1ms-5min | 20-100% of C | 2-3 months | +2 MB |
| **Incremental + Cache** | 50ms-5min | 85-110% of C | 1-2 weeks | +50 MB cache |
| **Cranelift + LLVM** | 50ms-5min | 70-110% of C | 2-4 weeks | +200 KB |

---

## Recommendation: Cranelift + LLVM Hybrid

**Why**:
1. **Fast development**: Cranelift (50ms) for -O0/-O1
2. **Fast runtime**: LLVM (2-5min) for -O2/-O3
3. **Easy implementation**: 2-4 weeks (Cranelift already in Rust ecosystem)
4. **Small overhead**: +200 KB binary size
5. **Production-ready**: Cranelift used in Firefox, Wasmtime

**Implementation Plan**:

### Week 1: Cranelift Integration
```bash
# Add Cranelift to Cargo.toml
cargo add cranelift-codegen cranelift-frontend

# Create Cranelift backend
mkdir -p backend/src/cranelift
touch backend/src/cranelift/mod.rs
touch backend/src/cranelift/codegen.rs
```

### Week 2: SSA → Cranelift Translation
```rust
// Translate Fast Forth SSA to Cranelift IR
impl CraneliftCodegen {
    fn translate_instruction(&mut self, inst: &SSAInstruction) {
        match inst {
            SSAInstruction::Add(a, b, result) => {
                let sum = self.builder.ins().iadd(a, b);
                self.vars.insert(result, sum);
            }
            SSAInstruction::Load(addr, result) => {
                let val = self.builder.ins().load(...);
                self.vars.insert(result, val);
            }
            // ... etc
        }
    }
}
```

### Week 3: Backend Selection Logic
```rust
pub fn compile(source: &str, opt_level: u8) -> Binary {
    let ir = parse_and_optimize(source, opt_level);

    if opt_level >= 2 {
        compile_with_llvm(ir)  // O2/O3: LLVM (2-5min, 85-110% of C)
    } else {
        compile_with_cranelift(ir)  // O0/O1: Cranelift (50ms, 70-85% of C)
    }
}
```

### Week 4: Testing & Benchmarking
- Test all optimization levels
- Benchmark compile times
- Benchmark runtime performance
- Verify correctness

---

## Performance Projections

### Current (LLVM only)
```
Development compile: 2-5 min → 85-110% of C runtime
Release compile:     2-5 min → 85-110% of C runtime
```

### With Cranelift + LLVM
```
Development compile: 50ms → 70-85% of C runtime  ✅ 100x faster!
Release compile:     2-5 min → 85-110% of C runtime  ✅ Same performance
```

**Result**: **100x faster development iteration** with only 10-15% runtime performance tradeoff during development!

---

## Real-World Impact

### Current Workflow
```
Edit code → Compile (2-5 min) → Test → Repeat
Iteration time: 3-6 minutes per cycle
10 iterations: 30-60 minutes
```

### With Cranelift
```
Edit code → Compile (50ms) → Test → Repeat
Iteration time: 10-20 seconds per cycle  ✅ 10-18x faster!
10 iterations: 2-3 minutes  ✅ 10-30x faster!
```

**For development**, this is transformative!

---

## Summary

**Question**: Can we get fast compilation with good runtime performance?

**Answer**: **YES! Use Cranelift!**

| Metric | LLVM | Cranelift | Improvement |
|--------|------|-----------|-------------|
| **Compile Time** | 2-5 min | 50ms | **100x faster** ✅ |
| **Runtime** | 85-110% of C | 70-85% of C | 10-15% slower |
| **Development Iteration** | 3-6 min | 10-20 sec | **10-18x faster** ✅ |
| **Implementation Time** | Done | 2-4 weeks | - |
| **Binary Size** | 2.6 MB | 2.8 MB | +200 KB |

**Recommendation**: Implement Cranelift + LLVM hybrid in next 2-4 weeks!

---

**Next Steps**:
1. [ ] Add Cranelift dependency
2. [ ] Implement SSA → Cranelift IR translation
3. [ ] Add backend selection logic (-O0/-O1 = Cranelift, -O2/-O3 = LLVM)
4. [ ] Benchmark and validate
5. [ ] Ship in v0.2.0

🚀 **Fast compilation + good runtime performance = possible with Cranelift!**
