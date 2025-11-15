# FFI and File I/O Architecture Design

## Executive Summary

This document outlines the architecture for adding Foreign Function Interface (FFI) and File I/O capabilities to FastForth, enabling seamless integration with C libraries and POSIX file operations.

## Phase 1: Requirements Analysis (COMPLETED)

### Current State Analysis
- FastForth has working JIT compilation with Cranelift backend
- Recursion support is functional
- Current SSA IR supports: arithmetic, stack ops, control flow, memory ops
- Missing: External C function calls, File I/O operations

### Requirements
1. **FFI Infrastructure**: Call arbitrary C functions from Forth code
2. **File I/O**: ANS Forth file operations (create, open, read, write, close, delete)
3. **System Calls**: Execute shell commands via `system()`
4. **Performance**: No degradation to existing ~50ms JIT compile times
5. **Standards Compliance**: ANS Forth file I/O word set

## Phase 2: Architecture Design

### Component Architecture

```
FastForth Architecture (with FFI)
=====================================

Frontend (frontend/src/*)
├── semantic.rs         → Add file I/O builtins
├── ssa.rs             → Add FFI/File SSA instructions
└── ast.rs             → No changes needed

Backend (backend/src/cranelift/*)
├── mod.rs             → Export FFI module
├── compiler.rs        → FFI registry integration
├── translator.rs      → FFI call translation
└── ffi.rs (NEW)       → FFI infrastructure
    ├── FFIRegistry    → C function registry
    ├── FFISignature   → C function signatures
    └── FFICall        → Cranelift call generation

Runtime (implicit in generated code)
└── libc bindings      → fopen, fread, fwrite, fclose, system
```

### Data Flow

```
Forth Source
    ↓
[ Parser ] → AST
    ↓
[ Semantic ] → Validated AST (file I/O words recognized)
    ↓
[ SSA Converter ] → SSA IR with FFI instructions
    ↓
[ Cranelift Translator ] → Cranelift IR with external calls
    ↓
[ JIT Compiler ] → Native x86-64 with libc linkage
    ↓
[ Execute ] → Calls libc functions at runtime
```

## Phase 3: Technology Stack Selection

### Cranelift FFI Capabilities (VALIDATED)

Cranelift supports external function calls via:
1. **External Function Declarations**: `Module::declare_function()` with `Linkage::Import`
2. **Function References**: `FuncRef` for imported C functions
3. **Call Instructions**: `builder.ins().call()` and `builder.ins().call_indirect()`
4. **ABI Support**: SystemV calling convention (standard C ABI on Linux/macOS)

### C Library Bindings

Use Rust's libc crate for standard C functions:
```rust
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

// File I/O
extern "C" {
    fn fopen(path: *const c_char, mode: *const c_char) -> *mut c_void;
    fn fread(ptr: *mut c_void, size: usize, count: usize, stream: *mut c_void) -> usize;
    fn fwrite(ptr: *const c_void, size: usize, count: usize, stream: *mut c_void) -> usize;
    fn fclose(stream: *mut c_void) -> c_int;
    fn remove(path: *const c_char) -> c_int;
}

// System calls
extern "C" {
    fn system(command: *const c_char) -> c_int;
}
```

## Phase 4: Detailed Design

### 4.1 SSA IR Extensions

Add new instructions to `frontend/src/ssa.rs`:

```rust
pub enum SSAInstruction {
    // ... existing instructions ...

    /// FFI call to external C function
    FFICall {
        dest: SmallVec<[Register; 4]>,  // Return values
        function: String,                // C function name
        args: SmallVec<[Register; 4]>,  // Arguments
    },

    /// File operations (ANS Forth compliant)
    FileOpen {
        dest_fileid: Register,  // File handle
        dest_ior: Register,     // I/O result (0 = success)
        path_addr: Register,    // String address
        path_len: Register,     // String length
        mode: Register,         // File access mode (r/o, w/o, r/w)
    },

    FileRead {
        dest_bytes: Register,   // Bytes read
        dest_ior: Register,     // I/O result
        buffer: Register,       // Buffer address
        count: Register,        // Max bytes to read
        fileid: Register,       // File handle
    },

    FileWrite {
        dest_ior: Register,     // I/O result
        buffer: Register,       // Buffer address
        count: Register,        // Bytes to write
        fileid: Register,       // File handle
    },

    FileClose {
        dest_ior: Register,     // I/O result
        fileid: Register,       // File handle
    },

    FileDelete {
        dest_ior: Register,     // I/O result
        path_addr: Register,    // String address
        path_len: Register,     // String length
    },
}
```

### 4.2 FFI Registry (backend/src/cranelift/ffi.rs)

