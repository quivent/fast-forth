\ ==================================================
\ Fast Forth Concurrency - Integration Tests
\ ==================================================
\ Tests the concurrency primitives from Forth code
\ ==================================================

\ Test Framework
\ ---------------

variable tests-run
variable tests-passed

: reset-tests ( -- )
  0 tests-run !
  0 tests-passed ! ;

: test-start ( -- )
  1 tests-run +! ;

: test-pass ( -- )
  1 tests-passed +! ;

: assert-eq ( actual expected -- )
  = if test-pass else
    ." FAIL: Values not equal" cr
  then ;

: .test-summary ( -- )
  cr
  ." ========================================" cr
  ." TEST SUMMARY" cr
  ." ========================================" cr
  ." Tests run:    " tests-run @ . cr
  ." Tests passed: " tests-passed @ . cr
  ." Tests failed: " tests-run @ tests-passed @ - . cr
  ." Success rate: "
  tests-passed @ 100 * tests-run @ / . ." %" cr
  ." ========================================" cr ;

\ ==================================================
\ TEST 1: Basic Channel Send/Recv
\ ==================================================

: test-channel-basic ( -- )
  test-start
  ." [TEST] Basic channel send/recv..." cr

  \ Create channel with capacity 10
  10 channel constant test-chan

  \ Send value
  42 test-chan send

  \ Receive value
  test-chan recv

  \ Check value
  42 assert-eq

  \ Cleanup
  test-chan close-channel
  test-chan destroy-channel

  ." ✅ PASS" cr ;

\ ==================================================
\ TEST 2: Multiple Values FIFO Order
\ ==================================================

: test-channel-fifo ( -- )
  test-start
  ." [TEST] Channel FIFO order..." cr

  100 channel constant fifo-chan

  \ Send 10 values
  10 0 do i fifo-chan send loop

  \ Receive and verify order
  10 0 do
    fifo-chan recv
    i assert-eq
  loop

  fifo-chan close-channel
  fifo-chan destroy-channel

  ." ✅ PASS" cr ;

\ ==================================================
\ TEST 3: Simple Thread Spawn and Join
\ ==================================================

variable thread-result

: simple-worker ( -- )
  \ Just store a value
  999 thread-result ! ;

: test-spawn-join ( -- )
  test-start
  ." [TEST] Thread spawn and join..." cr

  0 thread-result !

  \ Spawn thread
  ' simple-worker spawn constant worker-thread

  \ Join (wait for completion)
  worker-thread join

  \ Verify result
  thread-result @
  999 assert-eq

  ." ✅ PASS" cr ;

\ ==================================================
\ TEST 4: Thread Communication via Channel
\ ==================================================

100 channel constant comm-chan

: sender-worker ( -- )
  \ Send 5 values
  5 0 do
    i comm-chan send
  loop ;

: test-thread-communication ( -- )
  test-start
  ." [TEST] Thread communication via channel..." cr

  \ Spawn sender thread
  ' sender-worker spawn constant sender

  \ Receive 5 values
  5 0 do
    comm-chan recv
    i assert-eq
  loop

  \ Wait for sender
  sender join

  ." ✅ PASS" cr ;

\ ==================================================
\ TEST 5: Multi-Agent Pattern (Simplified)
\ ==================================================

100 channel constant work-queue
100 channel constant result-queue

: agent-worker ( agent-id -- )
  \ Get work from queue
  work-queue recv

  \ "Process" (square it)
  dup *

  \ Send result
  result-queue send ;

: test-multi-agent ( -- )
  test-start
  ." [TEST] Multi-agent pattern..." cr

  \ Distribute 10 specs
  10 0 do
    i work-queue send
  loop

  \ Spawn 10 agents
  10 0 do
    i ['] agent-worker spawn drop
  loop

  \ Collect 10 results
  10 0 do
    result-queue recv
    \ Result should be i²
    i dup * assert-eq
  loop

  ." ✅ PASS" cr ;

\ ==================================================
\ TEST 6: Channel Capacity and Blocking
\ ==================================================

: test-channel-capacity ( -- )
  test-start
  ." [TEST] Channel capacity (non-blocking fill)..." cr

  \ Create small channel
  5 channel constant small-chan

  \ Fill channel (should not block)
  5 0 do
    i small-chan send
  loop

  \ Drain channel
  5 0 do
    small-chan recv
    i assert-eq
  loop

  small-chan close-channel
  small-chan destroy-channel

  ." ✅ PASS" cr ;

\ ==================================================
\ TEST 7: Pipeline Pattern (3 stages)
\ ==================================================

100 channel constant stage1-out
100 channel constant stage2-out
100 channel constant final-out

: stage1-worker ( -- )
  \ Get input, add 10, pass to stage 2
  work-queue recv
  10 +
  stage1-out send ;

: stage2-worker ( -- )
  \ Get from stage 1, multiply by 2, pass to stage 3
  stage1-out recv
  2 *
  stage2-out send ;

: stage3-worker ( -- )
  \ Get from stage 2, send final result
  stage2-out recv
  final-out send ;

: test-pipeline ( -- )
  test-start
  ." [TEST] 3-stage pipeline pattern..." cr

  \ Send input
  5 work-queue send

  \ Spawn 3 stages
  ' stage1-worker spawn drop
  ' stage2-worker spawn drop
  ' stage3-worker spawn drop

  \ Collect result
  \ Expected: (5 + 10) * 2 = 30
  final-out recv
  30 assert-eq

  ." ✅ PASS" cr ;

\ ==================================================
\ TEST 8: Stress Test - Many Messages
\ ==================================================

: test-stress ( -- )
  test-start
  ." [TEST] Stress test (1000 messages)..." cr

  1000 channel constant stress-chan

  \ Send 1000 messages
  1000 0 do
    i stress-chan send
  loop

  \ Receive 1000 messages
  1000 0 do
    stress-chan recv
    i assert-eq
  loop

  stress-chan close-channel
  stress-chan destroy-channel

  ." ✅ PASS" cr ;

\ ==================================================
\ RUN ALL TESTS
\ ==================================================

: run-all-tests ( -- )
  reset-tests

  ." " cr
  ." ╔════════════════════════════════════════════════╗" cr
  ." ║  Fast Forth Concurrency - Integration Tests   ║" cr
  ." ╚════════════════════════════════════════════════╝" cr
  ." " cr

  test-channel-basic
  test-channel-fifo
  test-spawn-join
  test-thread-communication
  test-multi-agent
  test-channel-capacity
  test-pipeline
  test-stress

  .test-summary ;

\ Run tests automatically
run-all-tests
