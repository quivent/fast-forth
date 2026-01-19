# FastForth HTTP Client Library

HTTP POST client and Ollama AI integration for FastForth, ported from the Llama CLI gforth implementation.

## Quick Start

```forth
\ Load Ollama client (includes all dependencies)
include examples/ollama-client.fth

\ Query AI model
s" What is 2+2?" ollama if
  type cr
else
  2drop .( Failed ) cr
then
```

## Library Structure

```
lib/
├── strings.fth              # String manipulation primitives
└── http-client.fth          # HTTP POST using netcat

examples/
├── ollama-client.fth        # Ollama API integration
├── test-http-client.fth     # Full test suite
└── minimal-http-test.fth    # Basic tests (no Ollama needed)

docs/
├── HTTP_CLIENT_GUIDE.md     # Complete documentation
├── HTTP_CLIENT_QUICKREF.md  # Quick reference
└── LLAMA_CLI_PORT_SUMMARY.md # Port details
```

## Installation

No installation needed - libraries are included in FastForth distribution.

**Requirements**:
- FastForth with file I/O and system call support
- `nc` (netcat) command available
- Ollama server (for AI queries)

## Usage

### 1. String Manipulation

```forth
include lib/strings.fth

\ Copy memory
create buf 100 allot
s" Hello" buf 10 cmove

\ Store counted string
s" World" buf place

\ Find character
s" Hello World" [char] o scan
\ Returns: "o World"

\ Search substring
s" Hello World" s" World" search if
  \ Found
else
  2drop
then
```

### 2. HTTP POST

```forth
include lib/http-client.fth

\ Simple POST request
s" {\"test\":\"value\"}"              \ JSON body
s" http://localhost:8080/api/test"    \ URL
http-post if
  type cr                              \ Print response
else
  type cr                              \ Print error
then
```

### 3. Ollama Integration

```forth
include examples/ollama-client.fth

\ Default model (llama3.2)
s" Explain FastForth" ollama if
  type cr
else
  2drop
then

\ Specific model
s" Write a Forth function to factorial" s" gpt-oss:120b" query-ollama if
  type cr
else
  2drop
then

\ Interactive mode
ask
```

## Configuration

```forth
\ Change Ollama server
s" http://localhost:11434" set-ollama-url

\ Check current URL
ollama-url ollama-url-len @ type cr
```

## API Reference

### String Library (`lib/strings.fth`)

| Function | Stack Effect | Description |
|----------|-------------|-------------|
| `place` | `( addr len dest -- )` | Store counted string |
| `cmove` | `( src dest len -- )` | Copy memory |
| `count` | `( c-addr -- addr len )` | Get counted string as addr/len |
| `scan` | `( addr len char -- addr' len' )` | Find character in string |
| `search` | `( a1 l1 a2 l2 -- a3 l3 flag )` | Find substring |
| `compare` | `( a1 l1 a2 l2 -- n )` | Compare two strings |
| `/string` | `( addr len n -- addr+n len-n )` | Advance string pointer |
| `string-prefix?` | `( s1-a s1-l s2-a s2-l -- flag )` | Check prefix |
| `blank` | `( addr len -- )` | Fill with spaces |
| `fill` | `( addr len char -- )` | Fill with character |
| `n>string` | `( n -- addr len )` | Convert number to string |

### HTTP Client (`lib/http-client.fth`)

| Function | Stack Effect | Description |
|----------|-------------|-------------|
| `parse-url` | `( url-a url-l -- host-a host-l port )` | Extract host and port |
| `build-http-post` | `( json-a json-l host-a host-l port -- )` | Build HTTP request |
| `http-post` | `( json-a json-l url-a url-l -- resp-a resp-l ok? )` | Execute POST |
| `http-append` | `( addr len -- )` | Append to HTTP buffer |
| `http-clear` | `( -- )` | Clear HTTP buffer |

### Ollama Client (`examples/ollama-client.fth`)

| Function | Stack Effect | Description |
|----------|-------------|-------------|
| `query-ollama` | `( prompt-a prompt-l model-a model-l -- resp-a resp-l ok? )` | Query with model |
| `ollama` | `( prompt-a prompt-l -- resp-a resp-l ok? )` | Query default model |
| `set-ollama-url` | `( addr len -- )` | Set server URL |
| `test-ollama` | `( -- )` | Test connection |
| `ask` | `( -- )` | Interactive prompt |

## Implementation Details

### HTTP POST Flow

