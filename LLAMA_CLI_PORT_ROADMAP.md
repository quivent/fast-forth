# Porting Llama CLI to FastForth: Implementation Roadmap

## 📊 Current Status

**Llama CLI**: 422 lines of Forth (using gforth)
**FastForth**: Forth JIT compiler with recursion support ✅

## 🎯 Goal

Run `./fastforth-llama "What is recursion?"` → streams response from Ollama

---

## 📋 Phase 1: File I/O Support (CRITICAL)

**Required words**:
```forth
create-file  ( c-addr u fam -- fileid ior )
open-file    ( c-addr u fam -- fileid ior )
close-file   ( fileid -- ior )
read-file    ( c-addr u fileid -- u ior )
write-file   ( c-addr u fileid -- ior )
delete-file  ( c-addr u -- ior )
```

**File access modes**:
```forth
r/o  ( -- fam )  \ Read-only
w/o  ( -- fam )  \ Write-only
r/w  ( -- fam )  \ Read-write
```

**Implementation approach**:
1. Add to SSA IR: `SSAInstruction::FileOpen`, `FileRead`, `FileWrite`, etc.
2. Translate to Cranelift: use libc FFI
   ```rust
   // C functions to call
   extern "C" {
       fn fopen(path: *const c_char, mode: *const c_char) -> *mut FILE;
       fn fread(ptr: *mut c_void, size: usize, count: usize, stream: *mut FILE) -> usize;
       fn fwrite(ptr: *const c_void, size: usize, count: usize, stream: *mut FILE) -> usize;
       fn fclose(stream: *mut FILE) -> c_int;
   }
   ```
3. Add to semantic analyzer as builtins
4. Create FFI bindings in backend

**Complexity**: **Medium** (3-5 days)
- Need FFI infrastructure (new)
- Multiple system calls to wrap
- Error handling (ior return codes)

**Test case**:
```forth
: test-file-io
  s" /tmp/test.txt" w/o create-file
  if drop ." Failed to create" exit then
  { file-id }
  s" Hello from FastForth!" file-id write-file drop
  file-id close-file drop
  ." File written successfully"
;
```

---

## 📋 Phase 2: System Call Support (CRITICAL)

**Required word**:
```forth
system  ( c-addr u -- return-code )
```

**What it does**:
- Executes shell command
- Returns exit code (0 = success)

**Used for**:
```forth
\ Example from http-simple.fs:
s" nc 192.222.57.162 11434 < /tmp/http-req > /tmp/http-resp 2>/dev/null" system
```

**Implementation approach**:
1. Add `SSAInstruction::System { command_addr, command_len }`
2. Translate to C `system()` call:
   ```rust
   extern "C" {
       fn system(command: *const c_char) -> c_int;
   }
   ```
