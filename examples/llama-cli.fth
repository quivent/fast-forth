\ FastForth Llama CLI - Standalone Program
\ This is the FUTURE version that will work when FFI is implemented
\ Current status: Demonstrates intended architecture

\ NOTE: This code will not run until FFI support is added to FastForth.
\ See LLAMA_CLI_PORT_ROADMAP.md for implementation plan.

\ ============================================================================
\ CONSTANTS & CONFIGURATION
\ ============================================================================

\ Ollama API configuration
11434 constant OLLAMA-PORT
create OLLAMA-HOST ," localhost"
create OLLAMA-ENDPOINT ," /api/generate"

\ Buffer sizes
16384 constant JSON-BUFFER-SIZE
16384 constant RESPONSE-BUFFER-SIZE
256 constant TEMP-FILE-SIZE

\ ============================================================================
\ BUFFERS
\ ============================================================================

create json-buffer JSON-BUFFER-SIZE allot
variable json-len

create response-buffer RESPONSE-BUFFER-SIZE allot
variable response-len

create temp-file TEMP-FILE-SIZE allot

\ ============================================================================
\ STRING UTILITIES (Will need to be implemented in FastForth)
\ ============================================================================

\ Copy string to buffer
: str-copy ( src-addr src-len dest-addr -- )
  swap 0 do
    over i + c@
    over i + c!
  loop
  2drop
;

\ Append string to json buffer
: json-append ( addr len -- )
  dup json-len @ + JSON-BUFFER-SIZE > if
    2drop
    ." JSON buffer overflow!" cr
    1 exit  \ Exit with error
  then

  json-buffer json-len @ + swap str-copy
  json-len +!
;

\ ============================================================================
\ JSON REQUEST BUILDER
\ ============================================================================

\ Build JSON request for Ollama API
: build-json-request ( prompt-addr prompt-len model-addr model-len -- )
  \ Clear buffer
  json-buffer JSON-BUFFER-SIZE blank
  0 json-len !

  \ Start JSON object with model
  s\" {\"model\":\"" json-append
  json-append  \ model name
  s\" \",\"prompt\":\"" json-append
  json-append  \ prompt
  s\" \",\"stream\":false}" json-append
;

\ ============================================================================
\ HTTP CLIENT (Requires FFI - system() call)
\ ============================================================================

\ Execute system command (REQUIRES FFI)
\ : system ( addr len -- exit-code )
\   \ This will be implemented when FFI support is added
\   \ For now, this is a placeholder
\ ;

\ Call Ollama API using curl
: call-ollama-http ( json-addr json-len -- response-addr response-len success? )
  \ Build curl command
  response-buffer RESPONSE-BUFFER-SIZE blank
  0 response-len !

  \ Create temp file name
  s" /tmp/fastforth-llama-" temp-file str-copy
  \ TODO: Add timestamp to filename
  s" .json" temp-file +place

  \ Build curl command
  \ TODO: This requires system() FFI call
  \ curl -s -X POST http://localhost:11434/api/generate \
  \   -H "Content-Type: application/json" \
  \   -d '{"model":"llama3.2","prompt":"...","stream":false}' \
  \   -o /tmp/fastforth-llama-XXX.json

  \ For now, return placeholder
  s" [FFI not yet implemented]" true
;

\ ============================================================================
\ RESPONSE PARSER (Basic JSON extraction)
\ ============================================================================

\ Extract response field from JSON
\ NOTE: This is simplified - full JSON parser would be better
: extract-response ( json-addr json-len -- response-addr response-len )
  \ Search for "response":"
  \ Extract until closing quote
  \ This is a placeholder - real implementation would be more robust

  \ For now, just return the whole JSON
;

\ ============================================================================
\ MAIN CLI INTERFACE
\ ============================================================================

\ Print usage information
: print-usage ( -- )
  ." FastForth Llama CLI - AI-powered Forth assistant" cr
  ." " cr
  ." Usage:" cr
  ."   fastforth-llama <prompt>" cr
  ."   fastforth-llama -m <model> <prompt>" cr
  ." " cr
  ." Options:" cr
  ."   -m MODEL    Set Ollama model (default: llama3.2)" cr
  ."   -h          Show this help" cr
  ." " cr
  ." Examples:" cr
  ."   fastforth-llama \"What is recursion?\"" cr
  ."   fastforth-llama -m codellama \"Explain Forth\"" cr
  cr
;

\ Parse command-line arguments
\ NOTE: This requires argc/argv access from FFI
: parse-args ( -- model-addr model-len prompt-addr prompt-len )
  \ Default model
  s" llama3.2"

  \ TODO: Parse actual command-line arguments
  \ This requires FFI access to argc/argv

  \ For now, use hardcoded prompt
  s" Hello from FastForth!"
;

\ Main entry point
: llama-main ( -- exit-code )
  \ Parse arguments
  parse-args

  \ Build JSON request
  build-json-request

  \ Make HTTP request
  json-buffer json-len @ call-ollama-http

  if
    \ Success - print response
    type cr
    0  \ Exit code 0
  else
    \ Error
    2drop
    ." Error: Failed to call Ollama API" cr
    1  \ Exit code 1
  then
;

\ ============================================================================
\ WHEN FFI IS IMPLEMENTED, THIS WILL BE THE COMPILED ENTRY POINT
\ ============================================================================

\ Uncomment when FFI is ready:
\ llama-main bye

\ ============================================================================
\ TEMPORARY: Current demo version
\ ============================================================================

: demo-llama
  ." FastForth Llama CLI Demo" cr
  ." =========================" cr cr

  ." This demonstrates the intended architecture." cr
  ." FFI support is required for:" cr
  ."   - system() calls for curl/HTTP" cr
  ."   - File I/O for temp files" cr
  ."   - Command-line argument parsing" cr
  cr

  ." See LLAMA_CLI_PORT_ROADMAP.md for implementation plan." cr
  ." " cr
  ." For now, use the shell wrapper: bin/fastforth-llama" cr
;

\ Run demo
demo-llama