1. **Parse URL**: Extract host and port from URL string
2. **Build Request**: Create HTTP POST request with headers and JSON body
3. **Write to File**: Save request to `/tmp/fastforth-http-req`
4. **Execute Netcat**: `nc host port < req > resp`
5. **Read Response**: Load response from `/tmp/fastforth-http-resp`
6. **Parse Response**: Extract body after HTTP headers
7. **Cleanup**: Delete temporary files

### JSON Building

```forth
\ Ollama JSON request format:
{
  "model": "llama3.2",
  "prompt": "user prompt here",
  "stream": false
}
```

### JSON Parsing

Extracts `"response"` field from Ollama's JSON:

```json
{
  "model": "llama3.2",
  "created_at": "...",
  "response": "AI generated text here",
  "done": true
}
```

## Testing

### Basic Test (no Ollama)

```bash
cd /Users/joshkornreich/Documents/Projects/FastForth
./fastforth examples/minimal-http-test.fth
```

Expected output:
```
============================================
Minimal HTTP Client Test
============================================

Test 1: String functions
cmove: Hello
place: World

Test 2: URL parsing
Port: 8080
Host len: 9
Host: localhost

Test 3: JSON building
JSON: {"test":"value"}

Test 4: File I/O
Write OK
Read: test data

============================================
All basic tests complete
============================================
```

### Full Test (requires Ollama)

```bash
# Terminal 1: Start Ollama
ollama serve

# Terminal 2: Run tests
cd /Users/joshkornreich/Documents/Projects/FastForth
./fastforth examples/test-http-client.fth
```

## Troubleshooting

### Connection Failed

```forth
\ Verify Ollama is running
s" curl http://localhost:11434/api/tags" system drop

\ Check URL configuration
ollama-url ollama-url-len @ type cr
```

### Netcat Not Found

```bash
# Check netcat availability
which nc
# Should show: /usr/bin/nc

# Install if missing (macOS)
brew install netcat

# Install if missing (Linux)
sudo apt-get install netcat
```

### File I/O Errors

```bash
# Check temp directory
ls -la /tmp/
# Should be writable: drwxrwxrwt

# Check permissions
touch /tmp/test-write && rm /tmp/test-write
```

### Response Parsing Failed

- Ensure Ollama is returning valid JSON
- Check HTTP response format (should have `\r\n\r\n` separator)
- Verify response fits in 16KB buffer

## Performance

| Metric | Value |
|--------|-------|
| Local request latency | ~50-100ms |
| HTTP buffer size | 16KB |
| JSON buffer size | 4KB |
| Max response size | 16KB |
| Dependencies | netcat only |

## Examples

### Simple Q&A

```forth
: qa ( -- )
  s" What is the capital of France?" ollama if
    .( Answer: ) type cr
  else
    2drop .( Failed ) cr
  then
;
```

### Batch Processing

```forth
: batch-questions ( -- )
  s" What is 2+2?" ollama if type cr cr else 2drop then
  s" What is the speed of light?" ollama if type cr cr else 2drop then
  s" Who wrote Hamlet?" ollama if type cr cr else 2drop then
;
```

### Code Generation

```forth
: generate-code ( -- )
  s" Write a Forth function to reverse a string" ollama if
    cr .( Generated Code: ) cr type cr
  else
    2drop .( Failed ) cr
  then
;
```

## Porting Notes

Ported from gforth implementation with these changes:

- **No locals**: Used variables instead of `{ var }`
- **String functions**: Implemented from scratch (place, cmove, scan, etc.)
- **Stack management**: Explicit variable storage
- **Memory**: Fixed buffers instead of allocate/free
- **File I/O**: Adapted to FastForth's ANS Forth file words

## Contributing

Improvements welcome:
- HTTP GET support
- Streaming response handling
- HTTPS support (via openssl)
- Response caching
- Error recovery
- Multiple endpoints

## License

Same as FastForth main project.

## Credits

Ported from gforth Llama CLI:
- Original: `/Users/joshkornreich/Documents/Projects/Ollama/llama/variants/`
- Adapted for FastForth file I/O and system calls

## Resources

- [FastForth Documentation](../README.md)
- [HTTP Client Guide](../docs/HTTP_CLIENT_GUIDE.md)
- [Quick Reference](../docs/HTTP_CLIENT_QUICKREF.md)
- [Port Summary](../docs/LLAMA_CLI_PORT_SUMMARY.md)
