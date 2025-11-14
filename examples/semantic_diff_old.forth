\ Old Implementation - Recursive factorial

: factorial ( n -- n! )
  dup 2 < if
    drop 1
  else
    dup 1- recurse *
  then ;

: double ( n -- 2n )
  2 * ;

: average ( a b -- avg )
  + 2 / ;
