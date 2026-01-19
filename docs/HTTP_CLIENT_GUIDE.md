# FastForth HTTP Client Guide

This guide covers the HTTP client functionality ported from the Llama CLI gforth implementation to FastForth.

## Overview

The FastForth HTTP client provides:
- **String manipulation library** (`lib/strings.fth`) - Basic string operations
- **HTTP client library** (`lib/http-client.fth`) - HTTP POST using netcat
- **Ollama client** (`examples/ollama-client.fth`) - AI model interaction via Ollama API

## Installation

No installation required. The libraries are included in the FastForth distribution.

## Quick Start

```forth
\ Load the Ollama client (includes all dependencies)
include examples/ollama-client.fth

\ Query Ollama with default model (llama3.2)
s" What is 2+2?" ollama if
  type cr
else
  2drop .( Request failed ) cr
then
```

## Architecture

### 1. String Manipulation Library (`lib/strings.fth`)

Provides essential string operations:

| Word | Stack Effect | Description |
|------|-------------|-------------|
| `place` | `( addr len dest -- )` | Store counted string |
| `cmove` | `( src dest len -- )` | Copy memory |
| `count` | `( c-addr -- addr len )` | Get counted string |
| `scan` | `( addr len char -- addr' len' )` | Find character |
| `search` | `( addr1 len1 addr2 len2 -- addr3 len3 flag )` | Find substring |
| `compare` | `( addr1 len1 addr2 len2 -- n )` | Compare strings |
| `/string` | `( addr len n -- addr+n len-n )` | Advance string |
| `string-prefix?` | `( str-addr str-len prefix-addr prefix-len -- flag )` | Check prefix |
| `blank` | `( addr len -- )` | Fill with spaces |
| `fill` | `( addr len char -- )` | Fill with character |
| `n>string` | `( n -- addr len )` | Convert number to string |

### 2. HTTP Client Library (`lib/http-client.fth`)

#### Configuration

```forth
\ Temporary files used by HTTP client
/tmp/fastforth-http-req   \ HTTP request file
/tmp/fastforth-http-resp  \ HTTP response file
```

#### Main Functions

| Word | Stack Effect | Description |
|------|-------------|-------------|
| `parse-url` | `( url-addr url-len -- host-addr host-len port )` | Extract host and port from URL |
| `build-http-post` | `( json-addr json-len host-addr host-len port -- )` | Build HTTP POST request |
| `http-post` | `( json-addr json-len url-addr url-len -- response-addr response-len success? )` | Execute HTTP POST |

#### Internal Implementation

The HTTP client uses **netcat** for TCP connections:

1. Builds HTTP POST request with proper headers
2. Writes request to temporary file
3. Executes: `nc host port < /tmp/req > /tmp/resp`
4. Reads response from temporary file
5. Parses HTTP response to extract body
6. Cleans up temporary files

### 3. Ollama Client (`examples/ollama-client.fth`)

#### Configuration

```forth
\ Default Ollama server
init-ollama-url  \ Sets to http://192.222.57.162:11434

\ Change server URL
s" http://localhost:11434" set-ollama-url
```

#### Main Functions

| Word | Stack Effect | Description |
|------|-------------|-------------|
| `query-ollama` | `( prompt-addr prompt-len model-addr model-len -- response-addr response-len success? )` | Query with specific model |
| `ollama` | `( prompt-addr prompt-len -- response-addr response-len success? )` | Query with default model |
| `test-ollama` | `( -- )` | Test connection |
| `ask` | `( -- )` | Interactive prompt |

#### JSON Request Format

```json
{
  "model": "llama3.2",
  "prompt": "user prompt here",
  "stream": false
}
```

#### JSON Response Parsing

Extracts the `"response"` field from Ollama's JSON response:

```json
{
  "model": "llama3.2",
  "created_at": "2025-11-15T...",
  "response": "AI generated response here",
  "done": true
}
```

## Usage Examples

### Basic Query

```forth
\ Simple question
s" What is the capital of France?" ollama if
  .( AI: ) type cr
else
  2drop .( Failed ) cr
then
```

### Custom Model

