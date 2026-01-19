\ HTTP Client Library for FastForth
\ Provides HTTP POST functionality using netcat
\ Requires: strings.fth, file I/O, and system call support

\ Load string manipulation library
include lib/strings.fth

\ ============================================================================
\ CONFIGURATION AND BUFFERS
\ ============================================================================

\ Response buffer for building HTTP requests and storing responses
create http-buffer 16384 allot
variable http-len

\ Temporary file paths
create temp-req-path 256 allot
create temp-resp-path 256 allot

\ Initialize temp file paths
: init-temp-paths
  s" /tmp/fastforth-http-req" temp-req-path place
  s" /tmp/fastforth-http-resp" temp-resp-path place
;

init-temp-paths

\ ============================================================================
\ BUFFER OPERATIONS
\ ============================================================================

\ Clear HTTP buffer
: http-clear ( -- )
  0 http-len !
  http-buffer 16384 blank
;

\ Append string to HTTP buffer
: http-append ( addr len -- )
  http-buffer http-len @ + swap cmove
  http-len +!
;

\ Append character to HTTP buffer
: http-c-append ( char -- )
  http-buffer http-len @ + c!
  1 http-len +!
;

\ ============================================================================
\ URL PARSING
\ ============================================================================

\ Parse URL to extract host and port
\ Returns: host-addr host-len port
: parse-url ( url-addr url-len -- host-addr host-len port )
  \ Check for http:// prefix and skip it
  2dup s" http://" string-prefix? if
    7 /string  \ Skip "http://"
  then

  \ Find colon for port separator
  2dup [char] : scan dup 0> if
    \ Found port - extract host and port
    2>r                    \ Save remainder on return stack
    2dup 2r@ drop -        \ Calculate host length
    rot drop               \ host-addr host-len
    2r> 1 /string          \ Get port string (skip colon)
    0 0 2swap >number 2drop drop  \ Convert port to number
  else
    \ No port specified, use default Ollama port
    2drop 11434
  then
;

\ ============================================================================
\ HTTP REQUEST BUILDING
\ ============================================================================

\ Helper variables for building request
variable build-port
variable build-json-addr
variable build-json-len

\ Build HTTP POST request
: build-http-post ( json-addr json-len host-addr host-len port -- )
  build-port !
  >r >r                          \ Save host on return stack
  build-json-len !
  build-json-addr !
  r> r>                          \ Restore host

  http-clear

  \ POST line
  s" POST /api/generate HTTP/1.1" http-append
  13 http-c-append 10 http-c-append  \ \r\n

  \ Host header
  s" Host: " http-append
  http-append                     \ Append host
  [char] : http-c-append
  build-port @ n>string http-append
  13 http-c-append 10 http-c-append  \ \r\n

  \ Content-Type header
  s" Content-Type: application/json" http-append
  13 http-c-append 10 http-c-append  \ \r\n

  \ Content-Length header
  s" Content-Length: " http-append
  build-json-len @ n>string http-append
  13 http-c-append 10 http-c-append  \ \r\n

  \ Connection header
  s" Connection: close" http-append
  13 http-c-append 10 http-c-append  \ \r\n

  \ Empty line before body
  13 http-c-append 10 http-c-append  \ \r\n

  \ JSON body
  build-json-addr @ build-json-len @ http-append
;

\ ============================================================================
\ RESPONSE PARSING
\ ============================================================================

\ Find double newline in HTTP response (separates headers from body)
: find-body-start ( response-addr response-len -- body-addr body-len found? )
  \ Try CRLF CRLF first
  2dup s" \r\n\r\n" search if
    4 /string true
  else
    \ Try LF LF
    2dup s" \n\n" search if
      2 /string true
    else
      2drop 0 0 false
    then
  then
;

\ ============================================================================
\ FILE I/O HELPERS
\ ============================================================================

