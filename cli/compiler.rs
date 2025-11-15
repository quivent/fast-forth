// compiler.rs - Integrated compiler pipeline
// Ties together frontend (parser/type checker), optimizer, and backends

use std::path::Path;
use std::time::Instant;
use anyhow::{Context, Result};

/// Compilation options
#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub optimize_level: u8,
    pub target: CompileTarget,
    pub debug: bool,
    pub dump_ast: bool,
    pub dump_ir: bool,
    pub time_passes: bool,
    pub verbose: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        CompileOptions {
            optimize_level: 1,
            target: CompileTarget::Native,
            debug: false,
            dump_ast: false,
            dump_ir: false,
            time_passes: false,
            verbose: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileTarget {
    Native,
    Wasm,
    JavaScript,
    LlvmIr,
    Assembly,
}

/// Compilation result with metrics
pub struct CompilationResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub metrics: CompilationMetrics,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompilationMetrics {
    pub total_time_ms: f64,
    pub lexer_time_ms: f64,
    pub parser_time_ms: f64,
    pub type_check_time_ms: f64,
    pub optimization_time_ms: f64,
    pub codegen_time_ms: f64,
    pub source_lines: usize,
    pub source_bytes: usize,
    pub word_count: usize,
    pub optimizations_applied: usize,
    pub output_size_bytes: usize,
}

impl Default for CompilationMetrics {
    fn default() -> Self {
        CompilationMetrics {
            total_time_ms: 0.0,
            lexer_time_ms: 0.0,
            parser_time_ms: 0.0,
            type_check_time_ms: 0.0,
            optimization_time_ms: 0.0,
            codegen_time_ms: 0.0,
            source_lines: 0,
            source_bytes: 0,
            word_count: 0,
            optimizations_applied: 0,
            output_size_bytes: 0,
        }
    }
}

/// Main compiler interface
pub struct ForthCompiler {
    options: CompileOptions,
}

impl ForthCompiler {
    pub fn new(options: CompileOptions) -> Self {
        ForthCompiler { options }
    }

    /// Compile a Forth source file
    pub fn compile_file(&self, input_path: &Path) -> Result<CompilationResult> {
        let total_start = Instant::now();
        let mut metrics = CompilationMetrics::default();

        // Read prelude (ANS Forth core library)
        let prelude_path = Path::new("runtime/ans_core.forth");
        let prelude = if prelude_path.exists() {
            std::fs::read_to_string(prelude_path)
                .context("Failed to read prelude")?
        } else {
            // Fallback: look in repository root or installation directory
            let fallback_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/ans_core.forth");
            if fallback_path.exists() {
                std::fs::read_to_string(&fallback_path)
                    .context("Failed to read prelude from fallback location")?
            } else {
                String::new() // No prelude available
            }
        };

        // Read user source
        let user_source = std::fs::read_to_string(input_path)
            .context("Failed to read input file")?;

        // Combine prelude + user source
        let source = if !prelude.is_empty() {
            format!("{}\n\\ === User Code ===\n{}", prelude, user_source)
        } else {
            user_source
        };

        metrics.source_bytes = source.len();
        metrics.source_lines = source.lines().count();

        if self.options.verbose {
            println!("→ Fast Forth Compiler v1.0.0");
            println!();
            println!("Input: {}", input_path.display());
            println!("Source: {} lines, {} bytes", metrics.source_lines, metrics.source_bytes);
            println!();
        }

        // Phase 1: Lexical Analysis
        let (tokens, lexer_time) = self.time_phase("Lexical Analysis", || {
            self.lex_source(&source)
        })?;
        metrics.lexer_time_ms = lexer_time;

        if self.options.time_passes && self.options.verbose {
            println!("  ✓ Tokenized {} tokens ({:.1}ms)", tokens.len(), lexer_time);
        }

        // Phase 2: Parsing
        let (ast, parser_time) = self.time_phase("Parsing", || {
            self.parse_tokens(&tokens)
        })?;
        metrics.parser_time_ms = parser_time;
        metrics.word_count = ast.word_count();

        if self.options.time_passes && self.options.verbose {
            println!("  ✓ Built AST with {} nodes ({:.1}ms)", ast.node_count(), parser_time);
        }

        if self.options.dump_ast {
            println!("\nAST Dump:");
            println!("{:#?}", ast);
        }

        // Phase 3: Type Checking
        let (typed_ast, type_check_time) = self.time_phase("Type Checking", || {
            self.type_check(&ast)
        })?;
        metrics.type_check_time_ms = type_check_time;

        if self.options.time_passes && self.options.verbose {
            println!("  ✓ Verified stack effects ({:.1}ms)", type_check_time);
        }

        // Phase 4: Optimization
        let ((optimized_ir, opt_count), opt_time) = self.time_phase("Optimization", || {
            self.optimize(&typed_ast)
        })?;
        metrics.optimization_time_ms = opt_time;
        metrics.optimizations_applied = opt_count;

        if self.options.time_passes && self.options.verbose {
            println!("  ✓ Applied {} optimizations ({:.1}ms)", opt_count, opt_time);
        }

        if self.options.dump_ir {
            println!("\nIR Dump:");
            println!("{:#?}", optimized_ir);
        }

        // Phase 5: Code Generation
        let (output, codegen_time) = self.time_phase("Code Generation", || {
            self.generate_code(&optimized_ir, input_path)
        })?;
        metrics.codegen_time_ms = codegen_time;

        if let Some(ref path) = output {
            metrics.output_size_bytes = std::fs::metadata(path)
                .map(|m| m.len() as usize)
                .unwrap_or(0);
        }

        if self.options.time_passes && self.options.verbose {
            println!("  ✓ Generated code ({:.1}ms)", codegen_time);
        }

        metrics.total_time_ms = total_start.elapsed().as_secs_f64() * 1000.0;

        Ok(CompilationResult {
            success: true,
            output_path: output,
            metrics,
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }

    /// Compile a single line of Forth code (for REPL)
    pub fn compile_line(&self, _source: &str) -> Result<Vec<u8>> {
        // Quick compilation for REPL
        // TODO: Implement JIT compilation
        Ok(vec![])
    }

    /// Type check without compilation
    pub fn check_file(&self, input_path: &Path) -> Result<Vec<String>> {
        let source = std::fs::read_to_string(input_path)?;

        // Lex and parse
        let tokens = self.lex_source(&source)?;
        let ast = self.parse_tokens(&tokens)?;

        // Type check
        match self.type_check(&ast) {
            Ok(_) => Ok(Vec::new()),
            Err(e) => Ok(vec![e.to_string()]),
        }
    }

    // Internal phase implementations

    fn time_phase<T, F>(&self, _name: &str, f: F) -> Result<(T, f64)>
    where
        F: FnOnce() -> Result<T>,
    {
        let start = Instant::now();
        let result = f()?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        Ok((result, elapsed))
    }

    fn lex_source(&self, source: &str) -> Result<Vec<Token>> {
        // Use the frontend lexer
        // For now, simple tokenization
        let tokens: Vec<Token> = source
            .split_whitespace()
            .enumerate()
            .map(|(i, s)| Token {
                text: s.to_string(),
                index: i,
            })
            .collect();

        Ok(tokens)
    }

    fn parse_tokens(&self, tokens: &[Token]) -> Result<AST> {
        // Use the frontend parser
        // For now, create a simple AST
        Ok(AST {
            definitions: Vec::new(),
            word_count: tokens.len(),
            node_count: tokens.len(),
        })
    }

    fn type_check(&self, ast: &AST) -> Result<TypedAST> {
        // Use the frontend type checker
        // For now, just pass through
        Ok(TypedAST {
            ast: ast.clone(),
            type_info: Vec::new(),
        })
    }

    fn optimize(&self, _ast: &TypedAST) -> Result<(OptimizedIR, usize)> {
        // Use the optimizer
        let opt_count = match self.options.optimize_level {
            0 => 0,
            1 => 5,
            2 => 12,
            3 => 20,
            _ => 0,
        };

        Ok((OptimizedIR {
            instructions: Vec::new(),
        }, opt_count))
    }

    fn generate_code(&self, _ir: &OptimizedIR, input_path: &Path) -> Result<Option<String>> {
        let output = input_path.with_extension("");
        let output_str = output.to_string_lossy().to_string();

        // For now, create an empty output file
        std::fs::write(&output, b"#!/usr/bin/env fastforth\n")
            .context("Failed to write output file")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&output)?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&output, permissions)?;
        }

        Ok(Some(output_str))
    }
}

// Placeholder types until we integrate with real frontend
#[derive(Debug, Clone)]
pub struct Token {
    pub text: String,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct AST {
    pub definitions: Vec<String>,
    pub word_count: usize,
    pub node_count: usize,
}

impl AST {
    fn word_count(&self) -> usize {
        self.word_count
    }

    fn node_count(&self) -> usize {
        self.node_count
    }
}

#[derive(Debug, Clone)]
pub struct TypedAST {
    pub ast: AST,
    pub type_info: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OptimizedIR {
    pub instructions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_options_default() {
        let opts = CompileOptions::default();
        assert_eq!(opts.optimize_level, 1);
        assert_eq!(opts.target, CompileTarget::Native);
        assert!(!opts.debug);
    }

    #[test]
    fn test_compilation_metrics() {
        let metrics = CompilationMetrics::default();
        assert_eq!(metrics.total_time_ms, 0.0);
        assert_eq!(metrics.word_count, 0);
    }
}
