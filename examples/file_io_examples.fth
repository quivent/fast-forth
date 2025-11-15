\ FFI and File I/O Examples for FastForth
\ ANS Forth File Access word set implementation
\ Author: FastForth Team
\ Date: 2025-01-15

\ ============================================================================
\ EXAMPLE 1: Simple File Creation
\ ============================================================================

: simple-create ( -- )
  ." Creating a simple file..." cr
  s" /tmp/fastforth_test.txt" w/o create-file
  if
    drop
    ." Error: Failed to create file!" cr
  else
    ." File created successfully!" cr
    close-file drop
  then
;

\ Usage: simple-create

\ ============================================================================
\ EXAMPLE 2: Write Text to File
\ ============================================================================

: write-hello ( -- )
  ." Writing 'Hello FastForth!' to file..." cr

  \ Create/open file for writing
  s" /tmp/hello.txt" w/o create-file
  if
    drop
    ." Error: Cannot create file" cr
    exit
  then

  \ Save file handle on return stack
  >r

  \ Write string to file
  s" Hello FastForth!" r@ write-file
  if
    ." Error: Write failed" cr
  else
    ." Write successful!" cr
  then

  \ Close file
  r> close-file drop
;

\ Usage: write-hello

\ ============================================================================
\ EXAMPLE 3: Read File Contents
\ ============================================================================

\ Allocate buffer for reading
create read-buffer 256 allot

: read-hello ( -- )
  ." Reading from /tmp/hello.txt..." cr

  \ Open file for reading
  s" /tmp/hello.txt" r/o open-file
  if
    drop
    ." Error: Cannot open file" cr
    exit
  then

  \ Save file handle
  >r

  \ Read from file into buffer
  read-buffer 256 r@ read-file
  if
    drop
    ." Error: Read failed" cr
  else
    ." Read " . ." bytes: "
    read-buffer swap type cr
  then

  \ Close file
  r> close-file drop
;

\ Usage: read-hello

\ ============================================================================
\ EXAMPLE 4: Complete File Lifecycle
\ ============================================================================

: file-lifecycle-demo ( -- )
  ." === File Lifecycle Demo ===" cr

  \ 1. Create and write
  ." Step 1: Creating file..." cr
  s" /tmp/lifecycle.txt" w/o create-file
  if drop ." Failed to create!" cr exit then
  >r

  s" FastForth File I/O Demo" r@ write-file
  if ." Write failed!" cr r> close-file drop exit then

  r> close-file
  if ." Close failed!" cr exit then
  ." File created and written successfully!" cr

  \ 2. Open and read
  ." Step 2: Reading back..." cr
  s" /tmp/lifecycle.txt" r/o open-file
  if drop ." Failed to open!" cr exit then
  >r

  read-buffer 100 r@ read-file
  if drop ." Read failed!" cr r> close-file drop exit then

  ." Read: " read-buffer swap type cr

  r> close-file drop

  \ 3. Delete
  ." Step 3: Deleting file..." cr
  s" /tmp/lifecycle.txt" delete-file
  if ." Delete failed!" cr else ." File deleted!" cr then

  ." === Demo complete ===" cr
;

\ Usage: file-lifecycle-demo

\ ============================================================================
\ EXAMPLE 5: System Call Execution
\ ============================================================================

: run-ls ( -- )
  ." Executing 'ls -la /tmp' command..." cr
  s" ls -la /tmp" system
  ." Command returned with exit code: " . cr
;

\ Usage: run-ls

: check-directory ( -- )
  ." Checking if directory exists..." cr
  s" test -d /tmp && echo 'Directory exists' || echo 'Not found'" system
  drop
;

\ Usage: check-directory

\ ============================================================================
\ EXAMPLE 6: Error Handling
\ ============================================================================

: safe-file-write ( addr u filename-addr filename-u -- flag )
  \ Write string to file with error checking
  \ Returns: true on success, false on error

  w/o create-file                      \ Try to create file
  if
    2drop                              \ Clean stack on error
    ." Error: Cannot create file" cr
    false exit
  then

  >r                                   \ Save file handle
  r@ write-file                        \ Write data
  if
    ." Error: Write failed" cr
    r> close-file drop
    false exit
  then

  r> close-file                        \ Close file
  if
    ." Error: Close failed" cr
    false exit
  then

  true                                 \ Success
;

: demo-safe-write ( -- )
  s" Test data for safe write"
  s" /tmp/safe_write.txt"
  safe-file-write
  if
    ." Safe write succeeded!" cr
  else
    ." Safe write failed!" cr
  then
;

\ Usage: demo-safe-write

\ ============================================================================
\ EXAMPLE 7: File Access Modes
\ ============================================================================

: show-access-modes ( -- )
  ." File Access Modes:" cr
  ." r/o (read-only):  " r/o . cr
  ." w/o (write-only): " w/o . cr
  ." r/w (read-write): " r/w . cr
;

\ Usage: show-access-modes

\ ============================================================================
\ EXAMPLE 8: Append to File
\ ============================================================================

: append-line ( addr u -- )
  \ Append a line to /tmp/append_test.txt

  \ Open file for read-write (to append)
  s" /tmp/append_test.txt" r/w open-file
  if
    \ File doesn't exist, create it
    drop
    s" /tmp/append_test.txt" w/o create-file
    if
      drop 2drop
      ." Error: Cannot create file" cr
      exit
    then
  then

  >r

  \ TODO: Seek to end of file (requires file-position/reposition-file)
  \ For now, just write (will overwrite)

  \ Write the line
  r@ write-file
  if ." Write error!" cr then

  \ Write newline
  s"
