# Fast Forth CLI - Implementation Guide

**Target Audience**: Engineers implementing the Fast Forth CLI
**Version**: 1.0
**Last Updated**: 2025-11-14

---

## Overview

This guide provides detailed implementation instructions for the Fast Forth CLI developer tools, including code organization, testing strategies, and integration points.

---

## Phase 1: Core Infrastructure (Week 1-2)

### 1.1 CLI Argument Parsing

**Files**: `main.rs`

**Implementation**:
```rust
// Use clap derive API for clean, maintainable CLI
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fastforth")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short, long, global = true)]
    verbose: bool,
}

// Implement all command handlers
impl Cli {
    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        match &self.command {
            None => run_repl(self),
            Some(Commands::Compile { .. }) => run_compile(self),
            // ... other commands
        }
    }
}
```

**Testing**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_parsing() {
        let args = vec!["fastforth", "compile", "test.fth", "-O3"];
        let cli = Cli::parse_from(args);
        // Assert correct parsing
    }
}
```

**Success Criteria**:
- All commands parse correctly
- Help messages display properly
- Error messages for invalid arguments
- Global options work with all commands

---

### 1.2 Error Message System

**Files**: `error_messages.rs`

**Implementation Steps**:

1. **Define error types**:
```rust
pub enum ErrorType {
    StackUnderflow,
    TypeMismatch,
    UndefinedWord,
    StackEffectMismatch,
}
```

2. **Implement error formatter**:
```rust
pub struct ErrorMessage {
    severity: ErrorSeverity,
    title: String,
    context: Option<CodeContext>,
    suggestions: Vec<Suggestion>,
}

impl ErrorMessage {
    pub fn format(&self) -> String {
        // Build beautiful error message
        let mut output = String::new();
        output.push_str(&format!("{}: {}\n", self.severity, self.title));
        // ... add context, suggestions, etc.
        output
    }
}
```

3. **Implement fuzzy word matching**:
```rust
pub fn find_similar_words(word: &str, dictionary: &[String]) -> Vec<(String, f32)> {
    dictionary.iter()
        .map(|w| (w.clone(), calculate_similarity(word, w)))
        .filter(|(_, score)| *score > 0.4)
        .sorted_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
        .take(5)
        .collect()
}
```

**Testing**:
```rust
#[test]
fn test_error_formatting() {
    let error = ErrorTemplates::stack_underflow(
        "AVERAGE",
        location,
        source_line,
        column,
        2,  // expected
        1,  // actual
    );

    let formatted = error.format();
    assert!(formatted.contains("Stack underflow"));
    assert!(formatted.contains("AVERAGE"));
}

#[test]
fn test_fuzzy_matching() {
    let dict = vec!["AVERAGE".to_string(), "MERGE".to_string()];
    let similar = find_similar_words("AVERGE", &dict, 3);
    assert_eq!(similar[0].0, "AVERAGE");
    assert!(similar[0].1 > 0.85);
}
```

**Success Criteria**:
- All error types have beautiful formatting
- Fuzzy matching finds similar words accurately
- Code context displayed with pointer
- Suggestions are actionable and helpful

---

### 1.3 Basic REPL

**Files**: `repl.rs`

**Implementation Steps**:

1. **Set up rustyline editor**:
```rust
use rustyline::{Editor, Config};

pub struct Repl {
    editor: Editor<()>,
    stack: Stack,
    words: HashMap<String, WordDefinition>,
}

impl Repl {
    pub fn new(config: ReplConfig) -> Result<Self> {
        let editor = Editor::with_config(Config::builder()
            .history_ignore_space(true)
            .completion_type(CompletionType::List)
            .build())?;

        Ok(Repl {
            editor,
            stack: Stack::new(),
            words: HashMap::new(),
        })
    }
}
```

2. **Implement main loop**:
```rust
pub fn run(&mut self) -> Result<()> {
    loop {
        let line = self.editor.readline(&self.config.prompt)?;
        self.editor.add_history_entry(&line);

        let start = Instant::now();
        self.process_line(&line)?;
        let elapsed = start.elapsed();

        self.print_result(elapsed);
        self.print_stack_state();
    }
}
```

3. **Implement stack operations**:
```rust
impl Stack {
    pub fn push(&mut self, item: StackItem) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Result<StackItem, &'static str> {
        self.items.pop().ok_or("Stack underflow")
    }

    pub fn display(&self) -> String {
        if self.items.is_empty() {
            "[ ]  (empty)".to_string()
        } else {
            format!("[ {} ]", self.items.iter().map(|i| i.to_string()).join(" "))
        }
    }
}
```

**Testing**:
```rust
#[test]
fn test_stack_operations() {
    let mut stack = Stack::new();
    stack.push(StackItem::Integer(5));
    stack.push(StackItem::Integer(3));

    assert_eq!(stack.depth(), 2);
    assert_eq!(stack.pop().unwrap(), StackItem::Integer(3));
    assert_eq!(stack.depth(), 1);
}

