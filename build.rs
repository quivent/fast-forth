// Build script for Fast Forth
// - Compiles C runtime including concurrency primitives
// - Embeds entire source code into binary

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // ========================================================================
    // Part 1: Compile C Runtime
    // ========================================================================

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

    println!("cargo:rustc-link-lib=pthread");

    // Rebuild if any C files change
    println!("cargo:rerun-if-changed=runtime/");
    println!("cargo:rerun-if-changed=minimal_forth/");

    // ========================================================================
    // Part 2: Embed Entire Source Code
    // ========================================================================

    println!("cargo:warning=Embedding Fast Forth source code...");

    let archive_path = out_dir.join("embedded_source.tar.gz");

    let status = Command::new("tar")
        .args(&[
            "czf",
            archive_path.to_str().unwrap(),
            "--exclude=target",
            "--exclude=.git",
            "--exclude=build",
            "--exclude=*.o",
            "--exclude=*.a",
            "--exclude=*.so",
            "--exclude=*.dylib",
            "--exclude=.DS_Store",
            ".",
        ])
        .status();

    match status {
        Ok(status) if status.success() => {
            if let Ok(metadata) = std::fs::metadata(&archive_path) {
                let size_kb = metadata.len() / 1024;
                println!("cargo:warning=✓ Embedded source: {} KB", size_kb);
                println!("cargo:rustc-env=EMBEDDED_SOURCE_SIZE={}", metadata.len());
            }
        }
        _ => {
            println!("cargo:warning=Warning: Could not embed source code (tar failed)");
        }
    }

    println!(
        "cargo:rustc-env=EMBEDDED_SOURCE_PATH={}",
        archive_path.display()
    );
}