```forth
\ Use specific model
s" Explain quantum computing" s" gpt-oss:120b" query-ollama if
  type cr
else
  2drop .( Error ) cr
then
```

### Multiple Queries

```forth
\ Math question
s" Calculate 15 * 23" ollama if
  .( Answer: ) type cr
else
  2drop
then

\ Code generation
s" Write a Python function to reverse a string" ollama if
  cr type cr
else
  2drop
then
```

### Interactive Mode

```forth
\ Start interactive prompt
ask
\ User types prompt, gets AI response
```

## Testing

Run the test suite:

```forth
include examples/test-http-client.fth
```

This tests:
1. String manipulation functions
2. URL parsing
3. JSON building
4. HTTP POST execution (requires Ollama server)

## Troubleshooting

### Connection Failed

```forth
\ Check Ollama server URL
ollama-url ollama-url-len @ type cr

\ Verify server is running
\ Command line: curl http://192.222.57.162:11434/api/tags
```

### Netcat Not Found

Ensure `nc` (netcat) is available:

```bash
which nc
# Should show: /usr/bin/nc or similar
```

### File I/O Errors

Check temporary directory permissions:

```bash
ls -la /tmp/
# Should be writable: drwxrwxrwt
```

### Response Parsing Failed

The HTTP client expects standard HTTP/1.1 responses with `\r\n\r\n` or `\n\n` header separator.

## Performance

- **Netcat approach**: ~50-100ms per request (local server)
- **No external dependencies**: Uses only netcat (standard Unix tool)
- **Memory efficient**: Fixed buffers (16KB HTTP, 4KB JSON)

## Comparison with gforth Version

| Feature | gforth | FastForth |
|---------|--------|-----------|
| Locals (`{ }`) | ✅ Yes | ❌ No (uses variables) |
| String literals | ✅ `s" ..."` | ✅ `s" ..."` |
| File I/O | ✅ Standard | ✅ Newly implemented |
| System calls | ✅ `system` | ✅ `system` |
| Return stack | ✅ Yes | ✅ Yes |

### Key Adaptations

1. **No locals**: Used variables instead of `{ var1 var2 }`
2. **Stack management**: Explicit variable storage vs. locals
3. **String operations**: Implemented from scratch (place, cmove, scan, etc.)
4. **Number formatting**: Custom `n>string` implementation

## Future Enhancements

Potential improvements:

1. **HTTP GET support**: Add read-only requests
2. **Streaming responses**: Handle streamed JSON responses
3. **Error handling**: More detailed error messages
4. **Caching**: Cache responses for repeated queries
5. **Multiple endpoints**: Support different Ollama API endpoints
6. **Authentication**: Add API key support

## Integration Examples

### CLI Tool

```forth
\ Save as ollama-ask.fth
include examples/ollama-client.fth

\ Get prompt from command line
: main
  argc 1 > if
    \ Get all arguments as prompt
    1 argc 1- do
      i arg s"  " s+ s+
    loop
    ollama if type cr else 2drop .( Error ) cr then
  else
    .( Usage: fastforth ollama-ask.fth "your prompt" ) cr
  then
;

main bye
```

### Batch Processing

```forth
\ Process multiple prompts from file
: batch-ollama ( filename-addr filename-len -- )
  r/o open-file throw >r
  begin
    1024 allocate throw
    dup 1024 r@ read-line throw
  while
    swap ollama if
      type cr cr
    else
      2drop
    then
    free throw
  repeat
  drop free throw
  r> close-file throw
;

\ Usage: s" prompts.txt" batch-ollama
```

## API Reference

See inline documentation in:
- `/Users/joshkornreich/Documents/Projects/FastForth/lib/strings.fth`
- `/Users/joshkornreich/Documents/Projects/FastForth/lib/http-client.fth`
- `/Users/joshkornreich/Documents/Projects/FastForth/examples/ollama-client.fth`

## License

Same as FastForth main project.

## Credits

Ported from gforth Llama CLI implementation:
- Original: `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/http-simple.fs`
- Original: `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/ollama-client-pure.fs`

Adapted for FastForth by incorporating file I/O and system call capabilities.
