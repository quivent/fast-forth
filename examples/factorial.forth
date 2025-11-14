\ Factorial calculator
\ Calculate n!

: factorial ( n -- n! )
  dup 1 <=
  if
    drop 1
  else
    dup 1- factorial *
  then
;

\ Calculate 5!
5 factorial .