#[test]
fn test_arithmetic() {
    let mut repl = Repl::new(ReplConfig::default())?;
    repl.execute_line("5 3 +")?;
    assert_eq!(repl.stack.pop()?, StackItem::Integer(8));
}
```

**Success Criteria**:
- REPL starts and accepts input
- Stack operations work correctly
- Basic arithmetic works
- History persists across sessions
- Response time < 50ms for all operations

---

## Phase 2: Enhanced Features (Week 3-4)

### 2.1 Advanced REPL Features

**Implementation Steps**:

1. **Add completion support**:
```rust
use rustyline::completion::{Completer, Pair};

struct ForthCompleter {
    words: Vec<String>,
}

impl Completer for ForthCompleter {
    fn complete(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        // Find word being typed
        let word_start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let prefix = &line[word_start..pos];

        // Find matching words
        let matches = self.words.iter()
            .filter(|w| w.starts_with(prefix))
            .map(|w| Pair {
                display: w.clone(),
                replacement: w.clone(),
            })
            .collect();

        Ok((word_start, matches))
    }
}
```

2. **Add debugging commands**:
```rust
impl Repl {
    fn handle_debug_command(&mut self, word: &str) -> Result<()> {
        println!("→ Entering debug mode for {}", word);

        loop {
            let cmd = self.editor.readline("debug> ")?;
            match cmd.as_str() {
                "s" | "step" => self.debug_step()?,
                "c" | "continue" => break,
                "p" | "print" => self.debug_print()?,
                "q" | "quit" => return Ok(()),
                _ => println!("Unknown command: {}", cmd),
            }
        }

        Ok(())
    }
}
```

**Testing**:
```rust
#[test]
fn test_completion() {
    let completer = ForthCompleter {
        words: vec!["AVERAGE".to_string(), "ADD".to_string()],
    };

    let (_, matches) = completer.complete("AV", 2)?;
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].display, "AVERAGE");
}
```

---

### 2.2 Profiler Implementation

**Files**: `profiler.rs`

**Implementation Steps**:

1. **Set up profiling infrastructure**:
```rust
pub struct Profiler {
    profiles: HashMap<String, WordProfile>,
    call_stack: Vec<CallFrame>,
    total_time: Duration,
}

impl Profiler {
    pub fn enter_word(&mut self, word: String) {
        let frame = CallFrame {
            word: word.clone(),
            start_time: Instant::now(),
            children_time: Duration::from_secs(0),
        };
        self.call_stack.push(frame);
    }

    pub fn exit_word(&mut self, word: &str) {
        if let Some(frame) = self.call_stack.pop() {
            let elapsed = frame.start_time.elapsed();
            let self_time = elapsed - frame.children_time;

            // Update profile
            self.profiles.entry(frame.word)
                .and_modify(|p| {
                    p.call_count += 1;
                    p.total_time += elapsed;
                    p.self_time += self_time;
                });
        }
    }
}
```

2. **Generate report**:
```rust
impl ProfilerReport {
    pub fn display(&self) {
        self.print_header();
        self.print_hot_spots();
        self.print_call_graph();
        self.print_optimization_opportunities();
        self.print_summary();
    }

