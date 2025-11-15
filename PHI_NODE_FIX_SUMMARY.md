# Cranelift Phi Node Bug Fix Summary

## Problem
The Cranelift backend was failing to compile nested if-then-else constructs with the error:
```
Phi node in block BlockId(2) missing incoming value from block BlockId(5)
```

This prevented execution of the Ackermann function and other nested control flow.

## Root Cause
The `collect_branch_args` function in `backend/src/cranelift/translator.rs` only looked for direct predecessors in Phi node incoming lists. However, with nested if-then-else:

```
Outer IF (block 0)
├─ Then: Inner IF (block 1)
│  ├─ Then (block 3) ──┐
│  └─ Else (block 4) ──┼─→ Inner Merge (block 5) [has Phi]
│                      │    └─→ Outer Merge (block 2) [has Phi expecting blocks 1, 6]
└─ Else (block 6) ─────────→ Outer Merge (block 2)
```

Block 5 (inner merge) jumps to block 2 (outer merge), but block 2's Phi expects values from blocks 1 and 6, not block 5. The SSA generation creates Phi nodes based on abstract control flow (outer then/else), but actual execution creates intermediate merge blocks.

## Solution
Modified `collect_branch_args` to handle indirect control flow:

1. **First**: Try to find `from_block` directly in the target Phi's incoming list (original behavior)
2. **If not found**: Check if `from_block` has a Phi node
3. **If yes**: Use the Phi's destination register - this represents the merged value from that block's control flow, which is exactly what should be passed forward

## Changes Made

### File: `backend/src/cranelift/translator.rs`

1. **Added control flow graph tracking**:
   ```rust
   block_predecessors: HashMap<BlockId, Vec<BlockId>>
   ```

2. **Updated Branch and Jump instructions** to record control flow edges

3. **Fixed `collect_branch_args` function** to handle indirect paths:
   - When `from_block` is not in target Phi's incoming list
   - But `from_block` has a Phi node
   - Use that Phi's destination register as the value to pass

## Test Results

### Nested If-Then-Else
```forth
: test1 dup 0 = if drop 1 else dup 1 = if drop 2 else drop 3 then then ;
0 test1  → 1 ✓
1 test1  → 2 ✓
5 test1  → 3 ✓
```

### Ackermann Function
```forth
: ack over 0 = if swap drop 1 + else dup 0 = if drop 1 - 1 ack else over swap 1 - ack swap 1 - swap ack then then ;
ack(0,10) → 11 ✓
ack(1,5)  → 7 ✓
ack(2,3)  → 9 ✓
ack(3,4)  → 125 ✓
```

### Deeply Nested Control Flow (4 levels)
```forth
: triple-nest dup 0 = if drop 100 else dup 1 = if drop 200 else dup 2 = if drop 300 else dup 3 = if drop 400 else drop 500 then then then then ;
0 triple-nest  → 100 ✓
2 triple-nest  → 300 ✓
10 triple-nest → 500 ✓
```

### Full Test Suite
```
cargo test --lib
test result: ok. 117 passed; 0 failed; 0 ignored; 0 measured
```

## Technical Details

The fix recognizes that:
- SSA Phi nodes represent abstract control flow merges
- Actual Cranelift block jumps may differ from SSA structure
- Intermediate merge blocks (from inner control flow) pass their Phi results to outer merges
- The Phi destination register is the correct value to propagate through nested merges

This approach handles arbitrary nesting depths and maintains compatibility with all existing code patterns.

## Files Modified
- `backend/src/cranelift/translator.rs`:
  - Added `block_predecessors` field to `SSATranslator`
  - Updated `new()` to initialize the CFG tracking
  - Modified `Branch` instruction handling to record edges
  - Modified `Jump` instruction handling to record edges
  - Rewrote `collect_branch_args()` to handle indirect control flow

## Verification
All tests pass, including:
- 117 unit/integration tests
- Nested if-then-else (simple and complex)
- Recursive functions with nested conditionals (Ackermann)
- Arbitrary nesting depths (4+ levels)
