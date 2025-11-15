// Extract embedded source code from Fast Forth binary
// This allows users to audit and rebuild from source

use std::fs::File;
use std::io::Write;
use std::path::Path;

// This will be populated by build.rs with embedded source
static EMBEDDED_SOURCE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/embedded_source.tar.gz"));

pub fn extract_source(output_path: &str) -> std::io::Result<()> {
    println!("════════════════════════════════════════════════════════════");
    println!("  Extracting Fast Forth Source Code");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("This binary contains the complete Fast Forth source code.");
    println!("Extracting to: {}", output_path);
    println!();

    let path = Path::new(output_path);
    let mut file = File::create(path)?;
    file.write_all(EMBEDDED_SOURCE)?;

    let size_kb = EMBEDDED_SOURCE.len() / 1024;
    println!("✓ Extracted {} KB to: {}", size_kb, output_path);
    println!();
    println!("════════════════════════════════════════════════════════════");
    println!("  Rebuild Instructions");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("Option 1: Minimal Compiler (30s, 30-50% of C performance)");
    println!("  tar xzf {}", output_path);
    println!("  cd fast-forth");
    println!("  make -C minimal_forth");
    println!("  ./minimal_forth/forth");
    println!();
    println!("Option 2: Full Optimizations (5-25 min, 85-110% of C performance)");
    println!("  tar xzf {}", output_path);
    println!("  cd fast-forth");
    println!("  ./scripts/install-rust.sh");
    println!("  cargo build --release");
    println!("  ./target/release/fastforth");
    println!();
    println!("Option 3: Pre-compiled Binary (instant)");
    println!("  Just use this binary: ./fastforth");
    println!();

    Ok(())
}

fn main() {
    if let Err(e) = extract_source("fastforth-source.tar.gz") {
        eprintln!("Error extracting source: {}", e);
        std::process::exit(1);
    }
}