    fn print_hot_spots(&self) {
        println!("TOP 10 HOT SPOTS:");
        for (i, profile) in self.hot_spots.iter().enumerate().take(10) {
            let percentage = profile.percentage(self.total_time);
            println!("{:2}  {:<15} {:>6.0}ms {:>4.1}%",
                i + 1,
                profile.name,
                profile.self_time.as_secs_f64() * 1000.0,
                percentage
            );
        }
    }
}
```

3. **Generate flame graph**:
```rust
impl Profiler {
    pub fn generate_flame_graph(&self) -> String {
        let mut html = String::new();
        html.push_str(include_str!("flame_graph_template.html"));

        // Generate JSON data
        let data = self.build_flame_graph_tree();
        let json = serde_json::to_string(&data)?;

        html.replace("{{DATA}}", &json)
    }
}
```

**Testing**:
```rust
#[test]
fn test_profiler() {
    let mut profiler = Profiler::new();
    profiler.start();

    profiler.enter_word("MAIN".to_string());
    std::thread::sleep(Duration::from_millis(10));

    profiler.enter_word("HELPER".to_string());
    std::thread::sleep(Duration::from_millis(5));
    profiler.exit_word("HELPER");

    profiler.exit_word("MAIN");
    profiler.stop();

    let report = profiler.generate_report();
    assert!(report.total_time.as_millis() >= 15);
}
```

---

### 2.3 Documentation Generator

**Files**: `doc_generator.rs`

**Implementation Steps**:

1. **Parse stack effect comments**:
```rust
pub fn parse_word_documentation(source: &str) -> Vec<WordDoc> {
    let mut docs = Vec::new();

    // Regex to match word definitions with stack effects
    let re = Regex::new(r": (\w+) \( ([^)]+) \)").unwrap();

    for cap in re.captures_iter(source) {
        let name = cap[1].to_string();
        let stack_effect = cap[2].to_string();

        docs.push(WordDoc {
            name,
            stack_effect,
            description: extract_description(source, name),
            examples: extract_examples(source, name),
        });
    }

    docs
}
```

2. **Generate HTML**:
```rust
pub fn generate_html_doc(word: &WordDoc) -> String {
    format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>{}</style>
</head>
<body>
    <h1>{}</h1>
    <div class="stack-effect">( {} )</div>
    <p>{}</p>
    {}
</body>
</html>
"#,
        word.name,
        include_str!("doc_style.css"),
        word.name,
        word.stack_effect,
        word.description,
        generate_examples_html(&word.examples)
    )
}
```

**Testing**:
```rust
#[test]
fn test_doc_parsing() {
    let source = r#"
        : AVERAGE ( a b -- avg )
          \ Computes average of two numbers
          + 2 / ;
    "#;

    let docs = parse_word_documentation(source);
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "AVERAGE");
    assert_eq!(docs[0].stack_effect, "a b -- avg");
}
```

---

## Phase 3: LSP Implementation (Week 5-6)

### 3.1 LSP Server Setup

**Files**: `lsp/server.rs`

**Implementation Steps**:

1. **Set up LSP server**:
```rust
use tower_lsp::{LspService, Server};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

#[derive(Debug)]
struct FastForthLanguageServer {
    client: Client,
    documents: Arc<RwLock<HashMap<Url, TextDocumentItem>>>,
    symbols: Arc<RwLock<SymbolTable>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for FastForthLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![":".into(), "(".into()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                // ... other capabilities
                ..Default::default()
            },
            ..Default::default()
        })
    }
}
```

2. **Implement completion**:
```rust
async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
    let uri = params.text_document_position.text_document.uri;
    let position = params.text_document_position.position;

    // Get document
    let documents = self.documents.read().await;
    let doc = documents.get(&uri).ok_or_else(|| /* error */)?;

    // Find word being typed
    let line = get_line(doc, position.line);
    let prefix = get_word_at_position(&line, position.character);

    // Get completions
    let symbols = self.symbols.read().await;
    let completions = symbols.find_completions(&prefix);

    Ok(Some(CompletionResponse::Array(
        completions.into_iter()
            .map(|sym| CompletionItem {
                label: sym.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(sym.stack_effect.clone()),
                documentation: Some(Documentation::String(sym.description.clone())),
                ..Default::default()
            })
            .collect()
    )))
}
```

3. **Implement diagnostics**:
```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;

    // Update document
    let mut documents = self.documents.write().await;
    if let Some(doc) = documents.get_mut(&uri) {
        apply_changes(doc, params.content_changes);
    }

    // Run diagnostics
    let diagnostics = self.run_diagnostics(&uri).await;

    // Publish diagnostics
    self.client.publish_diagnostics(uri.clone(), diagnostics, None).await;
}

