# Stream 5: Type-Driven Specialization - Usage Guide and Examples

## Quick Start

### Basic Usage
```rust
use fastforth_optimizer::{
    ForthIR, TypeSpecializer, TypeInferenceResults, TypeSignature, ConcreteType
};

// Create IR from Forth code
let mut ir = ForthIR::parse(": SQUARE DUP * ;")?;

// Create type inference results
let mut type_info = TypeInferenceResults::new();

// Add type signature for SQUARE called with Int
type_info.add_word_signature(
    "square".to_string(),
    TypeSignature::new(
        vec![ConcreteType::Int],
        vec![ConcreteType::Int]
    )
);

// Specialize
let mut specializer = TypeSpecializer::new();
let stats = specializer.specialize(&mut ir, &type_info)?;

println!("{}", stats);  // View statistics
```

## Detailed Examples

### Example 1: Simple Polymorphic Word

#### Input Code
```forth
: DOUBLE ( n -- 2n )  2 * ;
: TRIPLE ( n -- 3n )  3 * ;

: MIXED-MATH ( -- )
  5 DOUBLE      \ Int operand
  3.14 TRIPLE   \ Float operand
  +
;
```

#### Type Analysis
```
DOUBLE with Int:    ( Int -- Int )     → Needs specialization
DOUBLE with Float:  ( Float -- Float ) → Needs specialization
TRIPLE with Int:    ( Int -- Int )     → Needs specialization
TRIPLE with Float:  ( Float -- Float ) → Needs specialization
```

#### Generated Specializations
```forth
: DOUBLE_INT ( Int -- Int )
  2 IMUL    \ Integer multiply

: DOUBLE_FLOAT ( Float -- Float )
  2.0 FMUL  \ Float multiply

: TRIPLE_INT ( Int -- Int )
  3 IMUL

: TRIPLE_FLOAT ( Float -- Float )
  3.0 FMUL

: MIXED-MATH ( -- )
  5 DOUBLE_INT      \ Direct Int operation
  3.14 TRIPLE_FLOAT \ Direct Float operation
  +
;
```

#### Performance Impact
- Dispatch overhead eliminated
- LLVM can optimize Int and Float paths separately
- Expected speedup: 12-15%

### Example 2: Stack Operations with Type Information

#### Input
```forth
: DUPLICATE-AND-SQUARE ( n -- n n² )
  DUP DUP * ;

: CALC
  42 DUPLICATE-AND-SQUARE
  3.14159 DUPLICATE-AND-SQUARE
;
```

#### Specialization
```rust
// Type signature for Int
TypeSignature::new(
    vec![ConcreteType::Int],
    vec![ConcreteType::Int, ConcreteType::Int]
)

// Type signature for Float
TypeSignature::new(
    vec![ConcreteType::Float],
    vec![ConcreteType::Float, ConcreteType::Float]
)
```

#### Generated Code
```forth
: DUPLICATE-AND-SQUARE-INT ( int -- int int )
  DUP DUP IMUL  # Specialized for integers

: DUPLICATE-AND-SQUARE-FLOAT ( float -- float float )
  DUP DUP FMUL  # Specialized for floats

: CALC
  42 DUPLICATE-AND-SQUARE-INT
  3.14159 DUPLICATE-AND-SQUARE-FLOAT
;
```

#### Benefits
- Type-specific stack operations
- Register allocator knows exact types
- Better cache utilization
- Expected speedup: 8-12%

### Example 3: Comparison Operations

#### Input
```forth
: MAX ( a b -- max )
  2DUP > IF DROP ELSE NIP THEN ;

: FIND-MAX ( n1 n2 f1 f2 -- )
  OVER OVER MAX     \ Max of ints
  ROT ROT MAX        \ Max of floats
  . .
;
```

#### Type Analysis
```
First MAX call: ( Int Int -- Int )
  Uses: INT comparison, INT memory ops

Second MAX call: ( Float Float -- Float )
  Uses: FLOAT comparison, FLOAT memory ops
```

#### Specialized Versions
```forth
: MAX-INT ( int int -- int )
  2DUP ICMP-GT IF DROP ELSE NIP THEN

: MAX-FLOAT ( float float -- float )
  2DUP FCMP-GT IF DROP ELSE NIP THEN
```

#### Comparison Instructions Generated
| Operation | Type  | Instruction |
|-----------|-------|------------|
| >         | Int   | icmp sgt   |
| >         | Float | fcmp ogt   |
| <         | Int   | icmp slt   |
| <         | Float | fcmp olt   |
| ==        | Int   | icmp eq    |
| ==        | Float | fcmp oeq   |

### Example 4: Memory Operations with Type Information

#### Input
```forth
: STORE-VAL ( addr val -- )  ! ;
: LOAD-VAL ( addr -- val )   @ ;

: MIXED-MEM
  BUFFER 42 STORE-VAL
  BUFFER LOAD-VAL
;
```

#### Type Information
```
STORE-VAL with Int:   ( Addr Int -- )
LOAD-VAL with Int:    ( Addr -- Int )
```

#### Specialization Details
```rust
// Type-aware memory operations
Instruction::Store with ConcreteType::Int
  → Generates optimized 64-bit store
  → LLVM can infer alignment
  → Enables prefetching analysis

Instruction::Load with ConcreteType::Int
  → Generates optimized 64-bit load
  → Enables alias analysis
  → Improves memory disambiguation
```

