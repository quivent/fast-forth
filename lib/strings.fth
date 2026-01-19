\ String Manipulation Library for FastForth
\ Provides basic string operations needed for HTTP client

\ ============================================================================
\ BASIC STRING OPERATIONS
\ ============================================================================

\ Store counted string (Pascal-style string with length byte)
\ Format: length-byte followed by characters
: place ( addr len dest -- )
  over over c!     \ Store length at dest
  1+ swap cmove    \ Copy string after length byte
;

\ Copy memory from source to destination
: cmove ( src dest len -- )
  0 ?do
    over c@ over c!   \ Copy one byte
    1+ swap 1+        \ Advance both pointers
  loop
  2drop               \ Clean up stack
;

\ Get counted string as addr/len pair
: count ( c-addr -- addr len )
  dup 1+ swap c@
;

\ ============================================================================
\ STRING SCANNING AND SEARCHING
\ ============================================================================

\ Scan for character in string, return remainder after character
\ Returns 0 0 if not found, or remaining string if found
: scan ( addr len char -- addr' len' )
  >r                  \ Save character to return stack
  begin
    dup 0> while      \ While length > 0
    over c@ r@ = if   \ If current char matches
      r> drop exit    \ Return remaining string
    then
    1 /string         \ Advance to next character
  repeat
  r> drop             \ Clean up return stack
;

\ Search for substring in string
\ Returns found string and true, or original string and false
: search ( addr1 len1 addr2 len2 -- addr3 len3 flag )
  dup >r              \ Save search string length
  >r                  \ Save search string address
  begin
    dup r@ >= while   \ While enough characters remain
    2dup              \ Duplicate haystack addr/len
    r@ r@ compare 0= if  \ If strings match
      r> r> 2drop     \ Clean up return stack
      true exit       \ Return found position and true
    then
    1 /string         \ Advance haystack by one character
  repeat
  r> r> 2drop         \ Clean up return stack
  false               \ Not found
;

\ Compare two strings (returns 0 if equal, -1 if s1<s2, 1 if s1>s2)
: compare ( addr1 len1 addr2 len2 -- n )
  rot                 \ addr1 addr2 len1 len2
  2dup = if           \ If lengths equal
    drop              \ addr1 addr2 len1
    0 ?do             \ Compare each character
      over c@ over c@ - dup if
        >r 2drop r> unloop exit
      then
      drop 1+ swap 1+
    loop
    2drop 0           \ Strings are equal
  else                \ Lengths differ
    > if 1 else -1 then
    >r 2drop 2drop r>
  then
;

\ Advance string pointer (skip n characters)
: /string ( addr len n -- addr+n len-n )
  dup >r            \ Save n
  - swap r> +       \ Subtract n from len, add n to addr
  swap
;

\ Check if string starts with prefix
: string-prefix? ( str-addr str-len prefix-addr prefix-len -- flag )
  rot               \ str-addr prefix-addr str-len prefix-len
  2dup > if         \ If prefix longer than string
    2drop 2drop false exit
  then
  >r swap r@        \ str-addr prefix-addr str-len prefix-len
  compare 0=        \ Compare first prefix-len characters
  r> drop
;

\ ============================================================================
\ STRING BUFFER OPERATIONS
\ ============================================================================

\ Blank (fill with spaces)
: blank ( addr len -- )
  bl fill
;

\ Fill with character
: fill ( addr len char -- )
  -rot 0 ?do
    2dup c!
    1+
  loop
  2drop
;

\ ============================================================================
\ NUMBER TO STRING CONVERSION
\ ============================================================================

\ Number formatting using pictured numeric output
: n>string ( n -- addr len )
  dup >r          \ Save number
  abs             \ Get absolute value
  0 <# #s         \ Convert to string
  r> 0< if        \ If negative
    [char] - hold \ Add minus sign
  then
  #>              \ Get string
;

\ ============================================================================
\ STRING CONSTRUCTION HELPERS
\ ============================================================================

\ Append character to buffer at offset
: c+append ( char addr offset -- offset' )
  2dup + c!          \ Store char at addr+offset
  1+                 \ Increment offset
;

\ Append string to buffer at offset
: s+append ( src-addr src-len dest-addr offset -- offset' )
  >r                 \ Save offset
  r@ + swap cmove    \ Copy string to dest+offset
  r> +               \ Add string length to offset
;

.( String manipulation library loaded ) cr
