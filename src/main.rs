//! Fast Forth - Main binary
//!
//! A high-performance Forth compiler with LLVM backend

use fastforth::{Compiler, CompilationMode, OptimizationLevel};
use clap::{Parser, Subcommand};
use colored::Colorize;
use rustyline::DefaultEditor;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "fastforth")]
#[command(about = "Fast Forth - High-performance Forth compiler with LLVM backend", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Optimization level (0-3)
    #[arg(short = 'O', long, default_value = "2", global = true)]
    opt_level: u8,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Forth source file
    Compile {
        /// Input Forth source file
        input: PathBuf,

        /// Output file (default: based on input name)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compilation mode (aot or jit)
        #[arg(short, long, default_value = "aot")]
        mode: String,
    },

    /// Run Forth code in JIT mode
    Run {
        /// Forth source file to run
        input: PathBuf,
    },

    /// Execute Forth code from command line
    Execute {
        /// Forth code to execute
        code: String,
    },

    /// Start interactive REPL
    Repl,

    /// Display compiler information
    Info,
}

fn main() {
    let cli = Cli::parse();

    // Initialize tracing if verbose
    #[cfg(feature = "verbose")]
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    // Convert optimization level
    let opt_level = match cli.opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Basic,
        2 => OptimizationLevel::Standard,
        _ => OptimizationLevel::Aggressive,
    };

    let compiler = Compiler::new(opt_level);

    match &cli.command {
        Some(Commands::Compile { input, output, mode }) => {
            let compilation_mode = match mode.as_str() {
                "aot" => CompilationMode::AOT,
                "jit" => CompilationMode::JIT,
                _ => {
                    eprintln!("{}: Invalid mode '{}', use 'aot' or 'jit'", "Error".red(), mode);
                    process::exit(1);
                }
            };

            match compiler.compile_file(input, compilation_mode) {
                Ok(result) => {
                    println!("{}", "✓ Compilation successful".green().bold());
                    println!("  Mode: {:?}", result.mode);
                    println!("  Time: {}ms", result.compile_time_ms);
                    println!("  Definitions: {}", result.stats.definitions_count);
                    println!(
                        "  Optimization: {:.1}% reduction",
                        result.stats.optimization_savings() * 100.0
                    );

                    if let Some(output_path) = &result.output_path {
                        println!("  Output: {}", output_path);
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Compilation failed".red().bold(), e);
                    process::exit(1);
                }
            }
        }

        Some(Commands::Run { input }) => {
            match compiler.compile_file(input, CompilationMode::JIT) {
                Ok(result) => {
                    println!("{}", "✓ Execution complete".green().bold());
                    println!("  Time: {}ms", result.compile_time_ms);
                    if let Some(jit_result) = result.jit_result {
                        println!("  Result: {}", jit_result);
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Execution failed".red().bold(), e);
                    process::exit(1);
                }
            }
        }

        Some(Commands::Execute { code }) => {
            match compiler.compile_string(code, CompilationMode::JIT) {
                Ok(result) => {
                    if let Some(jit_result) = result.jit_result {
                        println!("{}", jit_result);
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Error".red(), e);
                    process::exit(1);
                }
            }
        }

        Some(Commands::Repl) => {
            run_repl(compiler);
        }

        Some(Commands::Info) => {
            print_info(&compiler);
        }

        None => {
            // Default: start REPL
            run_repl(compiler);
        }
    }
}

fn run_repl(compiler: Compiler) {
    println!("{}", "Fast Forth REPL".cyan().bold());
    println!("Optimization: {:?}", compiler.optimization_level());
    println!("Type {} to exit\n", "'.quit'".yellow());

    let mut rl = DefaultEditor::new().unwrap();
    let mut line_number = 1;

    loop {
        let prompt = format!("{}> ", line_number.to_string().cyan());
        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                if trimmed == ".quit" || trimmed == ".exit" {
                    break;
                }

                if trimmed == ".help" {
                    print_repl_help();
                    continue;
                }

                if trimmed.starts_with(".load ") {
                    let path = trimmed.trim_start_matches(".load ").trim();
                    match compiler.compile_file(&PathBuf::from(path), CompilationMode::JIT) {
                        Ok(_) => println!("{}", "✓ File loaded".green()),
                        Err(e) => eprintln!("{}: {}", "Error".red(), e),
                    }
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(&line);

                // Try to compile and execute
                match compiler.compile_string(trimmed, CompilationMode::JIT) {
                    Ok(result) => {
                        if let Some(jit_result) = result.jit_result {
                            println!("{} {}", "=>".green(), jit_result);
                        } else {
                            println!("{}", "ok".green());
                        }

                        if result.stats.definitions_count > 0 {
                            println!(
                                "{} {} definitions",
                                "✓".green(),
                                result.stats.definitions_count
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Error".red(), e);
                    }
                }

                line_number += 1;
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                break;
            }
        }
    }

    println!("\n{}", "Goodbye!".cyan());
}

fn print_repl_help() {
    println!("\n{}", "REPL Commands:".cyan().bold());
    println!("  {}        - Show this help", ".help".yellow());
    println!("  {}        - Quit the REPL", ".quit".yellow());
    println!("  {} <file> - Load and execute a Forth file", ".load".yellow());
    println!("\n{}", "Forth Basics:".cyan().bold());
    println!("  {}       - Push 42 on stack", "42".yellow());
    println!("  {}        - Duplicate top of stack", "dup".yellow());
    println!("  {}       - Drop top of stack", "drop".yellow());
    println!("  {}       - Swap top two items", "swap".yellow());
    println!("  {}    - Add, subtract, multiply, divide", "+ - * /".yellow());
    println!("  {}    - Define a new word", ": double 2 * ;".yellow());
    println!();
}

fn print_info(compiler: &Compiler) {
    println!("\n{}", "Fast Forth Compiler".cyan().bold());
    println!("{}", "=".repeat(50));
    println!();

    println!("{}", "Components:".green().bold());
    println!("  ✓ Frontend: Parsing, Type Inference, SSA Conversion");
    println!("  ✓ Optimizer: 5 optimization passes");
    println!("  • Backend: LLVM IR generation (in progress)");
    println!("  • Runtime: C runtime library");
    println!();

    println!("{}", "Optimization Passes:".green().bold());
    println!("  1. Stack Caching (TOS/NOS/3OS in registers)");
    println!("  2. Superinstructions (pattern fusion)");
    println!("  3. Constant Folding (compile-time evaluation)");
    println!("  4. Dead Code Elimination");
    println!("  5. Inlining (with stack effect analysis)");
    println!();

    println!("{}", "Current Configuration:".green().bold());
    println!("  Optimization Level: {:?}", compiler.optimization_level());
    println!();

    println!("{}", "Supported Modes:".green().bold());
    println!("  • AOT: Ahead-of-time compilation to native executable");
    println!("  • JIT: Just-in-time compilation and execution");
    println!();
}
