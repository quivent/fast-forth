\ ANS Forth Core Word Set
\ Stream 6: Standard library implemented in Forth
\
\ This file defines ANS Forth standard words using primitives.
\ These will be compiled by Fast Forth and linked into the runtime.

\ ============================================================================
\ STACK MANIPULATION (Extended)
\ ============================================================================

: ?DUP  ( n -- 0 | n n )    \ Duplicate if non-zero
    DUP IF DUP THEN ;

: DEPTH  ( -- n )           \ Return stack depth
    SP@ S0 @ - CELL / ;

\ ============================================================================
\ ARITHMETIC (Extended)
\ ============================================================================

: 1+  ( n -- n+1 )  1 + ;
: 1-  ( n -- n-1 )  1 - ;
: 2+  ( n -- n+2 )  2 + ;
: 2-  ( n -- n-2 )  2 - ;
: 2*  ( n -- n*2 )  1 LSHIFT ;
: 2/  ( n -- n/2 )  1 RSHIFT ;

: NEGATE  ( n -- -n )  0 SWAP - ;
: ABS     ( n -- |n| )  DUP 0< IF NEGATE THEN ;

: */  ( a b c -- a*b/c )
    >R * R> / ;

: */MOD  ( a b c -- rem quot )
    >R * R> /MOD ;

: SM/REM  ( d n -- rem quot )  \ Symmetric division
    2DUP XOR >R              \ Save sign of quotient
    OVER >R                  \ Save sign of remainder
    ABS >R DABS R> /         \ Unsigned division
    SWAP R> 0< IF NEGATE THEN  \ Apply remainder sign
    SWAP R> 0< IF NEGATE THEN  \ Apply quotient sign
;

: FM/MOD  ( d n -- rem quot )  \ Floored division
    DUP >R
    SM/REM
    DUP 0< IF
        SWAP R> +
        SWAP 1-
    ELSE
        R> DROP
    THEN
;

\ ============================================================================
\ DOUBLE-CELL ARITHMETIC
\ ============================================================================

: D+  ( d1 d2 -- d3 )       \ Double-cell addition
    >R SWAP >R
    + DUP R> + SWAP
    OVER U< IF 1+ THEN
    R> + ;

: D-  ( d1 d2 -- d3 )       \ Double-cell subtraction
    DNEGATE D+ ;

: DNEGATE  ( d -- -d )      \ Negate double-cell
    SWAP NEGATE SWAP NEGATE
    OVER 0= IF 1+ THEN ;

: DABS  ( d -- |d| )        \ Absolute value double
    DUP 0< IF DNEGATE THEN ;

: D2*  ( d -- d*2 )         \ Double-cell left shift
    2DUP D+ ;

: D2/  ( d -- d/2 )         \ Double-cell right shift
    DUP 1 AND >R            \ Save low bit
    2 / SWAP 2 / SWAP       \ Shift both cells
    R> IF $8000000000000000 OR THEN ;  \ Restore sign

\ ============================================================================
\ LOGIC (Extended)
\ ============================================================================

: TRUE   -1 ;
: FALSE   0 ;

: NOT  ( x -- flag )  0= ;

\ ============================================================================
\ COMPARISON (Extended)
\ ============================================================================

: WITHIN  ( n lo hi -- flag )  \ n in [lo, hi)?
    OVER - >R - R> U< ;

: 0<>  ( n -- flag )  0= NOT ;
: U<   ( u1 u2 -- flag )  2DUP XOR 0< IF DROP 0< ELSE - 0< THEN ;
: U>   ( u1 u2 -- flag )  SWAP U< ;
: U<=  ( u1 u2 -- flag )  U> NOT ;
: U>=  ( u1 u2 -- flag )  U< NOT ;

: D=   ( d1 d2 -- flag )  ROT = >R = R> AND ;
: D<   ( d1 d2 -- flag )
    ROT 2DUP = IF
        2DROP <
    ELSE
        > IF DROP DROP FALSE ELSE 2DROP TRUE THEN
    THEN ;