async fn run_diagnostics(&self, uri: &Url) -> Vec<Diagnostic> {
    let documents = self.documents.read().await;
    let doc = documents.get(uri).unwrap();

    let mut diagnostics = Vec::new();

    // Parse and check for errors
    match parse_forth(&doc.text) {
        Ok(ast) => {
            // Type check
            if let Err(errors) = type_check(&ast) {
                for error in errors {
                    diagnostics.push(Diagnostic {
                        range: error.range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: error.message,
                        ..Default::default()
                    });
                }
            }
        }
        Err(parse_error) => {
            diagnostics.push(Diagnostic {
                range: parse_error.range,
                severity: Some(DiagnosticSeverity::ERROR),
                message: parse_error.message,
                ..Default::default()
            });
        }
    }

    diagnostics
}
```

**Testing**:
```rust
#[tokio::test]
async fn test_lsp_completion() {
    let server = create_test_server().await;

    let params = CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: Url::parse("file:///test.fth").unwrap(),
            },
            position: Position { line: 0, character: 2 },
        },
        ..Default::default()
    };

    let result = server.completion(params).await.unwrap();
    assert!(result.is_some());
}
```

---

### 3.2 VSCode Extension

**Files**: `vscode-extension/`

**Implementation Steps**:

1. **Create extension skeleton**:
```typescript
// extension.ts
import * as vscode from 'vscode';
import * as path from 'path';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    // Server options
    const serverOptions: ServerOptions = {
        command: 'fastforth',
        args: ['lsp'],
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'forth' }],
    };

    // Create client
    client = new LanguageClient(
        'fastforth',
        'Fast Forth Language Server',
        serverOptions,
        clientOptions
    );

    // Start client
    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
```

2. **Add syntax highlighting**:
```json
// syntaxes/forth.tmLanguage.json
{
  "scopeName": "source.forth",
  "patterns": [
    {
      "name": "comment.line.forth",
      "match": "\\\\.*$"
    },
    {
      "name": "keyword.control.forth",
      "match": "\\b(IF|THEN|ELSE|DO|LOOP|BEGIN|UNTIL|WHILE|REPEAT)\\b"
    },
    {
      "name": "keyword.other.forth",
      "match": "\\b(:|;|VARIABLE|CONSTANT|CREATE|ALLOT)\\b"
    },
    {
      "name": "constant.numeric.forth",
      "match": "\\b[-+]?[0-9]+\\.?[0-9]*\\b"
    },
    {
      "name": "string.quoted.double.forth",
      "match": "\"[^\"]*\""
    }
  ]
}
```

**Testing**:
- Manual testing in VSCode
- Extension test suite using `vscode-test`

---

## Phase 4: Polish & Testing (Week 7-8)

### 4.1 Performance Optimization

**Target**: All operations < 50ms

**Strategies**:

1. **Cache parsed AST**:
```rust
struct ASTCache {
    cache: HashMap<PathBuf, (SystemTime, Arc<AST>)>,
}

impl ASTCache {
    pub fn get_or_parse(&mut self, path: &Path) -> Result<Arc<AST>> {
        let modified = fs::metadata(path)?.modified()?;

        if let Some((cached_time, ast)) = self.cache.get(path) {
            if *cached_time >= modified {
                return Ok(ast.clone());
            }
        }

        let source = fs::read_to_string(path)?;
        let ast = Arc::new(parse(&source)?);
        self.cache.insert(path.to_owned(), (modified, ast.clone()));

        Ok(ast)
    }
}
```

2. **Use rayon for parallel processing**:
```rust
use rayon::prelude::*;

pub fn compile_workspace(files: &[PathBuf]) -> Result<Vec<Compiled>> {
    files.par_iter()
        .map(|file| compile_file(file))
        .collect()
}
```

3. **Profile and optimize hot paths**:
```bash
# Profile with perf
cargo build --release
perf record --call-graph=dwarf ./target/release/fastforth compile large.fth
perf report

