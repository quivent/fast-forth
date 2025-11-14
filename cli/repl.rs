// repl.rs - Fast Forth REPL Implementation
// Interactive Read-Eval-Print Loop with history, completion, and debugging

use rustyline::error::ReadlineError;
use rustyline::{Config, DefaultEditor};
use std::collections::HashMap;
use std::time::Instant;

/// REPL configuration
#[derive(Debug, Clone)]
pub struct ReplConfig {
    pub prompt: String,
    pub show_stack_depth: bool,
    pub show_timing: bool,
    pub history_size: usize,
    pub auto_indent: bool,
    pub syntax_highlight: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        ReplConfig {
            prompt: "forth> ".to_string(),
            show_stack_depth: true,
            show_timing: true,
            history_size: 1000,
            auto_indent: true,
            syntax_highlight: true,
        }
    }
}

/// Stack representation for REPL display
#[derive(Debug, Clone)]
pub struct Stack {
    items: Vec<StackItem>,
}

#[derive(Debug, Clone)]
pub enum StackItem {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl std::fmt::Display for StackItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StackItem::Integer(n) => write!(f, "{}", n),
            StackItem::Float(n) => write!(f, "{}", n),
            StackItem::String(s) => write!(f, "\"{}\"", s),
            StackItem::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
        }
    }
}

impl Stack {
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    pub fn push(&mut self, item: StackItem) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<StackItem> {
        self.items.pop()
    }

    pub fn depth(&self) -> usize {
        self.items.len()
    }

