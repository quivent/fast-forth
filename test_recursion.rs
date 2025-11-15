// Test recursion compilation
use anyhow::Result;
use backend::cranelift::{CraneliftBackend, CraneliftSettings};
use fastforth_frontend::{parse_program, convert_to_ssa};

fn main() -> Result<()> {
    // Test simple recursion
    let source = ": double 2 * ; : quad 2 double ; 5 quad";

    println!("Testing: {}", source);

    // Parse
    let program = parse_program(source)?;
    println!("✓ Parse successful");

    // Convert to SSA
    let functions = convert_to_ssa(&program)?;
    println!("✓ SSA conversion successful, {} functions", functions.len());

    // Create backend
    let settings = CraneliftSettings::default();
    let mut backend = CraneliftBackend::new(settings)?;
    println!("✓ Backend created");

    // Two-pass compilation
    let funcs_with_names: Vec<(String, &_)> = functions.iter()
        .map(|f| (f.name.clone(), f))
        .collect();

    // Pass 1: Declare
    backend.declare_all_functions(&funcs_with_names)?;
    println!("✓ Pass 1: Declared {} functions", funcs_with_names.len());

    // Pass 2: Compile
    for (name, func) in &funcs_with_names {
        backend.compile_function(func, name)?;
        println!("✓ Compiled function: {}", name);
    }

    // Pass 3: Finalize
    backend.finalize_all()?;
    println!("✓ Pass 3: Finalized all functions");

    // Get function pointer
    let main_name = &functions.last().unwrap().name;
    let func_ptr = backend.get_function(main_name)
        .ok_or_else(|| anyhow::anyhow!("Failed to get function pointer"))?;
    println!("✓ Got function pointer for '{}'", main_name);

    // Execute
    let mut stack = vec![0i64; 256];
    type ForthFn = unsafe extern "C" fn(*mut i64) -> *mut i64;
    let forth_fn: ForthFn = unsafe { std::mem::transmute(func_ptr) };

    let stack_ptr = stack.as_mut_ptr();
    let result_ptr = unsafe { forth_fn(stack_ptr) };

    let stack_depth = unsafe { result_ptr.offset_from(stack_ptr) };
    let result = if stack_depth > 0 {
        unsafe { *result_ptr.offset(-1) }
    } else {
        0
    };

    println!("✓ Execution complete");
    println!("Result: {} (expected: 20)", result);

    if result == 20 {
        println!("\n🎉 SUCCESS! Recursion and function calls work!");
    } else {
        println!("\n⚠️  Result mismatch");
    }

    Ok(())
}