: D0=  ( d -- flag )  OR 0= ;
: D0<  ( d -- flag )  NIP 0< ;

\ ============================================================================
\ MEMORY (Extended)
\ ============================================================================

: CELL   8 ;                \ Cell size in bytes (64-bit)
: CELLS  ( n -- n*cell )  3 LSHIFT ;  \ Multiply by 8
: CELL+  ( addr -- addr+cell )  CELL + ;

: CHAR+  ( addr -- addr+1 )  1+ ;
: CHARS  ( n -- n )  ;      \ Characters are bytes

: ALIGN  ( -- )             \ Align HERE to cell boundary
    HERE 7 AND IF
        8 HERE 7 AND - ALLOT
    THEN ;

: ALIGNED  ( addr -- aligned )
    DUP 7 AND IF
        8 SWAP 7 AND - +
    THEN ;

: COUNT  ( c-addr -- addr len )  \ Get counted string
    DUP 1+ SWAP C@ ;

: MOVE  ( src dest count -- )  \ Move memory
    >R
    BEGIN
        R@ 0>
    WHILE
        OVER C@ OVER C!
        1+ SWAP 1+ SWAP
        R> 1- >R
    REPEAT
    R> DROP 2DROP ;

: FILL  ( addr count char -- )  \ Fill memory
    SWAP >R SWAP
    BEGIN
        R@ 0>
    WHILE
        2DUP C!
        1+
        R> 1- >R
    REPEAT
    R> DROP 2DROP ;

: ERASE  ( addr count -- )  \ Fill with zeros
    0 FILL ;

\ ============================================================================
\ STRING OPERATIONS
\ ============================================================================

: COMPARE  ( addr1 len1 addr2 len2 -- n )
    ROT 2DUP - >R            \ Save length difference
    MIN >R                   \ Minimum length
    BEGIN
        R@ 0>
    WHILE
        OVER C@ OVER C@ - ?DUP IF
            NIP NIP R> DROP R> DROP EXIT
        THEN
        1+ SWAP 1+ SWAP
        R> 1- >R
    REPEAT
    R> DROP R> ;

: SEARCH  ( addr1 len1 addr2 len2 -- addr3 len3 flag )
    2>R 2DUP
    BEGIN
        DUP R@ >=
    WHILE
        OVER R@ 2R@ COMPARE 0= IF
            2R> 2DROP TRUE EXIT
        THEN
        1- SWAP 1+ SWAP
    REPEAT
    2DROP 2R> 2DROP 0 0 FALSE ;

\ ============================================================================
\ I/O (Extended)
\ ============================================================================

: ."  ( -- )                \ Print string (compile time)
    POSTPONE S"
    POSTPONE TYPE
; IMMEDIATE

: .( ( -- )                 \ Print string (immediate)
    [CHAR] ) PARSE TYPE
; IMMEDIATE

: EMIT  ( char -- )         \ Already primitive
: CR    ( -- )              \ Already primitive
: SPACE ( -- )              \ Already primitive
: SPACES ( n -- )           \ Already primitive

: .  ( n -- )               \ Print number
    DUP ABS 0 <# #S ROT SIGN #> TYPE SPACE ;

: U.  ( u -- )              \ Print unsigned
    0 <# #S #> TYPE SPACE ;

: .R  ( n width -- )        \ Print right-justified
    >R DUP ABS 0 <# #S ROT SIGN #>
    R> OVER - SPACES TYPE ;

: U.R  ( u width -- )       \ Print unsigned right-justified
    >R 0 <# #S #>
    R> OVER - SPACES TYPE ;

\ ============================================================================
\ NUMBER CONVERSION
\ ============================================================================

VARIABLE BASE  10 BASE !

: DECIMAL  10 BASE ! ;
: HEX      16 BASE ! ;
: BINARY    2 BASE ! ;

: <#  ( -- )  PAD HLD ! ;
: HOLD  ( char -- )  HLD @ 1- DUP HLD ! C! ;
: #  ( d -- d )
    BASE @ UD/MOD ROT
    DUP 9 > IF 7 + THEN
    [CHAR] 0 + HOLD ;
