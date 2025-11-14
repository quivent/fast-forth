//! Abstract Syntax Tree definitions for Forth

use std::fmt;

/// A complete Forth program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub definitions: Vec<Definition>,
    pub top_level_code: Vec<Word>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
            top_level_code: Vec::new(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

/// A word definition (: name ... ;)
#[derive(Debug, Clone, PartialEq)]
pub struct Definition {
    pub name: String,
    pub body: Vec<Word>,
    pub immediate: bool,
    pub stack_effect: Option<StackEffect>,
    pub location: SourceLocation,
}

/// Source code location for error reporting
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

/// Stack effect declaration ( in1 in2 -- out1 )
#[derive(Debug, Clone, PartialEq)]
pub struct StackEffect {
    pub inputs: Vec<StackType>,
    pub outputs: Vec<StackType>,
}

impl StackEffect {
    pub fn new(inputs: Vec<StackType>, outputs: Vec<StackType>) -> Self {
        Self { inputs, outputs }
    }

    pub fn net_effect(&self) -> i32 {
        self.outputs.len() as i32 - self.inputs.len() as i32
    }
}

impl fmt::Display for StackEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "( ")?;
        for (i, input) in self.inputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", input)?;
        }
        write!(f, " -- ")?;
        for (i, output) in self.outputs.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", output)?;
        }
        write!(f, " )")
    }
}

/// Stack value types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StackType {
    /// Integer (cell size)
    Int,
    /// Floating point
    Float,
    /// Address/pointer
    Addr,
    /// Boolean
    Bool,
    /// Character
    Char,
    /// String
    String,
    /// Polymorphic type variable (for type inference)
    Var(TypeVar),
    /// Unknown type (to be inferred)
    Unknown,
}

impl fmt::Display for StackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackType::Int => write!(f, "int"),
            StackType::Float => write!(f, "float"),
            StackType::Addr => write!(f, "addr"),
            StackType::Bool => write!(f, "bool"),
            StackType::Char => write!(f, "char"),
            StackType::String => write!(f, "string"),
            StackType::Var(v) => write!(f, "{}", v),
            StackType::Unknown => write!(f, "?"),
        }
    }
}

/// Type variable for polymorphic types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeVar {
    pub id: usize,
    pub name: Option<String>,
}

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "'{}", name)
        } else {
            write!(f, "'t{}", self.id)
        }
    }
}

/// A word in Forth code
#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    /// A literal integer value
    IntLiteral(i64),

    /// A literal float value
    FloatLiteral(f64),

    /// A string literal
    StringLiteral(String),

    /// A word reference (calling another word)
    WordRef {
        name: String,
        location: SourceLocation,
    },

    /// Control structure: IF
    If {
        then_branch: Vec<Word>,
        else_branch: Option<Vec<Word>>,
    },

    /// Control structure: BEGIN...UNTIL
    BeginUntil {
        body: Vec<Word>,
    },

    /// Control structure: BEGIN...WHILE...REPEAT
    BeginWhileRepeat {
        condition: Vec<Word>,
        body: Vec<Word>,
    },

    /// Control structure: DO...LOOP
    DoLoop {
        body: Vec<Word>,
        increment: i64, // 1 for LOOP, variable for +LOOP
    },

    /// Variable definition
    Variable {
        name: String,
    },

    /// Constant definition
    Constant {
        name: String,
        value: i64,
    },

    /// Comment (preserved for documentation)
    Comment(String),
}

/// Compilation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationMode {
    /// Interpreting (executing immediately)
    Interpret,
    /// Compiling (adding to current definition)
    Compile,
}

/// Token types from lexer
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// : (start definition)
    Colon,
    /// ; (end definition)
    Semicolon,
    /// Word/identifier
    Word(String),
    /// Integer literal
    Integer(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// ( (start comment or stack effect)
    LeftParen,
    /// ) (end comment or stack effect)
    RightParen,
    /// -- (stack effect separator)
    StackEffectSep,
    /// IF keyword
    If,
    /// THEN keyword
    Then,
    /// ELSE keyword
    Else,
    /// DO keyword
    Do,
    /// LOOP keyword
    Loop,
    /// +LOOP keyword
    PlusLoop,
    /// BEGIN keyword
    Begin,
    /// UNTIL keyword
    Until,
    /// WHILE keyword
    While,
    /// REPEAT keyword
    Repeat,
    /// VARIABLE keyword
    Variable,
    /// CONSTANT keyword
    Constant,
    /// IMMEDIATE keyword
    Immediate,
    /// End of file
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Colon => write!(f, ":"),
            Token::Semicolon => write!(f, ";"),
            Token::Word(w) => write!(f, "{}", w),
            Token::Integer(i) => write!(f, "{}", i),
            Token::Float(fl) => write!(f, "{}", fl),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::StackEffectSep => write!(f, "--"),
            Token::If => write!(f, "IF"),
            Token::Then => write!(f, "THEN"),
            Token::Else => write!(f, "ELSE"),
            Token::Do => write!(f, "DO"),
            Token::Loop => write!(f, "LOOP"),
            Token::PlusLoop => write!(f, "+LOOP"),
            Token::Begin => write!(f, "BEGIN"),
            Token::Until => write!(f, "UNTIL"),
            Token::While => write!(f, "WHILE"),
            Token::Repeat => write!(f, "REPEAT"),
            Token::Variable => write!(f, "VARIABLE"),
            Token::Constant => write!(f, "CONSTANT"),
            Token::Immediate => write!(f, "IMMEDIATE"),
            Token::Eof => write!(f, "<EOF>"),
        }
    }
}
