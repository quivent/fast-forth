\ Comprehensive File I/O Test

\ Test 1: Create and write to file
: test-write
  \ Create file
  "/tmp/fastforth-io-test.txt" w/o create-file
  \ Check for errors
  dup 0= if
    drop
    dup "Hello from FastForth!" rot write-file drop
    close-file drop
    1  \ Success
  else
    \ Error - clean up stack
    drop drop
    0  \ Failure
  then
;

\ Test 2: Read from file
: test-read
  "/tmp/fastforth-io-test.txt" r/o open-file
  dup 0= if
    drop
    \ File handle is on stack
    \ TODO: Allocate buffer and read
    close-file drop
    1  \ Success
  else
    drop drop
    0  \ Failure
  then
;

\ Test 3: Delete file
: test-delete
  "/tmp/fastforth-io-test.txt" delete-file
  0=  \ 0 = success
;

\ Run test-write
test-write