\ Write string to file
: write-string-to-file ( addr len c-addr-path u-path -- success? )
  w/o create-file if
    drop 2drop false exit
  then
  >r                          \ Save file handle
  r@ write-file if
    r> close-file drop
    false exit
  then
  r> close-file 0=
;

\ Read file contents into buffer
: read-file-to-buffer ( c-addr-path u-path buffer bufsize -- len success? )
  >r >r                       \ Save buffer and bufsize
  r/o open-file if
    drop r> r> 2drop
    0 false exit
  then
  >r                          \ Save file handle
  r> r> r@ read-file if       \ buffer bufsize fileid
    drop r> close-file drop
    0 false exit
  then
  r> close-file if
    drop 0 false
  else
    true
  then
;

\ ============================================================================
\ HTTP POST EXECUTION
\ ============================================================================

\ Helper variables for HTTP execution
variable exec-host-addr
variable exec-host-len
variable exec-port
create exec-cmd-buffer 512 allot
variable exec-cmd-len

\ Execute HTTP POST using netcat
: http-post ( json-addr json-len url-addr url-len -- response-addr response-len success? )
  \ Save URL for parsing
  parse-url                    \ ( json-addr json-len host-addr host-len port )
  exec-port !
  exec-host-len !
  exec-host-addr !

  \ Save JSON parameters
  build-json-len !
  build-json-addr !

  \ Build HTTP request
  build-json-addr @ build-json-len @
  exec-host-addr @ exec-host-len @
  exec-port @
  build-http-post

  \ Write request to temp file
  http-buffer http-len @
  temp-req-path count write-string-to-file
  0= if
    s" Failed to write request file" false exit
  then

  \ Build netcat command in separate buffer
  exec-cmd-buffer 512 blank
  0 exec-cmd-len !

  s" nc " exec-cmd-buffer exec-cmd-len @ + swap cmove
  3 exec-cmd-len +!

  exec-host-addr @ exec-host-len @ exec-cmd-buffer exec-cmd-len @ + swap cmove
  exec-host-len @ exec-cmd-len +!

  s"  " exec-cmd-buffer exec-cmd-len @ + swap cmove
  1 exec-cmd-len +!

  exec-port @ n>string exec-cmd-buffer exec-cmd-len @ + swap cmove
  exec-cmd-len +!

  s"  < " exec-cmd-buffer exec-cmd-len @ + swap cmove
  3 exec-cmd-len +!

  temp-req-path count exec-cmd-buffer exec-cmd-len @ + swap cmove
  exec-cmd-len +!

  s"  > " exec-cmd-buffer exec-cmd-len @ + swap cmove
  3 exec-cmd-len +!

  temp-resp-path count exec-cmd-buffer exec-cmd-len @ + swap cmove
  exec-cmd-len +!

  s"  2>/dev/null" exec-cmd-buffer exec-cmd-len @ + swap cmove
  12 exec-cmd-len +!

  \ Execute netcat command
  exec-cmd-buffer exec-cmd-len @ system
  0= if
    \ Read response file
    temp-resp-path count http-buffer 16384 read-file-to-buffer if
      http-len !

      \ Clean up temp files
      temp-req-path count delete-file drop
      temp-resp-path count delete-file drop

      \ Parse HTTP response to get body
      http-buffer http-len @ find-body-start if
        true
      else
        s" Failed to parse HTTP response" false
      then
    else
      \ Failed to read response
      temp-req-path count delete-file drop
      temp-resp-path count delete-file drop
      s" Failed to read response file" false
    then
  else
    \ Netcat command failed
    temp-req-path count delete-file drop
    temp-resp-path count delete-file drop
    s" Netcat command failed" false
  then
;

\ ============================================================================
\ CONVENIENCE FUNCTIONS
\ ============================================================================

\ Simple HTTP GET (future expansion)
\ : http-get ( url-addr url-len -- response-addr response-len success? )
\   \ Not implemented yet
\   2drop s" GET not implemented" false
\ ;

.( HTTP client library loaded ) cr