3. Need to null-terminate string (Forth strings aren't null-terminated)

**Complexity**: **Low-Medium** (1-2 days)
- Simpler than file I/O (one function)
- String termination handling needed

**Test case**:
```forth
: test-system
  s" echo Hello from FastForth" system
  0= if ." Success!" else ." Failed!" then
;
```

---

## 📋 Phase 3: String Operations (HIGH PRIORITY)

**Required words**:
```forth
place         ( addr len dest -- )      \ Store counted string
cmove         ( addr1 addr2 u -- )       \ Copy memory
scan          ( addr len char -- addr' len' )  \ Search for char
string-prefix? ( addr1 len1 addr2 len2 -- flag )  \ Check prefix
>number       ( ud addr len -- ud' addr' len' )    \ Parse number
```

**Implementation approach**:
1. Most are **memory operations** - should be straightforward
2. Add to SSA IR or implement as library functions
3. `>number` is trickier (number parsing with base support)

**Complexity**: **Low** (2-3 days)
- Mostly pointer arithmetic
- Well-defined semantics

**Test case**:
```forth
create test-buf 256 allot
: test-strings
  s" Hello" test-buf place
  test-buf count type  \ Should print "Hello"
;
```

---

## 📋 Phase 4: I/O Words (MEDIUM PRIORITY)

**Required words**:
```forth
type   ( addr len -- )        \ Print string
emit   ( char -- )            \ Print character
cr     ( -- )                 \ Print newline
.      ( n -- )               \ Print number
```

**Implementation approach**:
1. Use libc `printf` family
2. Or write to stdout directly
   ```rust
   extern "C" {
       fn write(fd: c_int, buf: *const c_void, count: usize) -> isize;
   }
   ```

**Complexity**: **Low** (1-2 days)
- Basic I/O functions
- Could start with minimal implementations

**Test case**:
```forth
: test-io
  ." Hello " 42 . cr
  ." from FastForth!" cr
;
```

---

## 📋 Phase 5: Local Variables (OPTIONAL - Nice to Have)

**Current Llama CLI usage**:
```forth
: build-json-request ( prompt-addr prompt-len model-addr model-len -- )
  { prompt-addr prompt-len model-addr model-len }  \ Local vars
  ...
  prompt-addr prompt-len json-append
  model-addr model-len json-append
;
```

**Workaround** (no locals needed):
```forth
: build-json-request ( prompt-addr prompt-len model-addr model-len -- )
  2swap                    \ Reorder stack
  json-append              \ Use model
  json-append              \ Use prompt
;
```

**Implementation if desired**:
1. Parse `{ }` syntax in frontend
2. Allocate locals on return stack or in dedicated frame
3. Generate load/store instructions

**Complexity**: **Medium-High** (4-6 days)
- Parser changes
- Stack frame management
- Not strictly needed - can rewrite code

**Priority**: 🟢 **Low** (Can port CLI without this)

---

## 📋 Phase 6: JSON Support (OPTIONAL - Future Enhancement)

**Current approach**: Manual string building
```forth
s\" {" json-append
[char] " json-buffer json-len @ + c! 1 json-len +!
s" model" json-append
```

**Future enhancement**: Dedicated JSON library
```forth
json-object-start
  s" model" s" llama3.2" json-string
  s" prompt" user-prompt json-string
json-object-end
```

**Priority**: 🟢 **Low** (Manual approach works fine)

---

## 🗓️ Timeline Estimate

### **Minimum Viable Port** (Core features only)
| Phase | Feature | Time | Dependencies |
|-------|---------|------|--------------|
| 1 | File I/O | 3-5 days | FFI infrastructure |
| 2 | System calls | 1-2 days | FFI infrastructure |
| 3 | String ops | 2-3 days | Memory operations |
| 4 | I/O words | 1-2 days | libc bindings |
| **Total** | **Minimum viable** | **7-12 days** | |

### **Full-Featured Port** (Everything)
| Phase | Feature | Time | Optional? |
|-------|---------|------|-----------|
| 1-4 | Core (above) | 7-12 days | Required |
| 5 | Local variables | 4-6 days | ✅ Optional |
| 6 | JSON library | 3-4 days | ✅ Optional |
| **Total** | **Full port** | **14-22 days** | |

---

## 🎯 Recommended Approach

### **Option A: Quick & Dirty** (7-12 days)
1. Implement File I/O (Phase 1)
2. Implement System calls (Phase 2)
3. Implement String ops (Phase 3)
4. Implement I/O words (Phase 4)
5. **Port Llama CLI as-is** (rewrite locals using stack)

**Result**: Functional Llama CLI in FastForth

### **Option B: Gradual Enhancement** (Recommended)
1. Start with **Phase 2** (system calls) - easiest
2. Test: `system("curl http://localhost:11434/api/generate")`
3. Add **Phase 4** (I/O) - print responses
4. Add **Phase 3** (strings) - build JSON manually in Rust helper
5. Add **Phase 1** (File I/O) last - complete independence

**Result**: Incremental progress, testable at each step

### **Option C: Use Existing CLI** (0 days)
Keep using `gforth` for Llama CLI, use FastForth for other projects.

**Result**: No work needed, both tools coexist

---

## 🔑 Key Architectural Decision

**FFI Infrastructure Required**

FastForth currently **only generates native code** - it doesn't call external C functions. To add file I/O and system calls, you need:

### **New Component: FFI Support**

```rust
// backend/src/cranelift/ffi.rs

pub struct FFIRegistry {
    functions: HashMap<String, (*const u8, Signature)>,
}

impl FFIRegistry {
    pub fn register_libc(&mut self) {
        // Register C library functions
        self.register("fopen", libc::fopen as *const u8, fopen_sig);
        self.register("fread", libc::fread as *const u8, fread_sig);
        self.register("system", libc::system as *const u8, system_sig);
        // ... etc
    }
}
```

**Cranelift translation**:
```rust
SSAInstruction::FileOpen { path, mode, dest } => {
    let ffi_ref = self.ffi_registry.get("fopen");
    let path_val = self.get_register(path);
    let mode_val = self.get_register(mode);

    let call = self.builder.ins().call(ffi_ref, &[path_val, mode_val]);
    let result = self.builder.inst_results(call)[0];
    self.def_register(dest, result);
}
```

---

## 📊 Complexity Breakdown

| Component | New Code | Difficulty | Risk |
|-----------|----------|------------|------|
| FFI infrastructure | ~300 lines | Medium | Low |
| File I/O words | ~200 lines | Low | Low |
| System call | ~50 lines | Low | Medium (security) |
| String ops | ~400 lines | Low | Low |
| I/O words | ~150 lines | Low | Low |
| **Total** | **~1,100 lines** | | |

**Comparison**:
- Recursion implementation: ~2,500 lines, High complexity
- Llama CLI port: ~1,100 lines, Medium complexity

---

## 🚀 Getting Started

### **Step 1: Verify FFI Works**

Test with a simple C function call:

```rust
// Test: Call C printf from FastForth
: test-ffi
  s" Hello from C!" c-print
;
```

### **Step 2: Implement System Call**

Simplest useful feature - proves FFI works:

```forth
: test-ollama
  s" curl http://localhost:11434/api/generate -d '{\"model\":\"llama3.2\",\"prompt\":\"hi\"}'"
  system
  0= if ." Success!" else ." Failed!" then
;
```

### **Step 3: Add File I/O**

Now you can save/load responses without system calls.

### **Step 4: Port Llama CLI**

With FFI + File I/O + System + Strings, you have everything needed.

---

## 💡 Alternative: Hybrid Approach

**Keep using gforth for Llama CLI, use FastForth as library**:

```bash
#!/bin/bash
# llama-fast - Hybrid CLI

# Use FastForth for compute-intensive parts
./fastforth execute ": process-response ... ;" > /tmp/result

# Use gforth for I/O and system interaction
gforth ollama-client.fs -e "call-ollama" -e "bye"
```

**Benefits**:
- No FFI implementation needed
- Best of both worlds (fast compute + rich I/O)
- Works today with zero changes

**Drawback**:
- Not a pure FastForth solution
- IPC overhead (minimal for LLM latency)

---

## 🎯 Recommendation

**Start with Option B (Gradual Enhancement)** OR **Use Hybrid Approach**

**Why**:
1. FFI is useful beyond just Llama CLI (opens up entire C ecosystem)
2. System calls enable process orchestration
3. File I/O is fundamental for real programs
4. These features benefit all FastForth users

**Timeline**:
- **Week 1-2**: FFI + system calls (minimal viable)
- **Week 3**: File I/O (complete independence)
- **Week 4**: Polish + documentation

**Deliverable**: FastForth with full OS interaction capabilities + Llama CLI port

---

*This is a living document. Update as implementation progresses.*