    pub fn display(&self) -> String {
        if self.items.is_empty() {
            "[ ]  (empty)".to_string()
        } else {
            let items: Vec<String> = self.items.iter().map(|i| i.to_string()).collect();
            format!("[ {} ]", items.join(" "))
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

/// REPL state
pub struct Repl {
    config: ReplConfig,
    stack: Stack,
    editor: DefaultEditor,
    words: HashMap<String, WordDefinition>,
    multiline_buffer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WordDefinition {
    pub name: String,
    pub stack_effect: String,
    pub implementation: String,
    pub description: Option<String>,
}

impl Repl {
    pub fn new(config: ReplConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let editor_config = Config::builder()
            .history_ignore_space(true)
            .completion_type(rustyline::CompletionType::List)
            .build();

        let mut editor = DefaultEditor::with_config(editor_config)?;

        // Load history from file
        let history_file = dirs::home_dir()
            .unwrap_or_default()
            .join(".fastforth_history");
        let _ = editor.load_history(&history_file);

        Ok(Repl {
            config,
            stack: Stack::new(),
            editor,
            words: HashMap::new(),
            multiline_buffer: None,
        })
    }

    /// Main REPL loop
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.print_welcome();

        loop {
            let prompt = if self.multiline_buffer.is_some() {
                "...    "
            } else {
                &self.config.prompt
            };

            match self.editor.readline(prompt) {
                Ok(line) => {
                    self.editor.add_history_entry(&line);

                    if let Err(e) = self.process_line(&line) {
                        eprintln!("\x1b[31m✗ Error:\x1b[0m {}", e);
                    }

                    // Show stack state after each command
                    if self.config.show_stack_depth && self.multiline_buffer.is_none() {
                        self.print_stack_state();
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    println!("Goodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history
        let history_file = dirs::home_dir()
            .unwrap_or_default()
            .join(".fastforth_history");
        self.editor.save_history(&history_file)?;

        Ok(())
    }

    /// Process a single line of input
    fn process_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let line = line.trim();

        // Empty line
        if line.is_empty() {
            return Ok(());
        }

        // Check for meta-commands
        if self.is_meta_command(line) {
            return self.handle_meta_command(line);
        }

        // Handle multiline input (word definitions)
        if line.starts_with(':') {
            self.multiline_buffer = Some(line.to_string());
            return Ok(());
        }

        if let Some(buffer) = &mut self.multiline_buffer {
            buffer.push(' ');
            buffer.push_str(line);

            if line.ends_with(';') {
                let complete = buffer.clone();
                self.multiline_buffer = None;
                return self.execute_word_definition(&complete);
            }
            return Ok(());
        }

        // Execute single line
        let start = Instant::now();
        self.execute_line(line)?;
        let elapsed = start.elapsed();

        if self.config.show_timing {
            println!(
                "  \x1b[32m✓ OK\x1b[0m ({:.1}ms)",
                elapsed.as_secs_f64() * 1000.0
            );
        }

        Ok(())
    }

    /// Check if line is a meta-command
    fn is_meta_command(&self, line: &str) -> bool {
        matches!(
            line.to_uppercase().as_str(),
            "HELP" | "QUIT" | "EXIT" | "CLEAR" | "CLS" | ".S" | "WORDS" | "CLEAR-STACK"
                | "DEPTH" | "HISTORY" | "VERSION" | "ENV"
        ) || line.to_uppercase().starts_with("SEE ")
            || line.to_uppercase().starts_with("HELP ")
            || line.to_uppercase().starts_with("LOAD ")
            || line.to_uppercase().starts_with("SAVE ")
            || line.to_uppercase().starts_with("DEBUG ")
    }

    /// Handle REPL meta-commands
    fn handle_meta_command(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let command = parts[0].to_uppercase();

        match command.as_str() {
            "HELP" => {
                if parts.len() > 1 {
                    self.show_word_help(parts[1])?;
                } else {
                    self.print_help();
                }
            }
            "QUIT" | "EXIT" => {
                std::process::exit(0);
            }
            "CLEAR" | "CLS" => {
                print!("\x1b[2J\x1b[1;1H");
            }
            ".S" => {
                println!("Stack: {}", self.stack.display());
            }
            "WORDS" => {
                self.list_words(parts.get(1).copied())?;
            }
            "CLEAR-STACK" => {
                self.stack.clear();
                println!("Stack cleared");
            }
            "DEPTH" => {
                println!("Stack depth: {}", self.stack.depth());
            }
            "SEE" => {
                if parts.len() > 1 {
                    self.show_word_definition(parts[1])?;
                } else {
                    println!("Usage: SEE <word>");
                }
            }
            "HISTORY" => {
                self.show_history()?;
            }
            "VERSION" => {
                println!("Fast Forth v1.0.0");
            }
            "ENV" => {
                self.show_environment();
            }
            _ => {
                println!("Unknown command: {}", line);
            }
        }

        Ok(())
    }

    /// Execute a line of Forth code
    fn execute_line(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse and execute tokens
        for token in line.split_whitespace() {
            self.execute_token(token)?;
        }
        Ok(())
    }

    /// Execute a single token
    fn execute_token(&mut self, token: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Try to parse as number
        if let Ok(n) = token.parse::<i64>() {
            self.stack.push(StackItem::Integer(n));
            return Ok(());
        }

        if let Ok(n) = token.parse::<f64>() {
            self.stack.push(StackItem::Float(n));
            return Ok(());
        }

        // String literal
        if token.starts_with('"') && token.ends_with('"') {
            let s = token[1..token.len() - 1].to_string();
            self.stack.push(StackItem::String(s));
            return Ok(());
        }

        // Execute as word
        self.execute_word(token)?;
        Ok(())
    }

    /// Execute a word
    fn execute_word(&mut self, word: &str) -> Result<(), Box<dyn std::error::Error>> {
        match word.to_uppercase().as_str() {
            "+" => self.word_add()?,
            "-" => self.word_subtract()?,
            "*" => self.word_multiply()?,
            "/" => self.word_divide()?,
            "DUP" => self.word_dup()?,
            "DROP" => self.word_drop()?,
            "SWAP" => self.word_swap()?,
            "." => self.word_dot()?,
            ".\"" => { /* Handle string output */ }
            _ => {
                // Check user-defined words
                if let Some(_def) = self.words.get(word) {
                    // Execute user-defined word
                    println!("Executing user word: {}", word);
                } else {
                    return Err(format!("Undefined word: {}", word).into());
                }
            }
        }
        Ok(())
    }

    /// Execute word definition
    fn execute_word_definition(&mut self, definition: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Parse: : WORD-NAME ( stack-effect ) implementation ;
        let parts: Vec<&str> = definition.split_whitespace().collect();

        if parts.len() < 3 {
            return Err("Invalid word definition".into());
        }

        let name = parts[1].to_uppercase();

        // Extract stack effect (if present)
        let mut stack_effect = String::new();
        let mut impl_start = 2;

        if parts.get(2) == Some(&"(") {
            // Find matching )
            for (i, &part) in parts.iter().enumerate().skip(3) {
                if part == ")" {
                    impl_start = i + 1;
                    break;
                }
                stack_effect.push_str(part);
                stack_effect.push(' ');
            }
        }

        // Extract implementation (everything between stack effect and ;)
        let implementation: Vec<&str> = parts[impl_start..]
            .iter()
            .take_while(|&&p| p != ";")
            .copied()
            .collect();

        let word = WordDefinition {
            name: name.clone(),
            stack_effect: stack_effect.trim().to_string(),
            implementation: implementation.join(" "),
            description: None,
        };

        self.words.insert(name.clone(), word);

        println!("  \x1b[32m✓ Defined {}\x1b[0m", name);
        if !stack_effect.is_empty() {
            println!("  Stack Effect: ( {} )", stack_effect);
        }
        println!("  Implementation: {}", implementation.join(" "));

        Ok(())
    }

    // Stack operation words
    fn word_add(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;

        match (a, b) {
            (StackItem::Integer(a), StackItem::Integer(b)) => {
                self.stack.push(StackItem::Integer(a + b));
            }
            (StackItem::Float(a), StackItem::Float(b)) => {
                self.stack.push(StackItem::Float(a + b));
            }
            _ => return Err("Type mismatch".into()),
        }
        Ok(())
    }

    fn word_subtract(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;

        match (a, b) {
            (StackItem::Integer(a), StackItem::Integer(b)) => {
                self.stack.push(StackItem::Integer(a - b));
            }
            (StackItem::Float(a), StackItem::Float(b)) => {
                self.stack.push(StackItem::Float(a - b));
            }
            _ => return Err("Type mismatch".into()),
        }
        Ok(())
    }

    fn word_multiply(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;

        match (a, b) {
            (StackItem::Integer(a), StackItem::Integer(b)) => {
                self.stack.push(StackItem::Integer(a * b));
            }
            (StackItem::Float(a), StackItem::Float(b)) => {
                self.stack.push(StackItem::Float(a * b));
            }
            _ => return Err("Type mismatch".into()),
        }
        Ok(())
    }

    fn word_divide(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;

        match (a, b) {
            (StackItem::Integer(a), StackItem::Integer(b)) => {
                if b == 0 {
                    return Err("Division by zero".into());
                }
                self.stack.push(StackItem::Integer(a / b));
            }
            (StackItem::Float(a), StackItem::Float(b)) => {
                self.stack.push(StackItem::Float(a / b));
            }
            _ => return Err("Type mismatch".into()),
        }
        Ok(())
    }

    fn word_dup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let item = self.stack.items.last().ok_or("Stack underflow")?.clone();
        self.stack.push(item);
        Ok(())
    }

    fn word_drop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.stack.pop().ok_or("Stack underflow")?;
        Ok(())
    }

    fn word_swap(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let b = self.stack.pop().ok_or("Stack underflow")?;
        let a = self.stack.pop().ok_or("Stack underflow")?;
        self.stack.push(b);
        self.stack.push(a);
        Ok(())
    }

    fn word_dot(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let item = self.stack.pop().ok_or("Stack underflow")?;
        print!("{} ", item);
        Ok(())
    }

    // Display methods
    fn print_welcome(&self) {
        println!("┌─ Fast Forth REPL v1.0.0 ────────────────────────────────┐");
        println!("│ Type 'help' for help, 'quit' to exit                    │");
        println!("└──────────────────────────────────────────────────────────┘");
        println!();
    }

    fn print_stack_state(&self) {
        println!();
        println!(
            "Stack: {}{}",
            self.stack.display(),
            if self.config.show_stack_depth {
                format!("                         Depth: {}", self.stack.depth())
            } else {
                String::new()
            }
        );
        println!();
    }

    fn print_help(&self) {
        println!("Fast Forth REPL Commands:");
        println!();
        println!("Meta-Commands:");
        println!("  help           Show this help message");
        println!("  help <word>    Show help for specific word");
        println!("  quit, exit     Exit REPL");
        println!("  clear, cls     Clear screen");
        println!();
        println!("Stack Operations:");
        println!("  .S             Show stack contents (non-destructive)");
        println!("  CLEAR-STACK    Clear the stack");
        println!("  DEPTH          Show stack depth");
        println!();
        println!("Word Inspection:");
        println!("  SEE <word>     Show word definition");
        println!("  WORDS          List all words");
        println!("  WORDS <pat>    List words matching pattern");
        println!();
        println!("Other:");
        println!("  HISTORY        Show command history");
        println!("  VERSION        Show Fast Forth version");
        println!("  ENV            Show environment");
    }

    fn show_word_help(&self, word: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(def) = self.words.get(&word.to_uppercase()) {
            println!();
            println!("  Word: {}", def.name);
            println!("  Stack Effect: ( {} )", def.stack_effect);
            println!("  Implementation: {}", def.implementation);
            if let Some(desc) = &def.description {
                println!("  Description: {}", desc);
            }
            println!();
        } else {
            println!("Word '{}' not found", word);
        }
        Ok(())
    }

    fn show_word_definition(&self, word: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.show_word_help(word)
    }

    fn list_words(&self, pattern: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        println!();
        println!("User-Defined Words:");
        for (name, def) in &self.words {
            if let Some(pat) = pattern {
                if !name.contains(&pat.to_uppercase()) {
                    continue;
                }
            }
            println!("  {} ( {} )", name, def.stack_effect);
        }
        println!();
        Ok(())
    }

    fn show_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("History feature not fully implemented");
        Ok(())
    }

    fn show_environment(&self) {
        println!("Fast Forth Environment:");
        println!("  Version: 1.0.0");
        println!("  Stack depth: {}", self.stack.depth());
        println!("  Defined words: {}", self.words.len());
    }
}
