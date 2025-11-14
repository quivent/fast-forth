\ Simple math examples

: double ( n -- 2n )
  2 *
;

: square ( n -- n^2 )
  dup *
;

: cube ( n -- n^3 )
  dup dup * *
;

\ Test double
5 double .

\ Test square
7 square .

\ Test cube
3 cube .