### Example 5: Complex Polymorphic Pattern

#### Input
```forth
: VECTOR-DOT ( ptr-a ptr-b n -- dot )
  0 ROT ROT DO
    I @ SWAP I @ * +
  LOOP ;

: CALC-DISTANCES ( -- )
  INT-VEC1 INT-VEC2 10 VECTOR-DOT
  FLOAT-VEC1 FLOAT-VEC2 5 VECTOR-DOT
  + .
;
```

#### Expected Specializations
```
VECTOR-DOT-INT (addr addr int -- int)
  Uses:
    - Int load (LLVM: i64 load)
    - Int multiply (LLVM: imul)
    - Int add (LLVM: add)

VECTOR-DOT-FLOAT (addr addr int -- float)
  Uses:
    - Float load (LLVM: double load)
    - Float multiply (LLVM: fmul)
    - Float add (LLVM: fadd)
```

#### Performance Analysis
```
Original:
  - Runtime dispatch per operation: 3 * N operations
  - Generic code path
  - Poor branch prediction

Specialized:
  - Direct type-specific path
  - LLVM can unroll loops
  - Predictable code
  - Expected speedup: 15-20%
```

## Statistics Interpretation

### Example Output
```
Type Specialization Statistics:
  Words analyzed: 42
  Polymorphic words: 15
  Specializations created: 28
    - Int specializations: 18
    - Float specializations: 10
  Call sites rewritten: 67
  Type dispatch eliminations: 25
  Avg specialized size: 4.3 instructions
  Code size impact: 12.4%
  Estimated performance improvement: 18.2%
```

### Interpretation

| Metric | Value | Meaning |
|--------|-------|---------|
| Words analyzed | 42 | Total words examined for specialization |
| Polymorphic words | 15 | Words with multiple type signatures (candidates) |
| Specializations created | 28 | Specialized versions generated |
| Int specializations | 18 | Integer-specific versions |
| Float specializations | 10 | Float-specific versions |
| Call sites rewritten | 67 | Function calls updated to use specialized versions |
| Dispatch eliminations | 25 | Runtime type checks removed |
| Avg specialized size | 4.3 | Average instructions per specialized word |
| Code size impact | 12.4% | Total code size increase (trade-off) |
| Estimated speedup | 18.2% | Expected performance improvement |

## Performance Characteristics

### Specialization Decision Tree
```
Word called multiple times?
├─ No  → Don't specialize
└─ Yes → Has concrete types?
         ├─ No  → Don't specialize
         └─ Yes → Polymorphic OR uses Int/Float?
                  ├─ No  → Don't specialize
                  └─ Yes → SPECIALIZE!
```

### Code Size Trade-offs

| Scenario | Code Size Impact | Speedup | Decision |
|----------|------------------|---------|----------|
| Hot loop, 2 types | +5% | +15% | ✓ Specialize |
| Cold path, many types | +20% | +5% | ✗ Don't specialize |
| Medium hot, 3 types | +12% | +12% | ✓ Specialize |

### Optimization Opportunities Enabled

After type specialization, downstream optimizations become:

1. **Constant Folding**
   - Can fold type-specific constants
   - Knows exact integer/float operations

2. **Inlining**
   - Monomorphic code is safer to inline
   - Better code size estimates

3. **Superinstruction Recognition**
   - Type-specific patterns easier to recognize
   - DUP-IMUL vs DUP-FMUL clearly differentiated

4. **Stack Caching**
   - Known types improve register allocation
   - Better cache planning decisions

5. **Memory Optimization**
   - Type information enables better alias analysis
   - Prefetching decisions become clearer

## Practical Guidelines

### When to Enable Type Specialization

✓ **Good Candidates:**
- Polymorphic library functions called with known types
- Math-heavy code (Int/Float operations)
- Hot paths with multiple type patterns
- Code with dispatch-sensitive loops

✗ **Poor Candidates:**
- Single-type code (already monomorphic)
- Very rarely called words
- Code size-constrained environments
- Words with many (>5) different type signatures

### Configuration Tips

```rust
// For performance-critical code
let mut optimizer = Optimizer::new(OptimizationLevel::Aggressive);

// Type specialization runs early, provides benefits to all downstream passes
let ir = optimizer.optimize_with_types(ir, &type_info)?;

// Check if specialization was worthwhile
let stats = optimizer.specialization_stats();
if stats.estimated_speedup_percent >= 10.0 {
    println!("Specialization worthwhile: {:.1}%", stats.estimated_speedup_percent);
}
```

## Validation and Testing

### Basic Validation
```rust
#[test]
fn validate_specialization() {
    let mut ir = ForthIR::new();
    // Add word definitions

    let mut type_info = TypeInferenceResults::new();
    // Add type signatures

    let mut specializer = TypeSpecializer::new();
    let stats = specializer.specialize(&mut ir, &type_info)?;

    // Verify results
    assert!(stats.specializations_created > 0);
    assert!(stats.estimated_speedup_percent >= 10.0);
}
```

### Performance Validation
Compare before/after with benchmarks using real Forth programs:
- Vector operations (Int/Float)
- Numerical algorithms
- List processing with mixed types

## Summary

Type-driven specialization provides:
- **10-15%** speedup from dispatch elimination
- **3-7%** speedup from type-specific optimizations
- **2-5%** speedup from specialized instructions
- **Total: 15-20%** expected improvement

The implementation is transparent to users and automatically improves performance for polymorphic Forth code.
