//! Fast Forth - Main binary
//!
//! A high-performance Forth compiler with LLVM backend

use fastforth::{Compiler, CompilationMode, OptimizationLevel};
#[cfg(feature = "inference")]
use fastforth::inference::InferenceAPI;
#[cfg(feature = "server")]
use fastforth::server::{VerificationServer, ServerConfig};
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

        /// Output format for errors (human, json, json-pretty, plain)
        #[arg(long, default_value = "human")]
        error_format: String,

        /// Agent mode - JSON output only, compact diagnostics
        #[arg(long)]
        agent_mode: bool,

        /// Verify only - type check without code generation
        #[arg(long)]
        verify_only: bool,

        /// Include auto-fix suggestions in errors
        #[arg(long)]
        suggest_fixes: bool,
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

    /// Infer stack effect from code
    Infer {
        /// Forth code to analyze
        code: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Verify code matches expected stack effect
    VerifyEffect {
        /// Forth code to verify
        code: String,

        /// Expected stack effect (e.g., "( n -- n² )")
        effect: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Start verification server
    #[cfg(feature = "server")]
    Server {
        /// Server port
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Server host
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Specification commands
    Spec {
        #[command(subcommand)]
        command: SpecCommands,
    },

    /// Generate Forth code from specification
    Generate {
        /// Specification file (JSON)
        #[arg(long, value_name = "FILE")]
        from_spec: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Skip test generation
        #[arg(long)]
        no_tests: bool,

        /// Skip provenance metadata
        #[arg(long)]
        no_provenance: bool,
    },

    /// Generate tests for a word
    GenerateTests {
        /// Specification file (JSON)
        spec: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Number of random property tests to generate
        #[arg(long, default_value = "5")]
        random_count: usize,
    },

    /// Extract provenance metadata from source or binary
    Provenance {
        /// Source file or binary to extract from
        input: PathBuf,

        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,

        /// Filter by agent
        #[arg(long)]
        agent: Option<String>,

        /// Filter by pattern
        #[arg(long)]
        pattern: Option<String>,

        /// Show only verified code
        #[arg(long)]
        verified_only: bool,
    },

    /// Run benchmark suite
    Benchmark {
        /// Run specific benchmark (or all if not specified)
        #[arg(long)]
        name: Option<String>,

        /// Output format (text or json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Compose two stack effects (type algebra)
    Compose {
        /// First word or effect
        first: String,

        /// Second word or effect
        second: String,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Semantic diff between two implementations
    Diff {
        /// Old implementation file
        old: PathBuf,

        /// New implementation file
        new: PathBuf,

        /// Use semantic comparison
        #[arg(long)]
        semantic: bool,

        /// Output format (human or json)
        #[arg(long, default_value = "human")]
        format: String,
    },
}

#[derive(Subcommand)]
enum SpecCommands {
    /// Validate a specification file
    Validate {
        /// Specification file (JSON)
        spec: PathBuf,

        /// Use strict validation
        #[arg(long)]
        strict: bool,
    },

    /// Show specification details
    Show {
        /// Specification file (JSON)
        spec: PathBuf,
    },
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
        Some(Commands::Compile {
            input,
            output,
            mode,
            error_format,
            agent_mode,
            verify_only,
            suggest_fixes,
        }) => {
            let compilation_mode = match mode.as_str() {
                "aot" => CompilationMode::AOT,
                "jit" => CompilationMode::JIT,
                _ => {
                    eprintln!("{}: Invalid mode '{}', use 'aot' or 'jit'", "Error".red(), mode);
                    process::exit(1);
                }
            };

            // For verify-only mode, we only type-check
            if *verify_only {
                // TODO: Implement type-check only mode
                println!("{}", "Verify-only mode not yet implemented".yellow());
            }

            match compiler.compile_file(input, compilation_mode) {
                Ok(result) => {
                    // Agent mode: JSON output only
                    if *agent_mode {
                        let json_output = serde_json::json!({
                            "status": "success",
                            "mode": format!("{:?}", result.mode),
                            "compile_time_ms": result.compile_time_ms,
                            "definitions_count": result.stats.definitions_count,
                            "optimization_savings": result.stats.optimization_savings(),
                            "output_path": result.output_path,
                        });
                        println!("{}", serde_json::to_string(&json_output).unwrap());
                    } else {
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
                }
                Err(e) => {
                    if *agent_mode {
                        let json_output = serde_json::json!({
                            "status": "error",
                            "error": format!("{}", e),
                        });
                        println!("{}", serde_json::to_string(&json_output).unwrap());
                    } else {
                        eprintln!("{}: {}", "Compilation failed".red().bold(), e);
                    }
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

        #[cfg(feature = "inference")]
        Some(Commands::Infer { code, json }) => {
            let api = InferenceAPI::new();
            match api.infer(code) {
                Ok(result) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    } else {
                        println!("{}", "✓ Stack Effect Inference".green().bold());
                        println!("  Effect: {}", result.inferred_effect);
                        println!("  Depth Delta: {}", result.stack_depth_delta);
                        println!("  Operations: {}", result.operations.join(" "));
                        println!("  Latency: {:.3}ms", result.latency_ms);
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Inference failed".red().bold(), e);
                    process::exit(1);
                }
            }
        }

        #[cfg(feature = "inference")]
        Some(Commands::VerifyEffect { code, effect, json }) => {
            let api = InferenceAPI::new();
            match api.verify_effect(code, effect) {
                Ok(result) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&result).unwrap());
                    } else {
                        if result.valid {
                            println!("{}", "✓ Verification Successful".green().bold());
                        } else {
                            println!("{}", "✗ Verification Failed".red().bold());
                        }
                        println!("  Expected: {}", result.expected);
                        println!("  Inferred: {}", result.inferred);
                        println!("  Message: {}", result.message);
                        println!("  Latency: {:.3}ms", result.latency_ms);

                        if !result.valid {
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Verification failed".red().bold(), e);
                    process::exit(1);
                }
            }
        }

        #[cfg(feature = "server")]
        Some(Commands::Server { port, host }) => {
            let config = ServerConfig {
                host: host.clone(),
                port: *port,
                workers: num_cpus::get(),
            };

            let server = VerificationServer::new(config);
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = server.start().await {
                    eprintln!("{}: {}", "Server error".red().bold(), e);
                    process::exit(1);
                }
            });
        }

        Some(Commands::Spec { command }) => {
            handle_spec_command(command);
        }

        Some(Commands::Generate { from_spec, output, no_tests, no_provenance }) => {
            handle_generate_command(from_spec, output, *no_tests, *no_provenance);
        }

        Some(Commands::GenerateTests { spec, output, random_count }) => {
            handle_generate_tests_command(spec, output, *random_count);
        }

        Some(Commands::Provenance { input, format, agent, pattern, verified_only }) => {
            handle_provenance_command(input, format, agent, pattern, *verified_only);
        }

        Some(Commands::Benchmark { name, format }) => {
            handle_benchmark_command(name, format);
        }

        Some(Commands::Compose { first, second, json }) => {
            handle_compose_command(first, second, *json);
        }

        Some(Commands::Diff { old, new, semantic, format }) => {
            handle_diff_command(old, new, *semantic, format);
        }

        None => {
            // Default: start REPL
            run_repl(compiler);
        }
    }
}

fn handle_spec_command(command: &SpecCommands) {
    use fastforth::{Specification, SpecValidator};

    match command {
        SpecCommands::Validate { spec, strict } => {
            match Specification::from_file(spec) {
                Ok(specification) => {
                    let validator = if *strict {
                        SpecValidator::strict()
                    } else {
                        SpecValidator::new()
                    };

                    match validator.validate(&specification) {
                        Ok(()) => {
                            println!("{}", "✓ Specification is valid".green().bold());
                            println!("  Word: {}", specification.word);
                            println!("  Stack Effect: {}", specification.stack_comment());
                            if let Some(desc) = &specification.description {
                                println!("  Description: {}", desc);
                            }
                            println!("  Test Cases: {}", specification.test_count());
                        }
                        Err(e) => {
                            eprintln!("{}: {}", "Validation failed".red().bold(), e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Failed to load specification".red().bold(), e);
                    process::exit(1);
                }
            }
        }

        SpecCommands::Show { spec } => {
            match Specification::from_file(spec) {
                Ok(specification) => {
                    match specification.to_json_pretty() {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            eprintln!("{}: {}", "Failed to format specification".red().bold(), e);
                            process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Failed to load specification".red().bold(), e);
                    process::exit(1);
                }
            }
        }
    }
}

fn handle_generate_command(
    spec_path: &PathBuf,
    output: &Option<PathBuf>,
    no_tests: bool,
    no_provenance: bool,
) {
    use fastforth::{Specification, SpecCodeGenerator};

    match Specification::from_file(spec_path) {
        Ok(specification) => {
            let generator = SpecCodeGenerator::new()
                .with_tests(!no_tests)
                .with_provenance(!no_provenance);

            match generator.generate(&specification) {
                Ok(code) => {
                    if let Some(output_path) = output {
                        match std::fs::write(output_path, &code) {
                            Ok(()) => {
                                println!(
                                    "{} Generated code written to: {}",
                                    "✓".green().bold(),
                                    output_path.display()
                                );
                            }
                            Err(e) => {
                                eprintln!("{}: {}", "Failed to write output".red().bold(), e);
                                process::exit(1);
                            }
                        }
                    } else {
                        println!("{}", code);
                    }
                }
                Err(e) => {
                    eprintln!("{}: {}", "Code generation failed".red().bold(), e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Failed to load specification".red().bold(), e);
            process::exit(1);
        }
    }
}

fn handle_generate_tests_command(
    spec_path: &PathBuf,
    output: &Option<PathBuf>,
    random_count: usize,
) {
    use fastforth::{Specification, TestGenerator};

    match Specification::from_file(spec_path) {
        Ok(specification) => {
            let generator = TestGenerator::new()
                .with_random_count(random_count);

            match generator.generate(&specification) {
                Ok(tests) => {
                    let forth_tests = generator.generate_forth_tests(&specification, &tests);

                    if let Some(output_path) = output {
                        match std::fs::write(output_path, &forth_tests) {
                            Ok(()) => {
                                println!(
                                    "{} Generated {} tests written to: {}",
                                    "✓".green().bold(),
                                    tests.len(),
                                    output_path.display()
                                );
                            }
                            Err(e) => {
                                eprintln!("{}: {}", "Failed to write output".red().bold(), e);
                                process::exit(1);
                            }
                        }
                    } else {
                        println!("{}", forth_tests);
                    }

                    println!(
                        "\n{} Generated {} test cases",
                        "✓".green().bold(),
                        tests.len()
                    );
                }
                Err(e) => {
                    eprintln!("{}: {}", "Test generation failed".red().bold(), e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Failed to load specification".red().bold(), e);
            process::exit(1);
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
    println!("  ✓ Performance: Benchmark-driven generation");
    println!("  ✓ Provenance: Metadata tracking");
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

fn handle_provenance_command(
    input: &PathBuf,
    format: &str,
    agent_filter: &Option<String>,
    pattern_filter: &Option<String>,
    verified_only: bool,
) {
    use fastforth::provenance::extraction::{ProvenanceExtractor, generate_report};

    // Read source file
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: {}", "Failed to read file".red().bold(), e);
            process::exit(1);
        }
    };

    // Create extractor with filters
    let mut extractor = ProvenanceExtractor::new();
    if let Some(agent) = agent_filter {
        extractor = extractor.with_agent_filter(agent.clone());
    }
    if let Some(pattern) = pattern_filter {
        extractor = extractor.with_pattern_filter(pattern.clone());
    }
    if verified_only {
        extractor = extractor.verified_only();
    }

    // Extract metadata
    match extractor.extract(&source) {
        Ok(metadata) => {
            if metadata.is_empty() {
                println!("{}", "No provenance metadata found".yellow());
                return;
            }

            match format {
                "json" => {
                    match serde_json::to_string_pretty(&metadata) {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            eprintln!("{}: {}", "Failed to serialize metadata".red().bold(), e);
                            process::exit(1);
                        }
                    }
                }
                "text" | _ => {
                    let report = generate_report(&metadata);
                    println!("{}", report);
                }
            }
        }
        Err(e) => {
            eprintln!("{}: {}", "Failed to extract provenance".red().bold(), e);
            process::exit(1);
        }
    }
}

fn handle_benchmark_command(name: &Option<String>, format: &str) {
    use fastforth::performance::benchmarks::{StandardBenchmarks, BenchmarkReport};

    let suite = StandardBenchmarks::create_suite();

    let results = if let Some(benchmark_name) = name {
        // Run specific benchmark
        match suite.run_benchmark(benchmark_name) {
            Ok(result) => vec![result],
            Err(e) => {
                eprintln!("{}: {}", "Benchmark failed".red().bold(), e);
                process::exit(1);
            }
        }
    } else {
        // Run all benchmarks
        suite.run_all()
    };

    let report = BenchmarkReport::new(results);

    match format {
        "json" => {
            match report.to_json() {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("{}: {}", "Failed to serialize results".red().bold(), e);
                    process::exit(1);
                }
            }
        }
        "text" | _ => {
            println!("{}", report.format());
        }
    }
}

fn handle_compose_command(first: &str, second: &str, json: bool) {
    use fastforth::type_algebra::{TypeComposer, AlgebraicStackEffect};
    use fastforth_frontend::parse_program;
    use fastforth_frontend::stack_effects::StackEffectInference;

    // Parse first and second as either word names or stack effects
    // For simplicity, assume they are Forth code that defines words
    let first_prog = match parse_program(first) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}: Failed to parse first: {:?}", "Error".red().bold(), e);
            process::exit(1);
        }
    };

    let second_prog = match parse_program(second) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}: Failed to parse second: {:?}", "Error".red().bold(), e);
            process::exit(1);
        }
    };

    // Infer stack effects
    let mut inference = StackEffectInference::new();

    let first_effect = if !first_prog.definitions.is_empty() {
        inference.add_definition(&first_prog.definitions[0]).ok();
        inference.get_effect(&first_prog.definitions[0].name)
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("{}: Could not infer effect for first word", "Error".red().bold());
                process::exit(1);
            })
    } else {
        eprintln!("{}: No definition found in first code", "Error".red().bold());
        process::exit(1);
    };

    let second_effect = if !second_prog.definitions.is_empty() {
        inference.add_definition(&second_prog.definitions[0]).ok();
        inference.get_effect(&second_prog.definitions[0].name)
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("{}: Could not infer effect for second word", "Error".red().bold());
                process::exit(1);
            })
    } else {
        eprintln!("{}: No definition found in second code", "Error".red().bold());
        process::exit(1);
    };

    // Convert to algebraic effects
    let first_alg = AlgebraicStackEffect::from_frontend(&first_effect);
    let second_alg = AlgebraicStackEffect::from_frontend(&second_effect);

    // Compose
    let mut composer = TypeComposer::new();
    match composer.compose(&first_alg, &second_alg) {
        Ok(composed) => {
            if json {
                let result = serde_json::json!({
                    "valid": true,
                    "first": format!("{}", first_alg),
                    "second": format!("{}", second_alg),
                    "composed": format!("{}", composed),
                    "net_effect": composed.net_effect(),
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("{}", "Composition Result:".green().bold());
                println!("  First:    {}", format!("{}", first_alg).cyan());
                println!("  Second:   {}", format!("{}", second_alg).cyan());
                println!("  Composed: {}", format!("{}", composed).green().bold());
                println!("  Net Effect: {} stack items", composed.net_effect());
                println!("\n{}", "✓ Composition valid".green().bold());
            }
        }
        Err(e) => {
            if json {
                let result = serde_json::json!({
                    "valid": false,
                    "error": format!("{}", e),
                    "first": format!("{}", first_alg),
                    "second": format!("{}", second_alg),
                });
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("{}", "Composition Failed:".red().bold());
                println!("  First:  {}", format!("{}", first_alg).cyan());
                println!("  Second: {}", format!("{}", second_alg).cyan());
                println!("  Error:  {}", format!("{}", e).red());
            }
            process::exit(1);
        }
    }
}

fn handle_diff_command(old_path: &PathBuf, new_path: &PathBuf, _semantic: bool, format: &str) {
    use fastforth::semantic_diff::{SemanticDiffer, DiffReporter, ReportFormat};

    let differ = SemanticDiffer::new();

    let result = match differ.diff_files(old_path, new_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}: {}", "Diff failed".red().bold(), e);
            process::exit(1);
        }
    };

    let report_format = match format {
        "json" => ReportFormat::Json,
        _ => ReportFormat::Human,
    };

    let report = DiffReporter::report(&result, report_format);
    println!("{}", report);

    // Exit with non-zero if changes detected
    if result.changed_words > 0 {
        process::exit(1);
    }
}
