\ FastForth File I/O Demonstration
\ Phase 9 - Full file I/O support with working examples

\ Example 1: Create a file
\ Stack effect: ( -- fileid ior )
: create-demo
  "/tmp/fastforth-demo.txt" w/o create-file
;

\ Example 2: Write to file
\ Stack effect: ( fileid -- ior )
: write-demo
  dup "Hello from FastForth! File I/O is working." rot write-file drop
;

\ Example 3: Close file
\ Stack effect: ( fileid -- ior )
: close-demo
  close-file
;

\ Example 4: Complete file creation and write
: file-write-test
  create-demo          \ Create file
  dup 0= if            \ Check if successful (ior == 0)
    drop               \ Drop ior
    write-demo         \ Write content
    close-demo         \ Close file
  else
    drop drop          \ Clean up on error
  then
;

\ Example 5: System call
: system-demo
  "ls -la /tmp/fastforth-demo.txt" system drop
;

\ Example 6: Delete file
: cleanup-demo
  "/tmp/fastforth-demo.txt" delete-file drop
;

\ Main demo sequence
: run-demo
  file-write-test       \ Create and write file
  system-demo           \ Show file info
  cleanup-demo          \ Clean up
;

\ Uncomment to run:
\ run-demo
