\ sorting.fth - Sorting algorithm implementations
\ Demonstrates arrays, loops, and algorithm design

\ Note: This is pseudocode showing what the syntax would look like
\ Actual array support would need to be implemented in the compiler

\ Bubble Sort (simple, O(n²))
: BUBBLE-SORT ( array n -- )
  \ Sort array of n elements using bubble sort
  \ array is a pointer to the start of the array
  1 DO
    DUP I 1 DO
      \ Compare adjacent elements
      DUP J CELLS + @
      OVER J 1+ CELLS + @
      > IF
        \ Swap if out of order
        DUP J CELLS + @
        OVER J 1+ CELLS + @
        OVER J CELLS + !
        SWAP J 1+ CELLS + !
      ELSE
        DROP
      THEN
    LOOP
  LOOP DROP ;

\ Quick Sort (fast, O(n log n) average)
: PARTITION ( array low high -- pivot-index )
  \ Partition array for quicksort
  \ Implementation details omitted for brevity
  ;

: QUICK-SORT ( array low high -- )
  \ Sort array[low..high] using quicksort
  2DUP >= IF 2DROP EXIT THEN

  3DUP PARTITION

  \ Sort left partition
  2DUP 2>R
  1 - RECURSE

  \ Sort right partition
  2R> 1 + RECURSE ;

\ Example usage (pseudocode):
\ CREATE my-array 5 2 8 1 9 ,
\ my-array 5 BUBBLE-SORT
\ my-array 5 PRINT-ARRAY

." Sorting algorithms defined!" CR
." Note: Requires array support in Fast Forth" CR