```rust
use cranelift_codegen::ir::{types, AbiParam, Signature, ExternalName};
use cranelift_codegen::isa::CallConv;
use cranelift_module::{Module, Linkage, FuncId};
use std::collections::HashMap;

/// FFI function metadata
#[derive(Debug, Clone)]
pub struct FFISignature {
    pub name: String,
    pub params: Vec<types::Type>,
    pub returns: Vec<types::Type>,
}

/// Registry of external C functions
pub struct FFIRegistry {
    /// Map of function names to their Cranelift function IDs
    functions: HashMap<String, FuncId>,
    /// Function signatures
    signatures: HashMap<String, FFISignature>,
}

impl FFIRegistry {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            signatures: HashMap::new(),
        }
    }

    /// Register standard libc functions
    pub fn register_libc_functions<M: Module>(&mut self, module: &mut M) -> Result<()> {
        // FILE* fopen(const char* path, const char* mode)
        self.register_function(module, FFISignature {
            name: "fopen".to_string(),
            params: vec![types::I64, types::I64],  // Two pointers
            returns: vec![types::I64],             // FILE* pointer
        })?;

        // size_t fread(void* ptr, size_t size, size_t count, FILE* stream)
        self.register_function(module, FFISignature {
            name: "fread".to_string(),
            params: vec![types::I64, types::I64, types::I64, types::I64],
            returns: vec![types::I64],  // size_t (bytes read)
        })?;

        // size_t fwrite(const void* ptr, size_t size, size_t count, FILE* stream)
        self.register_function(module, FFISignature {
            name: "fwrite".to_string(),
            params: vec![types::I64, types::I64, types::I64, types::I64],
            returns: vec![types::I64],  // size_t (bytes written)
        })?;

        // int fclose(FILE* stream)
        self.register_function(module, FFISignature {
            name: "fclose".to_string(),
            params: vec![types::I64],   // FILE* pointer
            returns: vec![types::I32],  // int (0 = success)
        })?;

        // int remove(const char* path)
        self.register_function(module, FFISignature {
            name: "remove".to_string(),
            params: vec![types::I64],   // const char* path
            returns: vec![types::I32],  // int (0 = success)
        })?;

        // int system(const char* command)
        self.register_function(module, FFISignature {
            name: "system".to_string(),
            params: vec![types::I64],   // const char* command
            returns: vec![types::I32],  // int (exit code)
        })?;

        Ok(())
    }

    /// Register an external function
    fn register_function<M: Module>(&mut self, module: &mut M, sig: FFISignature) -> Result<()> {
        let mut cranelift_sig = Signature::new(CallConv::SystemV);

        for param_ty in &sig.params {
            cranelift_sig.params.push(AbiParam::new(*param_ty));
        }

        for return_ty in &sig.returns {
            cranelift_sig.returns.push(AbiParam::new(*return_ty));
        }

        let func_id = module.declare_function(&sig.name, Linkage::Import, &cranelift_sig)
            .map_err(|e| BackendError::FFI(format!("Failed to declare {}: {}", sig.name, e)))?;

        self.functions.insert(sig.name.clone(), func_id);
        self.signatures.insert(sig.name.clone(), sig);

        Ok(())
    }

    /// Get function ID for a registered C function
    pub fn get_function(&self, name: &str) -> Option<FuncId> {
        self.functions.get(name).copied()
    }
}
```

### 4.3 Semantic Analyzer Updates

Add file I/O words to builtins in `frontend/src/semantic.rs`:

```rust
// Add to builtin words list:
for word in &[
    // ... existing builtins ...

    // File I/O (ANS Forth)
    "create-file", "open-file", "close-file",
    "read-file", "write-file", "delete-file",
    "r/o", "w/o", "r/w",

    // System
    "system",
] {
    defined_words.insert(word.to_string());
}
```

### 4.4 SSA Translation (translator.rs)

Add FFI call translation:

```rust
SSAInstruction::FileOpen { dest_fileid, dest_ior, path_addr, path_len, mode } => {
    // Get C function reference for fopen
    let fopen_ref = self.get_ffi_function("fopen")?;

    // Prepare arguments
    let path_ptr = self.get_register(*path_addr);
    let mode_str = self.convert_mode_to_cstring(*mode);  // Convert Forth mode to C mode string

    // Call fopen(path, mode)
    let call = self.builder.ins().call(fopen_ref, &[path_ptr, mode_str]);
    let file_handle = self.builder.inst_results(call)[0];

    // Check if NULL (error)
    let null_ptr = self.builder.ins().iconst(types::I64, 0);
    let is_error = self.builder.ins().icmp(IntCC::Equal, file_handle, null_ptr);

    // Set ior: 0 = success, -1 = error
    let success = self.builder.ins().iconst(types::I64, 0);
    let error = self.builder.ins().iconst(types::I64, -1);
    let ior = self.builder.ins().select(is_error, error, success);

    // Store results
    self.def_register(*dest_fileid, file_handle);
    self.def_register(*dest_ior, ior);
}
```

