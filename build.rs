// Build script for Fast Forth
// Compiles C runtime including concurrency primitives

fn main() {
    // Compile C runtime with pthread support
    cc::Build::new()
        .file("runtime/forth_runtime.c")
        .file("runtime/memory.c")
        .file("runtime/ffi.c")
        .file("runtime/bootstrap.c")
        .file("runtime/concurrency.c")
        .include("runtime")
        .flag_if_supported("-pthread")
        .flag_if_supported("-O3")
        .flag_if_supported("-march=native")
        .flag_if_supported("-std=c11")
        .warnings(true)
        .compile("forthruntime");

    // Link pthread library
    println!("cargo:rustc-link-lib=pthread");

    // Rebuild if any C files change
    println!("cargo:rerun-if-changed=runtime/forth_runtime.c");
    println!("cargo:rerun-if-changed=runtime/forth_runtime.h");
    println!("cargo:rerun-if-changed=runtime/memory.c");
    println!("cargo:rerun-if-changed=runtime/ffi.c");
    println!("cargo:rerun-if-changed=runtime/bootstrap.c");
    println!("cargo:rerun-if-changed=runtime/concurrency.c");
    println!("cargo:rerun-if-changed=runtime/concurrency.h");
}
