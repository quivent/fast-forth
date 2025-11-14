\ New Implementation - Tail-recursive factorial (optimized)

: factorial ( n -- n! )
  1 swap factorial-iter ;

: factorial-iter ( acc n -- result )
  dup 2 < if
    drop
  else
    dup rot * swap 1- factorial-iter
  then ;

: double ( n -- 2n )
  dup + ;

: average ( a b -- avg )
  + 2 / ;
