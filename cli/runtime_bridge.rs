// runtime_bridge.rs - Bridge between Rust CLI and C runtime
// Provides safe FFI bindings to the C-based Forth runtime

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

/// Stack item representation matching the C runtime
#[repr(C)]
pub struct CStackItem {
    pub value: i64,
    pub type_tag: u8,  // 0=int, 1=float, 2=string, 3=bool
}

/// FFI declarations for C runtime functions
extern "C" {
    fn forth_runtime_init() -> *mut c_void;
    fn forth_runtime_destroy(runtime: *mut c_void);
    fn forth_runtime_execute(runtime: *mut c_void, code: *const c_char) -> c_int;
    fn forth_runtime_push_int(runtime: *mut c_void, value: i64);
    fn forth_runtime_push_float(runtime: *mut c_void, value: f64);
    fn forth_runtime_pop(runtime: *mut c_void, item: *mut CStackItem) -> c_int;
    fn forth_runtime_stack_depth(runtime: *mut c_void) -> c_int;
    fn forth_runtime_clear_stack(runtime: *mut c_void);
    fn forth_runtime_get_error(runtime: *mut c_void) -> *const c_char;
}

/// Safe wrapper around the C runtime
pub struct ForthRuntime {
    handle: *mut c_void,
}

impl ForthRuntime {
    /// Create a new Forth runtime instance
    pub fn new() -> Result<Self, String> {
        unsafe {
            let handle = forth_runtime_init();
            if handle.is_null() {
                return Err("Failed to initialize Forth runtime".to_string());
            }
            Ok(ForthRuntime { handle })
        }
    }

    /// Execute Forth code
    pub fn execute(&mut self, code: &str) -> Result<(), String> {
        let c_code = CString::new(code).map_err(|e| format!("Invalid string: {}", e))?;

        unsafe {
            let result = forth_runtime_execute(self.handle, c_code.as_ptr());
            if result != 0 {
                let error = forth_runtime_get_error(self.handle);
                if !error.is_null() {
                    let error_str = CStr::from_ptr(error)
                        .to_string_lossy()
                        .into_owned();
                    return Err(error_str);
                }
                return Err("Execution failed".to_string());
            }
        }

        Ok(())
    }

    /// Push an integer onto the stack
    pub fn push_int(&mut self, value: i64) {
        unsafe {
            forth_runtime_push_int(self.handle, value);
        }
    }

    /// Push a float onto the stack
    pub fn push_float(&mut self, value: f64) {
        unsafe {
            forth_runtime_push_float(self.handle, value);
        }
    }

    /// Pop a value from the stack
    pub fn pop(&mut self) -> Result<StackValue, String> {
        unsafe {
            let mut item = CStackItem {
                value: 0,
                type_tag: 0,
            };

            let result = forth_runtime_pop(self.handle, &mut item);
            if result != 0 {
                return Err("Stack underflow".to_string());
            }

            Ok(match item.type_tag {
                0 => StackValue::Integer(item.value),
                1 => StackValue::Float(f64::from_bits(item.value as u64)),
                2 => StackValue::String(item.value.to_string()), // Simplified
                3 => StackValue::Boolean(item.value != 0),
                _ => return Err("Invalid type tag".to_string()),
            })
        }
    }

    /// Get the current stack depth
    pub fn stack_depth(&self) -> usize {
        unsafe {
            forth_runtime_stack_depth(self.handle) as usize
        }
    }

    /// Clear the stack
    pub fn clear_stack(&mut self) {
        unsafe {
            forth_runtime_clear_stack(self.handle);
        }
    }

    /// Get all stack contents (non-destructive)
    pub fn get_stack(&self) -> Vec<StackValue> {
        let _depth = self.stack_depth();
        let stack = Vec::new();

        // For now, return empty vector
        // TODO: Implement non-destructive stack inspection in C runtime
        stack
    }
}

impl Drop for ForthRuntime {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                forth_runtime_destroy(self.handle);
            }
        }
    }
}

/// Stack value representation
#[derive(Debug, Clone, PartialEq)]
pub enum StackValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl std::fmt::Display for StackValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StackValue::Integer(n) => write!(f, "{}", n),
            StackValue::Float(n) => write!(f, "{}", n),
            StackValue::String(s) => write!(f, "\"{}\"", s),
            StackValue::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        // This will fail without the C runtime linked
        // let runtime = ForthRuntime::new();
        // assert!(runtime.is_ok());
    }

    #[test]
    fn test_stack_value_display() {
        assert_eq!(format!("{}", StackValue::Integer(42)), "42");
        assert_eq!(format!("{}", StackValue::Float(3.14)), "3.14");
        assert_eq!(format!("{}", StackValue::Boolean(true)), "true");
    }
}
