\ ==================================================
\ Fast Forth Concurrency - Performance Benchmarks
\ ==================================================
\ Measures latency and throughput of concurrency primitives
\ ==================================================

\ Benchmarking Utilities
\ -----------------------

variable bench-start-time
variable bench-end-time

: bench-start ( -- )
  time bench-start-time ! ;

: bench-end ( -- elapsed-ms )
  time bench-end-time !
  bench-end-time @ bench-start-time @ - ;

: .bench-result ( elapsed-ms count name -- )
  cr
  ." Benchmark: " type cr
  ." Total time: " over . ." ms" cr
  ." Operations: " dup . cr
  ." Average:    " swap over / . ." ms/op" cr
  ." Throughput: " 1000 * swap / . ." ops/sec" cr ;

\ ==================================================
\ BENCHMARK 1: Channel Send/Recv Latency
\ ==================================================

: bench-channel-latency ( -- )
  ." " cr
  ." [BENCH] Channel send/recv latency..." cr

  1000 channel constant bench-chan

  bench-start

  \ 10,000 send/recv pairs
  10000 0 do
    i bench-chan send
    bench-chan recv drop
  loop

  bench-end
  10000 s" channel send/recv" .bench-result

  bench-chan close-channel
  bench-chan destroy-channel ;

\ ==================================================
\ BENCHMARK 2: Thread Spawn Latency
\ ==================================================

: dummy-worker ( -- )
  \ Minimal worker - just returns
  ;

: bench-spawn-latency ( -- )
  ." " cr
  ." [BENCH] Thread spawn latency..." cr

  bench-start

  \ Spawn 100 threads
  100 0 do
    ' dummy-worker spawn
  loop

  bench-end

  \ Note: threads still need to be joined
  \ This measures just spawn time

  100 s" thread spawn" .bench-result ;

\ ==================================================
\ BENCHMARK 3: Multi-Agent Throughput
\ ==================================================

100 channel constant work-bench
100 channel constant result-bench

: bench-agent-worker ( -- )
  begin
    work-bench recv
    dup 0= if drop exit then  \ Sentinel
    \ "Process" the work
    dup *
    result-bench send
  again ;

: bench-multi-agent-throughput ( -- )
  ." " cr
  ." [BENCH] Multi-agent throughput (10 agents, 1000 specs)..." cr

  \ Start 10 agents
  10 0 do
    ' bench-agent-worker spawn drop
  loop

  bench-start

  \ Send 1000 specs
  1000 0 do
    i work-bench send
  loop

  \ Collect 1000 results
  1000 0 do
    result-bench recv drop
  loop

  bench-end

  \ Send shutdown sentinels
  10 0 do
    0 work-bench send
  loop

  1000 s" multi-agent processing" .bench-result ;

\ ==================================================
\ BENCHMARK 4: Pipeline Throughput
\ ==================================================

100 channel constant pipe1
100 channel constant pipe2
100 channel constant pipe3

: pipe-stage1 ( -- )
  begin
    work-bench recv
    dup 0= if drop exit then
    10 +
    pipe1 send
  again ;

: pipe-stage2 ( -- )
  begin
    pipe1 recv
    dup 0= if drop exit then
    2 *
    pipe2 send
  again ;

: pipe-stage3 ( -- )
  begin
    pipe2 recv
    dup 0= if drop exit then
    result-bench send
  again ;

: bench-pipeline-throughput ( -- )
  ." " cr
  ." [BENCH] 3-stage pipeline throughput (1000 items)..." cr

  \ Start pipeline stages
  ' pipe-stage1 spawn drop
  ' pipe-stage2 spawn drop
  ' pipe-stage3 spawn drop

  bench-start

  \ Send 1000 items
  1000 0 do
    i work-bench send
  loop

  \ Collect 1000 results
  1000 0 do
    result-bench recv drop
  loop

  bench-end

  \ Shutdown pipeline
  0 work-bench send
  0 pipe1 send
  0 pipe2 send

  1000 s" pipeline processing" .bench-result ;

\ ==================================================
\ BENCHMARK 5: Channel Contention (Multiple Writers)
\ ==================================================

100 channel constant contention-chan

: contention-writer ( id -- )
  \ Each writer sends 100 messages
  100 0 do
    dup contention-chan send
  loop
  drop ;

: bench-channel-contention ( -- )
  ." " cr
  ." [BENCH] Channel contention (10 writers, 100 msgs each)..." cr

  \ Spawn 10 concurrent writers
  10 0 do
    i ['] contention-writer spawn drop
  loop

  bench-start

  \ Collect 1000 messages (10 × 100)
  1000 0 do
    contention-chan recv drop
  loop

  bench-end

  1000 s" concurrent channel writes" .bench-result ;

\ ==================================================
\ BENCHMARK 6: Memory Overhead
\ ==================================================

: bench-memory-overhead ( -- )
  ." " cr
  ." [BENCH] Memory overhead estimation..." cr
  ." " cr

  ." Channel (capacity 100):" cr
  ." - Ring buffer: " 100 8 * . ." bytes" cr
  ." - Metadata:    40 bytes" cr
  ." - Total:       " 100 8 * 40 + . ." bytes" cr
  ." " cr

  ." Thread (pthread):" cr
  ." - Stack:       8 KB" cr
  ." - Forth VM:    4 KB" cr
  ." - Total:       ~12 KB per thread" cr
  ." " cr

  ." 10 agents + 2 channels (100 cap):" cr
  ." - Threads:     " 10 12 * . ." KB" cr
  ." - Channels:    " 2 840 * . ." bytes" cr
  ." - Total:       ~" 10 12 * . ." KB" cr ;

\ ==================================================
\ BENCHMARK 7: Scalability Test
\ ==================================================

: scalability-worker ( -- )
  100 0 do
    work-bench recv
    dup *
    result-bench send
  loop ;

: bench-scalability ( agents -- )
  ." " cr
  ." [BENCH] Scalability with " dup . ." agents..." cr

  \ Spawn N agents
  dup 0 do
    ' scalability-worker spawn drop
  loop

  \ Each agent processes 100 items = N × 100 total
  dup 100 * constant total-work

  bench-start

  \ Send work
  total-work 0 do
    i work-bench send
  loop

  \ Collect results
  total-work 0 do
    result-bench recv drop
  loop

  bench-end

  total-work s" scalability test" .bench-result ;

\ ==================================================
\ RUN ALL BENCHMARKS
\ ==================================================

: run-all-benchmarks ( -- )
  ." " cr
  ." ╔════════════════════════════════════════════════╗" cr
  ." ║  Fast Forth Concurrency - Performance Bench   ║" cr
  ." ╚════════════════════════════════════════════════╝" cr

  bench-channel-latency
  bench-spawn-latency
  bench-multi-agent-throughput
  bench-pipeline-throughput
  bench-channel-contention
  bench-memory-overhead

  \ Scalability tests
  1 bench-scalability
  5 bench-scalability
  10 bench-scalability
  20 bench-scalability

  ." " cr
  ." ╔════════════════════════════════════════════════╗" cr
  ." ║  Benchmarks Complete                           ║" cr
  ." ╚════════════════════════════════════════════════╝" cr ;

\ Run benchmarks automatically
run-all-benchmarks
