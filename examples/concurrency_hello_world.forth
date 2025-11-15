\ ==================================================
\ Fast Forth Concurrency - Hello World
\ ==================================================
\ Minimal example demonstrating concurrency primitives
\ ==================================================

\ Simple example: One thread sends, another receives
\ -----------------------------------------------------

: hello-world-concurrency ( -- )
  ." Fast Forth Concurrency Hello World" cr
  cr

  \ Create a channel with capacity 1
  1 channel constant hello-chan

  \ Define a simple sender thread
  : sender-thread ( -- )
    ." [SENDER] Sending message..." cr
    42 hello-chan send
    ." [SENDER] Message sent: 42" cr ;

  \ Spawn the sender thread
  ." Spawning sender thread..." cr
  ' sender-thread spawn constant sender

  \ Receive the message (main thread)
  ." [MAIN] Waiting for message..." cr
  hello-chan recv
  ." [MAIN] Received message: " . cr

  \ Wait for sender to complete
  ." Waiting for sender thread to finish..." cr
  sender join
  ." Sender thread completed" cr

  \ Cleanup
  hello-chan close-channel
  hello-chan destroy-channel

  cr
  ." ✅ Concurrency test complete!" cr ;

\ Run the test
hello-world-concurrency

\ ==================================================
\ Expected Output:
\ ==================================================
\ Fast Forth Concurrency Hello World
\
\ Spawning sender thread...
\ [SENDER] Sending message...
\ [MAIN] Waiting for message...
\ [SENDER] Message sent: 42
\ [MAIN] Received message: 42
\ Waiting for sender thread to finish...
\ Sender thread completed
\
\ ✅ Concurrency test complete!
\ ==================================================