" r@ write-file drop

  \ Close file
  r> close-file drop
;

: demo-append ( -- )
  s" First line" append-line
  s" Second line" append-line
  s" Third line" append-line
  ." Lines appended to /tmp/append_test.txt" cr
;

\ Usage: demo-append

\ ============================================================================
\ EXAMPLE 9: Binary File I/O
\ ============================================================================

\ Create a buffer for binary data
create binary-buffer 1024 allot

: write-binary-file ( -- )
  ." Writing binary data..." cr

  \ Fill buffer with binary data (0-255 pattern)
  256 0 do
    i binary-buffer i + c!
  loop

  \ Create binary file
  s" /tmp/binary_data.bin" w/o create-file
  if drop ." Error creating binary file" cr exit then
  >r

  \ Write 256 bytes
  binary-buffer 256 r@ write-file
  if ." Binary write failed!" cr else ." Binary write OK!" cr then

  r> close-file drop
;

: read-binary-file ( -- )
  ." Reading binary data..." cr

  s" /tmp/binary_data.bin" r/o open-file
  if drop ." Cannot open binary file" cr exit then
  >r

  binary-buffer 256 r@ read-file
  if
    drop ." Binary read failed!" cr
  else
    ." Read " . ." bytes of binary data" cr

    \ Display first 16 bytes in hex
    ." First 16 bytes: "
    16 0 do
      binary-buffer i + c@ .
    loop
    cr
  then

  r> close-file drop
;

: demo-binary ( -- )
  write-binary-file
  read-binary-file
;

\ Usage: demo-binary

\ ============================================================================
\ EXAMPLE 10: Large File Operations
\ ============================================================================

: write-large-file ( -- )
  ." Creating 1KB file..." cr

  s" /tmp/large_file.txt" w/o create-file
  if drop ." Error creating file" cr exit then
  >r

  \ Write 1024 bytes (1KB) of 'A' characters
  1024 0 do
    s" A" r@ write-file drop
  loop

  r> close-file drop
  ." 1KB file created successfully!" cr
;

\ Usage: write-large-file

\ ============================================================================
\ EXAMPLE 11: Multiple File Handles
\ ============================================================================

: copy-file ( src-addr src-u dst-addr dst-u -- )
  \ Simple file copy implementation

  \ Open destination for writing
  w/o create-file
  if 2drop 2drop ." Error: Cannot create destination" cr exit then
  >r  \ Save dst handle

  \ Open source for reading
  r/o open-file
  if drop r> close-file drop ." Error: Cannot open source" cr exit then
  >r  \ Save src handle

  \ Read and write loop
  begin
    read-buffer 256 r@ read-file  \ Read from source
    if
      drop 0  \ Error, exit loop
    else
      dup 0= if  \ No more data
        drop 0
      else
        read-buffer swap r'@ write-file  \ Write to destination
        if drop 0 else -1 then  \ Continue if write OK
      then
    then
  until

  \ Close both files
  r> close-file drop  \ Close source
  r> close-file drop  \ Close destination

  ." File copied successfully!" cr
;

: demo-copy ( -- )
  s" /tmp/hello.txt" s" /tmp/hello_copy.txt" copy-file
;

\ Usage: demo-copy

\ ============================================================================
\ IMPLEMENTATION NOTES
\ ============================================================================

\ File Access Modes:
\   r/o = 0 (read-only,  maps to "r")
\   w/o = 1 (write-only, maps to "w")
\   r/w = 2 (read-write, maps to "r+")

\ I/O Result (ior) Convention:
\   0  = Success
\   -1 = Error (file not found, permission denied, etc.)

\ Stack Effects (ANS Forth standard):
\   create-file  ( c-addr u fam -- fileid ior )
\   open-file    ( c-addr u fam -- fileid ior )
\   close-file   ( fileid -- ior )
\   read-file    ( c-addr u fileid -- u ior )
\   write-file   ( c-addr u fileid -- ior )
\   delete-file  ( c-addr u -- ior )
\   system       ( c-addr u -- return-code )

\ Memory Management:
\   - File handles are pointers to FILE* structures
\   - Buffers must be allocated before use (using CREATE or ALLOT)
\   - Always close files to prevent resource leaks

\ String Handling:
\   - Forth strings are (addr len) pairs
\   - C functions expect null-terminated strings
\   - Conversion happens at FFI boundary automatically

\ ============================================================================
\ PERFORMANCE TIPS
\ ============================================================================

\ 1. Buffer Size:
\    - Larger buffers (256-4096 bytes) reduce system call overhead
\    - Smaller buffers use less memory
\    - 256 bytes is a good default for text files

\ 2. File Modes:
\    - Use r/o for read-only access (fastest)
\    - Use w/o for write-only (no read overhead)
\    - Use r/w only when you need both

\ 3. Error Handling:
\    - Always check ior after file operations
\    - Close files even on errors (use exception handling if available)
\    - Test file existence with system() before opening

\ 4. Large Files:
\    - Process in chunks to avoid memory issues
\    - Use file-position and reposition-file for seeking
\    - Consider streaming for very large files (>100MB)

\ ============================================================================
\ FFI EXTENSION EXAMPLES
\ ============================================================================

\ Example of direct FFI call (advanced usage):
\ : call-printf ( addr -- )
\   \ Call libc printf with format string
\   FFICall printf  \ Requires FFI plumbing
\ ;

\ Note: Direct FFI calls require additional SSA support
\ Use file I/O words for standard operations

\ ============================================================================
\ End of Examples
\ ============================================================================