# Profile with flamegraph
cargo flamegraph -- compile large.fth
```

---

### 4.2 Comprehensive Testing

**Test Categories**:

1. **Unit Tests** - Test individual functions
2. **Integration Tests** - Test component interactions
3. **End-to-End Tests** - Test full workflows
4. **Performance Tests** - Ensure targets met
5. **Property Tests** - Test with generated inputs

**Example Test Suite**:
```rust
// Unit test
#[test]
fn test_stack_operations() {
    let mut stack = Stack::new();
    stack.push(StackItem::Integer(5));
    assert_eq!(stack.depth(), 1);
}

// Integration test
#[test]
fn test_repl_session() {
    let mut repl = Repl::new(ReplConfig::default())?;
    repl.execute_line("5 3 +")?;
    repl.execute_line(".")?;
    // Check output
}

// Performance test
#[test]
fn test_repl_response_time() {
    let mut repl = Repl::new(ReplConfig::default())?;

    let start = Instant::now();
    repl.execute_line("5 3 +")?;
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_millis(50), "Response too slow: {:?}", elapsed);
}

// Property test
#[quickcheck]
fn test_arithmetic_commutativity(a: i64, b: i64) -> bool {
    let mut repl = Repl::new(ReplConfig::default()).unwrap();

    // Test a + b == b + a
    repl.execute_line(&format!("{} {} +", a, b)).unwrap();
    let result1 = repl.stack.pop().unwrap();

    repl.execute_line(&format!("{} {} +", b, a)).unwrap();
    let result2 = repl.stack.pop().unwrap();

    result1 == result2
}
```

---

## Integration Points

### With Frontend (Compiler)

```rust
// Compiler provides these interfaces
pub trait Compiler {
    fn parse(&self, source: &str) -> Result<AST>;
    fn type_check(&self, ast: &AST) -> Result<()>;
    fn compile(&self, ast: &AST, options: CompileOptions) -> Result<Binary>;
}

// CLI uses compiler
use fastforth_compiler::Compiler;

fn compile_file(path: &Path, options: CompileOptions) -> Result<()> {
    let compiler = Compiler::new();
    let source = fs::read_to_string(path)?;
    let ast = compiler.parse(&source)?;
    compiler.type_check(&ast)?;
    let binary = compiler.compile(&ast, options)?;
    binary.write_to_file(output_path)?;
    Ok(())
}
```

### With Runtime (JIT/Interpreter)

```rust
// Runtime provides these interfaces
pub trait Runtime {
    fn execute(&mut self, bytecode: &[u8]) -> Result<()>;
    fn call_word(&mut self, word: &str) -> Result<()>;
    fn get_stack(&self) -> &[StackItem];
}

// REPL uses runtime
use fastforth_runtime::Runtime;

impl Repl {
    fn execute_line(&mut self, line: &str) -> Result<()> {
        let bytecode = self.compiler.compile_line(line)?;
        self.runtime.execute(&bytecode)?;
        Ok(())
    }
}
```

---

## Continuous Integration

**GitHub Actions Workflow**:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check

  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - run: cargo bench
      - uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/output.txt
```

---

## Documentation

### User Documentation

1. **Tutorial** - Step-by-step introduction
2. **Reference** - Complete command reference
3. **Examples** - Real-world use cases
4. **FAQ** - Common questions

### Developer Documentation

1. **Architecture** - System design
2. **API Reference** - Code documentation
3. **Contributing Guide** - How to contribute
4. **Design Decisions** - Why choices were made

---

## Success Metrics

### Performance
- REPL response < 50ms ✓
- Compilation < 5ms per KB ✓
- Error detection < 100ms ✓
- LSP latency < 30ms ✓

### Quality
- Test coverage > 80%
- No clippy warnings
- All docs up to date
- Zero critical bugs

### User Satisfaction
- 85%+ satisfaction rating
- 100+ GitHub stars
- 10+ contributors
- Active community

---

## Next Steps

1. **Implement Phase 1** - Core infrastructure
2. **Test thoroughly** - Unit and integration tests
3. **Gather feedback** - From early users
4. **Iterate** - Based on feedback
5. **Document** - Keep docs updated
6. **Release** - When quality targets met

---

**Good luck building Fast Forth CLI!** 🚀

Remember: Focus on developer experience. Every detail matters. Make it delightful.
