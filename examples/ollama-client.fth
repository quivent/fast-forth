\ Ollama Client for FastForth
\ Provides interface to Ollama API for AI model interaction
\ Requires: lib/http-client.fth (which includes lib/strings.fth)

\ Load HTTP client library
include lib/http-client.fth

\ ============================================================================
\ CONFIGURATION
\ ============================================================================

\ Ollama server configuration
create ollama-url 256 allot
variable ollama-url-len

\ Initialize with default Ollama URL
: init-ollama-url
  s" http://192.222.57.162:11434" ollama-url ollama-url-len !
  ollama-url swap cmove
;

init-ollama-url

\ Set custom Ollama URL
: set-ollama-url ( addr len -- )
  dup ollama-url-len !
  ollama-url swap cmove
;

\ ============================================================================
\ JSON REQUEST BUILDING
\ ============================================================================

\ JSON buffer for building requests
create json-buffer 4096 allot
variable json-len

\ Clear JSON buffer
: json-clear ( -- )
  0 json-len !
  json-buffer 4096 blank
;

\ Append string to JSON buffer
: json-append ( addr len -- )
  json-buffer json-len @ + swap cmove
  json-len +!
;

\ Append character to JSON buffer
: json-c-append ( char -- )
  json-buffer json-len @ + c!
  1 json-len +!
;

\ Escape and append string (escape quotes and backslashes)
: json-escape-append ( addr len -- )
  0 ?do
    dup c@ dup [char] " = over [char] \ = or if
      [char] \ json-c-append
    then
    json-c-append
    1+
  loop
  drop
;

\ Build JSON request for Ollama
: build-ollama-json ( prompt-addr prompt-len model-addr model-len -- )
  >r >r                         \ Save model on return stack

  json-clear

  \ Opening brace
  [char] { json-c-append

  \ "model" field
  [char] " json-c-append
  s" model" json-append
  [char] " json-c-append
  [char] : json-c-append
  [char] " json-c-append
  r> r> json-append             \ Add model name
  [char] " json-c-append
  [char] , json-c-append

  \ "prompt" field
  [char] " json-c-append
  s" prompt" json-append
  [char] " json-c-append
  [char] : json-c-append
  [char] " json-c-append
  json-escape-append            \ Add escaped prompt
  [char] " json-c-append
  [char] , json-c-append

  \ "stream" field
  [char] " json-c-append
  s" stream" json-append
  [char] " json-c-append
  [char] : json-c-append
  s" false" json-append

  \ Closing brace
  [char] } json-c-append
;

\ ============================================================================
\ JSON RESPONSE PARSING
\ ============================================================================

\ Check if strings match character by character
: str-match? ( addr1 addr2 len -- flag )
  0 ?do
    over c@ over c@ <> if
      2drop false exit
    then
    1+ swap 1+
  loop
  2drop true
;

\ Find field in JSON response
: find-json-field ( response-addr response-len field-addr field-len -- value-addr value-len found? )
  >r >r                         \ Save field addr and len
  begin
    dup 0> while                \ While response has content
    \ Look for opening quote
    2dup [char] " scan dup 0> if
      1 /string                 \ Skip opening quote
      \ Check if this is our field
      2dup r@ r@ compare 0= if
        r@ /string              \ Skip field name
        \ Look for colon
        2dup [char] : scan dup 0> if
          1 /string             \ Skip colon
          \ Skip whitespace
          begin
            dup 0> over c@ bl = and while
            1 /string
          repeat
          \ Check if value starts with quote
          dup 0> if
            over c@ [char] " = if
              1 /string         \ Skip opening quote
              \ Find closing quote
              2dup [char] " scan dup 0> if
                >r 2dup r> -    \ Calculate value length
                rot drop        \ Remove extra copy
                r> r> 2drop     \ Clean up field info
                true exit
              then
              r> drop
            then
          then
        then
      then
    then
    1 /string                   \ Advance to next character
  repeat
  r> r> 2drop
  2drop 0 0 false
;

\ Parse Ollama JSON response to extract "response" field
: parse-ollama-response ( response-addr response-len -- result-addr result-len success? )
  s" response" find-json-field
;

\ ============================================================================
\ MAIN OLLAMA INTERFACE
\ ============================================================================

\ Query Ollama with specific model
: query-ollama ( prompt-addr prompt-len model-addr model-len -- response-addr response-len success? )
  build-ollama-json

  \ Execute HTTP POST
  json-buffer json-len @
  ollama-url ollama-url-len @
  http-post if
    \ Parse JSON response
    parse-ollama-response
  else
    \ HTTP request failed
    false
  then
;

\ Simple query with default model
: ollama ( prompt-addr prompt-len -- response-addr response-len success? )
  s" llama3.2" query-ollama
;

\ ============================================================================
\ TEST AND DEMO FUNCTIONS
\ ============================================================================

\ Test Ollama connection
: test-ollama
  .( Testing Ollama connection... ) cr
  s" What is 2+2?" s" llama3.2" query-ollama if
    .( Response: ) type cr
  else
    2drop .( Failed to get response ) cr
  then
;

\ Interactive Ollama prompt
create input-buffer 1024 allot

: ask ( -- )
  .( Enter prompt: ) cr
  input-buffer 1024 accept     \ Read input
  input-buffer swap s" llama3.2" query-ollama if
    cr .( AI: ) type cr cr
  else
    2drop .( Request failed ) cr
  then
;

\ ============================================================================
\ EXAMPLES
\ ============================================================================

\ Example: Simple question
\ s" What is the capital of France?" ollama if type cr else 2drop .( Failed ) cr then

\ Example: Math question
\ s" Calculate 15 * 23" s" llama3.2" query-ollama if type cr else 2drop .( Failed ) cr then

\ Example: Code generation
\ s" Write a simple bubble sort in Python" ollama if type cr else 2drop .( Failed ) cr then

.( Ollama client loaded ) cr
.( Usage: s\" your prompt here\" ollama if type cr else 2drop then ) cr
.( Or run: test-ollama ) cr