: #S  ( d -- 0 0 )  BEGIN # 2DUP D0= UNTIL ;
: #>  ( d -- addr len )  2DROP HLD @ PAD OVER - ;
: SIGN  ( n -- )  0< IF [CHAR] - HOLD THEN ;

\ ============================================================================
\ CONTROL STRUCTURES (Implementation)
\ ============================================================================

: IF  ( -- orig )           \ Compile-time if
    POSTPONE 0BRANCH
    HERE 0 ,
; IMMEDIATE

: THEN  ( orig -- )         \ Compile-time then
    HERE SWAP !
; IMMEDIATE

: ELSE  ( orig1 -- orig2 )  \ Compile-time else
    POSTPONE BRANCH
    HERE 0 ,
    SWAP
    HERE SWAP !
; IMMEDIATE

: BEGIN  ( -- dest )        \ Compile-time begin
    HERE
; IMMEDIATE

: UNTIL  ( dest -- )        \ Compile-time until
    POSTPONE 0BRANCH
    ,
; IMMEDIATE

: WHILE  ( dest -- orig dest )
    POSTPONE IF
    SWAP
; IMMEDIATE

: REPEAT  ( orig dest -- )
    POSTPONE BRANCH
    ,
    POSTPONE THEN
; IMMEDIATE

: DO  ( -- do-sys )         \ Compile-time do
    POSTPONE 2>R
    HERE
; IMMEDIATE

: LOOP  ( do-sys -- )       \ Compile-time loop
    POSTPONE (LOOP)
    ,
; IMMEDIATE

: +LOOP  ( do-sys -- )      \ Compile-time +loop
    POSTPONE (+LOOP)
    ,
; IMMEDIATE

: I  ( -- n )               \ Loop index
    R> R> TUCK >R >R ;

: J  ( -- n )               \ Outer loop index
    R> R> R> R> 2DUP >R >R ROT >R ROT >R ;

\ ============================================================================
\ CASE STATEMENT (ANS Forth extension)
\ ============================================================================

: CASE  ( -- case-sys )
    0                       \ Mark case start
; IMMEDIATE

: OF  ( x -- | x )
    POSTPONE OVER
    POSTPONE =
    POSTPONE IF
    POSTPONE DROP
; IMMEDIATE

: ENDOF  ( -- )
    POSTPONE ELSE
; IMMEDIATE

: ENDCASE  ( case-sys -- )
    POSTPONE DROP
    BEGIN
        ?DUP
    WHILE
        POSTPONE THEN
    REPEAT
; IMMEDIATE

\ ============================================================================
\ WORD DEFINITION UTILITIES
\ ============================================================================

: CONSTANT  ( n -- )        \ Create constant
    CREATE , DOES> @ ;

: VARIABLE  ( -- )          \ Create variable
    CREATE 0 , ;

: VALUE  ( n -- )           \ Create value
    CREATE , DOES> @ ;

: TO  ( n -- )              \ Modify value
    ' >BODY ! ; IMMEDIATE

: 2CONSTANT  ( d -- )       \ Double constant
    CREATE , , DOES> 2@ ;

: 2VARIABLE  ( -- )         \ Double variable
    CREATE 0 , 0 , ;

\ ============================================================================
\ ARRAYS AND BUFFERS
\ ============================================================================

: BUFFER:  ( n -- )         \ Create buffer
    CREATE ALLOT ;

: ARRAY  ( n -- )           \ Create array
    CREATE CELLS ALLOT DOES> SWAP CELLS + ;

\ ============================================================================
\ EXCEPTION HANDLING (Basic)
\ ============================================================================

VARIABLE HANDLER  0 HANDLER !

