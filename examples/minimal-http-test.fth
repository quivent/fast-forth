\ Minimal HTTP Client Test
\ Simple test to verify HTTP functionality without full Ollama

\ Load libraries
include lib/strings.fth
include lib/http-client.fth

.( ============================================ ) cr
.( Minimal HTTP Client Test ) cr
.( ============================================ ) cr cr

\ Test 1: String operations
.( Test 1: String functions ) cr
create test-buf 100 allot
s" Hello" test-buf 10 cmove
.( cmove: ) test-buf 5 type cr

s" World" test-buf place
.( place: ) test-buf count type cr cr

\ Test 2: URL parsing
.( Test 2: URL parsing ) cr
s" http://localhost:8080" parse-url
.( Port: ) . cr
.( Host len: ) . cr
.( Host: ) type cr cr

\ Test 3: Build simple JSON
.( Test 3: JSON building ) cr
http-clear
[char] { http-c-append
s\" "test":"value" http-append
[char] } http-c-append
.( JSON: ) http-buffer http-len @ type cr cr

\ Test 4: File operations
.( Test 4: File I/O ) cr
s" test data" s" /tmp/ff-test.txt" write-string-to-file if
  .( Write OK ) cr
  s" /tmp/ff-test.txt" http-buffer 100 read-file-to-buffer if
    .( Read: ) http-buffer swap type cr
  else
    drop .( Read failed ) cr
  then
else
  .( Write failed ) cr
then
s" /tmp/ff-test.txt" delete-file drop

cr
.( ============================================ ) cr
.( All basic tests complete ) cr
.( ============================================ ) cr
