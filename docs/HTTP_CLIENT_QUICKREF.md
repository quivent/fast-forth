# FastForth HTTP Client Quick Reference

## Load Libraries

```forth
include lib/strings.fth              \ String manipulation
include lib/http-client.fth          \ HTTP POST client
include examples/ollama-client.fth   \ Ollama AI integration
```

## String Operations

| Word | Stack | Description |
|------|-------|-------------|
| `place` | `( addr len dest -- )` | Store counted string |
| `cmove` | `( src dest len -- )` | Copy memory |
| `count` | `( c-addr -- addr len )` | Get counted string |
| `scan` | `( addr len char -- addr' len' )` | Find character |
| `search` | `( a1 l1 a2 l2 -- a3 l3 flag )` | Find substring |
| `compare` | `( a1 l1 a2 l2 -- n )` | Compare strings |
| `/string` | `( addr len n -- addr+n len-n )` | Advance pointer |
| `n>string` | `( n -- addr len )` | Number to string |

## HTTP Functions

| Word | Stack | Description |
|------|-------|-------------|
| `parse-url` | `( url-addr url-len -- host-addr host-len port )` | Parse URL |
| `http-post` | `( json-a json-l url-a url-l -- resp-a resp-l ok? )` | HTTP POST |

## Ollama Functions

| Word | Stack | Description |
|------|-------|-------------|
| `query-ollama` | `( prompt-a prompt-l model-a model-l -- resp-a resp-l ok? )` | Query specific model |
| `ollama` | `( prompt-a prompt-l -- resp-a resp-l ok? )` | Query default model |
| `test-ollama` | `( -- )` | Test connection |
| `ask` | `( -- )` | Interactive prompt |

## Quick Examples

### Simple Query

```forth
include examples/ollama-client.fth

s" What is 2+2?" ollama if
  type cr
else
  2drop .( Failed ) cr
then
```

### Custom Model

```forth
s" Explain AI" s" llama3.2" query-ollama if
  type cr
else
  2drop
then
```

### Change Server

```forth
s" http://localhost:11434" set-ollama-url
```

## File Locations

```
FastForth/
├── lib/
│   ├── strings.fth          # String manipulation
│   └── http-client.fth      # HTTP POST client
├── examples/
│   ├── ollama-client.fth    # Ollama integration
│   ├── test-http-client.fth # Full test suite
│   └── minimal-http-test.fth # Basic tests
└── docs/
    ├── HTTP_CLIENT_GUIDE.md      # Complete guide
    └── HTTP_CLIENT_QUICKREF.md   # This file
```

## Common Patterns

### Error Handling

```forth
s" prompt" ollama if
  \ Success - response is on stack (addr len)
  .( Response: ) type cr
else
  \ Failure - error message on stack
  .( Error: ) type cr
then
```

### Multiple Queries

```forth
: batch-query
  s" Question 1" ollama if type cr else 2drop then
  s" Question 2" ollama if type cr else 2drop then
  s" Question 3" ollama if type cr else 2drop then
;
```

## Troubleshooting

```forth
\ Check configuration
ollama-url ollama-url-len @ type cr

\ Verify netcat is available
s" which nc" system . cr

\ Test file I/O
s" test" s" /tmp/test.txt" write-string-to-file . cr
```

## Performance Notes

- Local Ollama: ~50-100ms per request
- Remote Ollama: Depends on network latency
- Uses netcat (nc) - standard Unix tool
- Fixed 16KB HTTP buffer, 4KB JSON buffer

## Dependencies

- **Required**: netcat (`nc` command)
- **Optional**: Ollama server running
- **Platform**: Unix-like (macOS, Linux)