## Phase 5: Implementation Plan

### 5.1 Create FFI Infrastructure (2-3 days)
1. Create `backend/src/cranelift/ffi.rs`
2. Implement `FFIRegistry` with libc function registration
3. Integrate FFI registry into `CraneliftBackend`
4. Test FFI calls with simple printf example

### 5.2 Add SSA Instructions (1 day)
1. Extend `SSAInstruction` enum with FFI and File operations
2. Update SSA converter to generate new instructions
3. Add serialization/display for new instructions

### 5.3 Implement File I/O Translation (2-3 days)
1. Add file operation translation in `translator.rs`
2. Implement helper functions for:
   - String null-termination (C strings)
   - Mode conversion (Forth r/o → C "r")
   - Error code mapping
3. Test each file operation individually

### 5.4 Add Forth File I/O Words (1 day)
1. Update semantic analyzer with file I/O builtins
2. Add SSA generation for file words
3. Document stack effects and usage

## Phase 6: Testing Strategy

### 6.1 Unit Tests
```forth
\ Test file creation
: test-create-file
  s" /tmp/test.txt" w/o create-file
  if
    drop ." Failed to create file" cr
    1
  else
    close-file drop
    ." File created successfully" cr
    0
  then
;

\ Test file write
: test-write-file
  s" /tmp/test.txt" w/o create-file
  if drop 1 exit then
  >r
  s" Hello FastForth!" r@ write-file
  r> close-file or
;

\ Test file read
: test-read-file
  create read-buf 256 allot
  s" /tmp/test.txt" r/o open-file
  if drop 1 exit then
  >r
  read-buf 256 r@ read-file
  if r> close-file drop 1 exit then
  . ." bytes read" cr
  r> close-file
;
```

### 6.2 Integration Tests
- Test file I/O with Llama CLI requirements
- Verify system() calls work correctly
- Test error handling (missing files, permission errors)
- Benchmark performance impact

### 6.3 Compliance Testing
- Run ANS Forth file I/O test suite
- Verify stack effects match specification
- Test edge cases (empty files, large files)

## Phase 7: Documentation

### 7.1 User Guide
- File I/O word reference
- Examples and usage patterns
- Migration from gforth

### 7.2 Developer Guide
- FFI extension mechanism
- Adding new C functions
- Debugging FFI calls

## Phase 8: Deployment

1. Build FastForth with FFI support
2. Run comprehensive test suite
3. Install updated binary globally: `cargo install --path .`
4. Verify Llama CLI compatibility
5. Update README and changelog

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| File I/O operations work | 100% | All ANS Forth file words functional |
| No performance regression | <5% | JIT compile time still ~50ms |
| ANS Forth compliance | 100% | Pass ANS file I/O test suite |
| Llama CLI ready | 100% | Can run Llama CLI with FastForth |
| Code quality | 90%+ | Passes clippy, no unsafe code |

## Risk Analysis

### Low Risk
- FFI infrastructure (well-supported by Cranelift)
- File I/O operations (standard POSIX)

### Medium Risk
- String handling (null termination, encoding)
- Error propagation (mapping C errors to Forth ior codes)

### Mitigation
- Comprehensive testing of edge cases
- Clear error messages and logging
- Fallback to gforth for unsupported operations

## Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Requirements | ✅ Complete | None |
| Phase 2: Architecture | ✅ Complete | Phase 1 |
| Phase 3: Technology | ✅ Complete | Phase 2 |
| Phase 4: Detailed Design | ✅ Complete | Phase 3 |
| Phase 5: Implementation | 6-8 days | Phase 4 |
| Phase 6: Testing | 2-3 days | Phase 5 |
| Phase 7: Documentation | 1-2 days | Phase 6 |
| Phase 8: Deployment | 1 day | Phase 7 |

**Total: 10-14 days**

## Appendix A: ANS Forth File I/O Word Set

```forth
\ File access modes
r/o  ( -- fam )       \ Read-only = 0
w/o  ( -- fam )       \ Write-only = 1
r/w  ( -- fam )       \ Read-write = 2

\ File operations
create-file  ( c-addr u fam -- fileid ior )
open-file    ( c-addr u fam -- fileid ior )
close-file   ( fileid -- ior )
read-file    ( c-addr u fileid -- u ior )
write-file   ( c-addr u fileid -- ior )
delete-file  ( c-addr u -- ior )
file-size    ( fileid -- ud ior )
file-position ( fileid -- ud ior )
reposition-file ( ud fileid -- ior )
```

## Appendix B: C Function Signatures

```c
// File I/O
FILE* fopen(const char* path, const char* mode);
size_t fread(void* ptr, size_t size, size_t count, FILE* stream);
size_t fwrite(const void* ptr, size_t size, size_t count, FILE* stream);
int fclose(FILE* stream);
int remove(const char* path);

// System
int system(const char* command);
```
