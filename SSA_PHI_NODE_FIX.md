# SSA Phi Node Fix for Nested Control Flow

## Problem
The SSA generator was creating invalid Phi nodes for nested if-then-else structures, causing validation failures:
```
SSA validation failed for ack: SSA conversion error: Phi node for %19 in block bb2 missing incoming value from predecessor bb5
```

## Root Cause
When generating SSA for nested if-then-else structures, the `convert_if` function assumed that:
- The `then_block` would jump directly to the `merge_block`
- The `else_block` would jump directly to the `merge_block`

However, when branches contain nested control flow (e.g., another if-then-else inside), the actual control flow creates intermediate merge blocks that jump to the outer merge block.

### Example Control Flow
```
Outer IF (block 0)
├─ Then: Simple code (block 1) → Outer Merge (block 2) ✓
└─ Else: Inner IF (block 6)
   ├─ Then (block 3) ──┐
   └─ Else (block 4) ──┼─→ Inner Merge (block 5)
                       │    └─→ Outer Merge (block 2)

Block 2's Phi incorrectly listed: (block 1, block 6)
Block 2's actual predecessors:   (block 1, block 5)
```

Block 6 doesn't jump to block 2 - it branches to blocks 3 and 4. Block 5 (the inner merge) jumps to block 2.

## Solution
Track the **actual block** we're in after converting each branch, not the initial branch target block:

### Changes to `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/frontend/src/ssa.rs`

1. **After converting then branch** (line 991):
   ```rust
   // Track which block we're actually in after conversion
   let actual_then_block = self.current_block;
   ```

2. **After converting else branch** (lines 997-1009):
   ```rust
   let (else_final, actual_else_block) = if let Some(else_words) = else_branch {
       self.set_current_block(else_block);
       let mut else_stack = original_stack.clone();
       self.convert_sequence(else_words, &mut else_stack)?;
       let result = else_stack.clone();
       let actual_block = self.current_block;  // Track actual block
       self.emit(SSAInstruction::Jump {
           target: merge_block,
       });
       (result, actual_block)
   } else {
       (original_stack.clone(), else_block)
   };
   ```

3. **Use actual blocks in Phi nodes** (lines 1050-1053):
   ```rust
   self.emit(SSAInstruction::Phi {
       dest: phi_reg,
       incoming: vec![
           (actual_then_block, then_reg),  // Not then_block
           (actual_else_block, else_reg),  // Not else_block
       ],
   });
   ```

## Why This Works
After `convert_sequence` processes a branch that contains nested control flow:
- `self.current_block` is updated to point to the last block created (e.g., an inner merge block)
- The subsequent `Jump` instruction is emitted from this actual block
- The Phi node now correctly lists this actual predecessor

## Test Results

### Ackermann Function (Primary Test Case)
```forth
: ack over 0 = if swap drop 1 + else dup 0 = if drop 1 - 1 ack else over swap 1 - ack swap 1 - swap ack then then ;
```

| Input | Expected | Actual | Status |
|-------|----------|--------|--------|
| 0, 10 | 11       | 11     | ✓      |
| 1, 5  | 7        | 7      | ✓      |
| 2, 3  | 9        | 9      | ✓      |
| 3, 4  | 125      | 125    | ✓      |

### Nested If-Then-Else (2 levels)
```forth
: test1 dup 0 = if drop 1 else dup 1 = if drop 2 else drop 3 then then ;
```

| Input | Expected | Actual | Status |
|-------|----------|--------|--------|
| 0     | 1        | 1      | ✓      |
| 1     | 2        | 2      | ✓      |
| 5     | 3        | 3      | ✓      |

### Deeply Nested Control Flow (4 levels)
```forth
: triple-nest dup 0 = if drop 100 else dup 1 = if drop 200 else dup 2 = if drop 300 else dup 3 = if drop 400 else drop 500 then then then then ;
```

| Input | Expected | Actual | Status |
|-------|----------|--------|--------|
| 0     | 100      | 100    | ✓      |
| 2     | 300      | 300    | ✓      |
| 10    | 500      | 500    | ✓      |

### Full Test Suite
```
cargo test --lib
test result: ok. 120 passed; 0 failed; 0 ignored; 0 measured
```

### Additional Edge Cases Verified

#### Nested If in Then Branch
```forth
: test-then-nest dup 10 < if dup 5 < if drop 1 else drop 2 then else drop 3 then ;
```
| Input | Expected | Actual | Status |
|-------|----------|--------|--------|
| 3     | 1        | 1      | ✓      |
| 7     | 2        | 2      | ✓      |
| 15    | 3        | 3      | ✓      |

#### Nested If in Both Then and Else Branches
```forth
: test-both-nest dup 10 < if dup 5 < if drop 1 else drop 2 then else dup 15 < if drop 3 else drop 4 then then ;
```
| Input | Expected | Actual | Status |
|-------|----------|--------|--------|
| 3     | 1        | 1      | ✓      |
| 7     | 2        | 2      | ✓      |
| 12    | 3        | 3      | ✓      |
| 20    | 4        | 4      | ✓      |

### Clean Build Verification
```
cargo clean && cargo test --lib
test result: ok. 120 passed; 0 failed; 0 ignored; 0 measured
```

## Technical Details

### SSA Validation Requirements
The SSA validator (`frontend/src/ssa_validator.rs:428-437`) enforces:
- Every actual predecessor (block with Jump/Branch to target) must appear in Phi incoming list
- Every Phi incoming block must be an actual predecessor (no extra entries)

### Why Previous Fix Was Incomplete
The previous fix in `PHI_NODE_FIX_SUMMARY.md` modified the Cranelift backend to handle the symptom (missing predecessors in Phi nodes). This new fix addresses the root cause in SSA generation, ensuring Phi nodes are created correctly from the start.

### Nested Control Flow Handling
The fix handles arbitrary nesting depths because:
- Each level of nesting updates `self.current_block` to reflect the actual control flow
- The Jump instruction is always emitted from the correct block
- Phi nodes reference the actual blocks that contain the Jump instructions

## Files Modified
- `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/fast-forth/frontend/src/ssa.rs`
  - Lines 955-1067: `convert_if` function
  - Added `actual_then_block` and `actual_else_block` tracking
  - Updated Phi node generation to use actual blocks

## Verification
✓ All 120 library tests pass
✓ Ackermann function executes correctly (all test cases)
✓ Simple nested if-then-else works (2 levels)
✓ Deep nesting works (4+ levels)
✓ No new validation errors introduced
✓ No regressions in existing functionality

## Success Criteria Met
- ✓ Ackermann function executes successfully and returns 125
- ✓ All 120 library tests pass (previously 117, 3 new tests added)
- ✓ No new validation errors introduced
