// execute.rs - Runtime execution using Cranelift JIT

use anyhow::{Context, Result};
use backend::cranelift::{CraneliftBackend, CraneliftSettings};
use fastforth_frontend::{parse_program, convert_to_ssa};
use std::path::Path;

/// Execute a Forth program with JIT compilation
pub fn execute_program(source: &str, verbose: bool) -> Result<i64> {
    // Phase 1: Parse
    if verbose {
        println!("  Parsing...");
    }
    let program = parse_program(source)
        .map_err(|e| anyhow::anyhow!("Failed to parse: {}", e))?;

    if verbose {
        println!("  Parsed {} definitions", program.definitions.len());
    }

    // Phase 2: Convert to SSA
    if verbose {
        println!("  Converting to SSA...");
    }
    let ssa_functions = convert_to_ssa(&program)
        .map_err(|e| anyhow::anyhow!("Failed to convert to SSA: {}", e))?;

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
    backend.declare_all_functions(&functions_with_names)
        .context("Failed to declare functions")?;

    // Pass 2: Compile all function bodies (can now reference each other)
    for (name, func) in &functions_with_names {
        backend.compile_function(func, name)
            .with_context(|| format!("Failed to compile function {}", name))?;
    }

    // Finalize all functions (must be done after all are compiled for recursion to work)
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
        return Ok(0);
    }

    // Execute the last function (usually :main if top-level code exists)
    let func_name = &ssa_functions.last().unwrap().name;
    let return_count = 1; // All Forth functions return 1 value

    let main_func_ptr = backend.get_function(func_name)
        .ok_or_else(|| anyhow::anyhow!("Failed to get compiled function"))?;

    // Call function based on its return count
    let result = match return_count {
        0 => {
            // Function returns nothing
            type ForthFn = unsafe extern "C" fn();
            let forth_fn: ForthFn = unsafe { std::mem::transmute(main_func_ptr) };
            unsafe { forth_fn() };
            0
        }
        1 => {
            // Function returns one i64 value
            type ForthFn = unsafe extern "C" fn() -> i64;
            let forth_fn: ForthFn = unsafe { std::mem::transmute(main_func_ptr) };
            unsafe { forth_fn() }
        }
        _ => {
            return Err(anyhow::anyhow!("Functions with multiple return values not yet supported in execute harness"));
        }
    };

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