: CATCH  ( xt -- exception# | 0 )
    SP@ >R                  \ Save data stack pointer
    HANDLER @ >R            \ Save previous handler
    RP@ HANDLER !           \ Set current handler
    EXECUTE                 \ Execute xt
    R> HANDLER !            \ Restore handler
    R> DROP                 \ Drop saved stack pointer
    0                       \ No exception
;

: THROW  ( exception# -- )
    ?DUP IF                 \ If exception
        HANDLER @ RP!       \ Restore return stack
        R> HANDLER !        \ Restore previous handler
        R> SWAP >R          \ Get saved stack pointer
        SP! DROP R>         \ Restore data stack
    THEN ;

\ ============================================================================
\ FILE ACCESS (Stub - OS-dependent)
\ ============================================================================

: OPEN-FILE  ( addr len mode -- fileid ior ) ;
: CLOSE-FILE  ( fileid -- ior ) ;
: READ-FILE  ( addr len fileid -- len ior ) ;
: WRITE-FILE  ( addr len fileid -- ior ) ;

\ ============================================================================
\ UTILITY WORDS
\ ============================================================================

: WORDS  ( -- )             \ List all words
    CR
    LATEST
    BEGIN
        ?DUP
    WHILE
        DUP .NAME SPACE
        @
    REPEAT
    CR ;

: SEE  ( "word" -- )        \ Decompile word
    ' >BODY                 \ Get word body
    CR ." : " DUP .NAME SPACE
    \ Decompilation logic here
    ." ;" CR ;

: DUMP  ( addr len -- )     \ Memory dump
    BASE @ >R HEX
    16 / 1+ 0 DO
        CR DUP 8 U.R SPACE SPACE
        DUP 16 0 DO
            DUP C@ 2 U.R SPACE
            1+
        LOOP
        DROP
        16 +
    LOOP
    DROP
    R> BASE ! ;

: ?  ( addr -- )            \ Print value at address
    @ . ;

\ ============================================================================
\ BENCHMARK AND PERFORMANCE
\ ============================================================================

: MS  ( n -- )              \ Millisecond delay (OS-dependent)
    1000000 * USLEEP ;      \ Approximate

: BENCHMARK  ( xt count -- )  \ Benchmark execution
    SWAP >R
    UTIME >R
    0 DO R@ EXECUTE LOOP
    UTIME R> - 1000 /       \ Convert to milliseconds
    CR ." Elapsed: " . ." ms"
    R> DROP ;

\ ============================================================================
\ DEBUGGING UTILITIES
\ ============================================================================

: .S  ( -- )                \ Print stack without consuming
    CR ." <" DEPTH 0 .R ." > "
    DEPTH 0> IF
        DEPTH 0 DO
            DEPTH I - 1- PICK .
        LOOP
    THEN ;

: TRACE  ( -- )             \ Enable tracing (implementation-specific)
    ;

: NOTRACE  ( -- )           \ Disable tracing
    ;

\ ============================================================================
\ ENVIRONMENT QUERIES (ANS Forth required)
\ ============================================================================

: ENVIRONMENT?  ( addr len -- false | value true )
    2DUP S" /COUNTED-STRING" COMPARE 0= IF
        2DROP 255 TRUE EXIT
    THEN
    2DUP S" /HOLD" COMPARE 0= IF
        2DROP 256 TRUE EXIT
    THEN
    2DUP S" /PAD" COMPARE 0= IF
        2DROP 1024 TRUE EXIT
    THEN
    2DUP S" ADDRESS-UNIT-BITS" COMPARE 0= IF
        2DROP 8 TRUE EXIT
    THEN
    2DUP S" MAX-CHAR" COMPARE 0= IF
        2DROP 255 TRUE EXIT
    THEN
    2DUP S" MAX-N" COMPARE 0= IF
        2DROP $7FFFFFFFFFFFFFFF TRUE EXIT
    THEN
    2DUP S" MAX-U" COMPARE 0= IF
        2DROP $FFFFFFFFFFFFFFFF TRUE EXIT
    THEN
    2DROP FALSE ;

\ ============================================================================
\ INITIALIZATION
\ ============================================================================

: COLD  ( -- )              \ Cold start
    DECIMAL
    CR ." Fast Forth v1.0"
    CR ." Type WORDS to see available words"
    CR ;

\ Auto-run on startup
COLD
