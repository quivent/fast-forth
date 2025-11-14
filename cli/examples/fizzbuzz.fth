\ fizzbuzz.fth - The classic interview question
\ Demonstrates loops, conditionals, and modulo arithmetic

: FIZZBUZZ ( n -- )
  \ Prints FizzBuzz sequence from 1 to n
  \ Multiples of 3: Fizz
  \ Multiples of 5: Buzz
  \ Multiples of 15: FizzBuzz
  \ Otherwise: the number
  1+ 1 DO
    I 15 MOD 0= IF
      ." FizzBuzz"
    ELSE
      I 3 MOD 0= IF
        ." Fizz"
      ELSE
        I 5 MOD 0= IF
          ." Buzz"
        ELSE
          I .
        THEN
      THEN
    THEN CR
  LOOP ;

\ Run FizzBuzz for numbers 1-100
." FizzBuzz from 1 to 100:" CR
100 FIZZBUZZ

\ More elegant version using helper words
: DIVISIBLE? ( n divisor -- flag )
  \ Check if n is divisible by divisor
  MOD 0= ;

: PRINT-FIZZBUZZ ( n -- )
  \ Print appropriate FizzBuzz output for number n
  DUP 15 DIVISIBLE? IF
    DROP ." FizzBuzz"
  ELSE DUP 3 DIVISIBLE? IF
    DROP ." Fizz"
  ELSE DUP 5 DIVISIBLE? IF
    DROP ." Buzz"
  ELSE
    .
  THEN THEN THEN ;

: FIZZBUZZ-V2 ( n -- )
  \ Cleaner FizzBuzz implementation
  1+ 1 DO
    I PRINT-FIZZBUZZ CR
  LOOP ;

." " CR
." FizzBuzz V2 (first 20):" CR
20 FIZZBUZZ-V2
