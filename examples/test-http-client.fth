\ Test script for FastForth HTTP client and Ollama integration
\ Run this to verify the HTTP client is working

.( ============================================ ) cr
.( FastForth HTTP Client Test Suite ) cr
.( ============================================ ) cr cr

\ Load the Ollama client (which includes HTTP client and strings)
include examples/ollama-client.fth

.( Test 1: String manipulation functions ) cr
.( ------------------------------------- ) cr

\ Test cmove
create test-buffer 64 allot
s" Hello, World!" test-buffer 20 cmove
.( cmove test: ) test-buffer 13 type cr

\ Test place
create counted-buffer 64 allot
s" Counted string" counted-buffer place
.( place test: ) counted-buffer count type cr cr

.( Test 2: URL parsing ) cr
.( ------------------- ) cr

.( Parsing: http://localhost:11434 ) cr
s" http://localhost:11434" parse-url
.( Port: ) . cr
.( Host length: ) . cr
.( Host: ) type cr cr

.( Test 3: JSON building ) cr
.( --------------------- ) cr

.( Building JSON request... ) cr
s" Test prompt" s" llama3.2" build-ollama-json
.( JSON: ) json-buffer json-len @ type cr cr

.( Test 4: HTTP POST (requires Ollama server) ) cr
.( ------------------------------------------- ) cr

.( Attempting to connect to Ollama... ) cr
s" Say hello in one word" s" llama3.2" query-ollama if
  .( Success! Response: ) type cr
else
  2drop .( Failed to connect to Ollama server ) cr
  .( Make sure Ollama is running at: ) ollama-url ollama-url-len @ type cr
then

cr
.( ============================================ ) cr
.( Test suite complete ) cr
.( ============================================ ) cr
