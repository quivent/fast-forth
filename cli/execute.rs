// execute.rs - Runtime execution using Cranelift JIT

use anyhow::{Context, Result};
use backend::cranelift::{CraneliftBackend, CraneliftSettings};
use fastforth_frontend::{parse_program, convert_to_ssa};
use std::path::Path;

/// Execute a Forth program with JIT compilation
pub fn execute_program(source: &str, verbose: bool) -> Result<i64> {
    eprintln!("[DEBUG] execute_program called, source len={}, verbose={}", source.len(), verbose);

    // Phase 1: Parse
    if verbose {
        println!("  Parsing...");
    }
    eprintln!("[DEBUG] Starting parse...");
    let program = match parse_program(source) {
        Ok(p) => {
            eprintln!("[DEBUG] Parse successful!");
            p
        }
        Err(e) => {
            eprintln!("[DEBUG] Parse failed: {:?}", e);
            return Err(anyhow::anyhow!("Failed to parse: {}", e));
        }
    };

    if verbose {
        println!("  Parsed {} definitions", program.definitions.len());
    }

    // Phase 2: Convert to SSA
    if verbose {
        println!("  Converting to SSA...");
    }
    eprintln!("[DEBUG] Converting to SSA...");
    let ssa_functions = match convert_to_ssa(&program) {
        Ok(funcs) => {
            eprintln!("[DEBUG] SSA conversion successful, got {} functions", funcs.len());
            funcs
        }
        Err(e) => {
            eprintln!("[DEBUG] SSA conversion failed: {:?}", e);
            return Err(anyhow::anyhow!("Failed to convert to SSA: {}", e));
        }
    };

    if verbose {
        println!("  Generated {} SSA functions", ssa_functions.len());
    }

    // Phase 3: JIT compile with Cranelift
    if verbose {
        println!("  JIT compiling...");
    }

    let settings = CraneliftSettings {
        opt_level: 1,
        debug_info: false,
        target_triple: None,
    };

    let mut backend = CraneliftBackend::new(settings)
        .context("Failed to initialize Cranelift backend")?;

    // Two-pass compilation for function calls and recursion

    // Prepare (name, function) pairs using actual function names from SSA
    let functions_with_names: Vec<(String, &_)> = ssa_functions.iter()
        .map(|func| (func.name.clone(), func))
        .collect();

    // Pass 1: Declare all functions
    eprintln!("[DEBUG] Pass 1: Declaring {} functions", functions_with_names.len());
    backend.declare_all_functions(&functions_with_names)
        .context("Failed to declare functions")?;

    // Pass 2: Compile all function bodies (can now reference each other)
    eprintln!("[DEBUG] Pass 2: Compiling function bodies");
    for (name, func) in &functions_with_names {
        eprintln!("[DEBUG] Compiling function: {}", name);
        eprintln!("[DEBUG]   Entry block: {:?}", func.entry_block);
        eprintln!("[DEBUG]   Parameters: {:?}", func.parameters);
        eprintln!("[DEBUG]   Blocks: {}", func.blocks.len());
        for (i, block) in func.blocks.iter().enumerate() {
            eprintln!("[DEBUG]     Block {}: {:?} with {} instructions", i, block.id, block.instructions.len());
            for inst in &block.instructions {
                eprintln!("[DEBUG]       {:?}", inst);
            }
        }
        match backend.compile_function(func, name) {
            Ok(_) => {
                eprintln!("[DEBUG] Successfully compiled {}", name);
            }
            Err(e) => {
                eprintln!("[DEBUG] Compilation error for {}: {:?}", name, e);
                return Err(anyhow::anyhow!("Failed to compile function {}: {}", name, e));
            }
        }
    }

    // Finalize all functions (must be done after all are compiled for recursion to work)
    eprintln!("[DEBUG] Finalizing all functions");
    backend.finalize_all()
        .context("Failed to finalize functions")?;

    if verbose {
        println!("  Compiled {} functions", ssa_functions.len());
    }

    // Phase 4: Execute
    if verbose {
        println!("  Executing...");
    }

    // Get the last compiled function (which will be :main if top-level code exists)
    if ssa_functions.is_empty() {
        eprintln!("[DEBUG] No functions to execute (empty program)");
        return Ok(0);
    }

    // Execute the last function (usually :main if top-level code exists)
    let func_name = &ssa_functions.last().unwrap().name;
    eprintln!("[DEBUG] Executing function: {}", func_name);

    let main_func_ptr = backend.get_function(func_name)
        .ok_or_else(|| anyhow::anyhow!("Failed to get compiled function"))?;

    eprintln!("[DEBUG] Calling JIT function at {:?}", main_func_ptr);

    // Create Forth data stack (256 cells = 2KB)
    const STACK_SIZE: usize = 256;
    let mut stack = vec![0i64; STACK_SIZE];

    // Stack grows upward: stack_ptr points to next free cell
    // Start at bottom of stack
    let stack_base = stack.as_mut_ptr();
    let stack_ptr = stack_base;

    // Cast to function pointer: fn(*mut i64) -> *mut i64
    type ForthFn = unsafe extern "C" fn(*mut i64) -> *mut i64;
    let forth_fn: ForthFn = unsafe { std::mem::transmute(main_func_ptr) };

    // Call the compiled function
    let result_ptr = unsafe { forth_fn(stack_ptr) };

    // Calculate how many items are on the stack
    let stack_depth = unsafe {
        result_ptr.offset_from(stack_base) as usize
    };

    eprintln!("[DEBUG] Execution complete, stack depth: {}", stack_depth);

    // Read top of stack as result (if any)
    let result = if stack_depth > 0 {
        unsafe { *result_ptr.offset(-1) }
    } else {
        0
    };

    eprintln!("[DEBUG] Top of stack: {}", result);

    Ok(result)
}

/// Execute a Forth file
pub fn execute_file(path: &Path, verbose: bool) -> Result<i64> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    execute_program(&source, verbose)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple() {
        let result = execute_program(": double 2 * ; 5 double", false);
        assert!(result.is_ok(), "Failed to execute: {:?}", result);
        assert_eq!(result.unwrap(), 10, "Expected 5 * 2 = 10");
    }

    #[test]
    fn test_execute_toplevel_constant() {
        let result = execute_program("42", true);
        assert!(result.is_ok(), "Failed to execute top-level constant: {:?}", result);
        assert_eq!(result.unwrap(), 42, "Top-level constant should return 42");
    }

    #[test]
    fn test_execute_definition_only() {
        let result = execute_program(": answer 42 ;", true);
        assert!(result.is_ok(), "Failed to compile definition: {:?}", result);
        // Definition only, no execution, should return 0
        assert_eq!(result.unwrap(), 0);
    }
}
