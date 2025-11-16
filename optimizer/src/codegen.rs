//! Code Generation from Optimized IR
//!
//! Generates efficient native code from the optimized IR.
//! Supports multiple backends:
//! - Cranelift (for JIT compilation)
//! - C (for static compilation)
//! - Assembly (for maximum control)

use crate::ir::{ForthIR, Instruction, WordDef};
use crate::Result;

/// Code generation backend
pub trait CodegenBackend {
    /// Generate code from IR
    fn generate(&mut self, ir: &ForthIR) -> Result<String>;

    /// Generate code for a single word
    fn generate_word(&mut self, word: &WordDef) -> Result<String>;
}

/// C code generator
pub struct CCodegen {
    /// Use computed goto for instruction dispatch
    use_computed_goto: bool,
}

impl CCodegen {
    pub fn new() -> Self {
        Self {
            use_computed_goto: true,
        }
    }

    fn generate_instruction(&self, inst: &Instruction) -> String {
        use Instruction::*;

        match inst {
            Literal(v) => format!("    PUSH({});", v),
            FloatLiteral(v) => format!("    PUSH_F({});", v),

            // Stack operations
            Dup => "    TOS = NOS;".to_string(),
            Drop => "    sp--;".to_string(),
            Swap => "    { cell_t tmp = TOS; TOS = NOS; NOS = tmp; }".to_string(),
            Over => "    PUSH(NOS);".to_string(),

            // Arithmetic
            Add => "    NOS += TOS; DROP;".to_string(),
            Sub => "    NOS -= TOS; DROP;".to_string(),
            Mul => "    NOS *= TOS; DROP;".to_string(),
            Div => "    NOS /= TOS; DROP;".to_string(),
            Mod => "    NOS %= TOS; DROP;".to_string(),
            Neg => "    TOS = -TOS;".to_string(),
            Abs => "    TOS = (TOS < 0) ? -TOS : TOS;".to_string(),

            // Bitwise
            And => "    NOS &= TOS; DROP;".to_string(),
            Or => "    NOS |= TOS; DROP;".to_string(),
            Xor => "    NOS ^= TOS; DROP;".to_string(),
            Not => "    TOS = ~TOS;".to_string(),
            Shl => "    NOS <<= TOS; DROP;".to_string(),
            Shr => "    NOS >>= TOS; DROP;".to_string(),

            // Comparisons
            Eq => "    NOS = (NOS == TOS) ? -1 : 0; DROP;".to_string(),
            Ne => "    NOS = (NOS != TOS) ? -1 : 0; DROP;".to_string(),
            Lt => "    NOS = (NOS < TOS) ? -1 : 0; DROP;".to_string(),
            Le => "    NOS = (NOS <= TOS) ? -1 : 0; DROP;".to_string(),
            Gt => "    NOS = (NOS > TOS) ? -1 : 0; DROP;".to_string(),
            Ge => "    NOS = (NOS >= TOS) ? -1 : 0; DROP;".to_string(),
            ZeroEq => "    TOS = (TOS == 0) ? -1 : 0;".to_string(),
            ZeroLt => "    TOS = (TOS < 0) ? -1 : 0;".to_string(),
            ZeroGt => "    TOS = (TOS > 0) ? -1 : 0;".to_string(),

            // Superinstructions (optimized)
            DupAdd => "    TOS = TOS + TOS;".to_string(),
            DupMul => "    TOS = TOS * TOS;".to_string(),
            IncOne => "    TOS++;".to_string(),
            DecOne => "    TOS--;".to_string(),
            MulTwo => "    TOS <<= 1;".to_string(),
            DivTwo => "    TOS >>= 1;".to_string(),
            LiteralAdd(n) => format!("    TOS += {};", n),
            LiteralMul(n) => format!("    TOS *= {};", n),

            // Stack caching
            CachedDup { .. } => "    PUSH(TOS);".to_string(),
            CachedSwap { .. } => {
                "    { cell_t tmp = TOS; TOS = NOS; NOS = tmp; }".to_string()
            }
            FlushCache => "    /* cache flushed */".to_string(),

            // Control flow
            Call(name) => format!("    {}();", sanitize_name(name)),
            Return => "    return;".to_string(),
            Branch(target) => format!("    goto L{};", target),
            BranchIf(target) => format!("    if (TOS) {{ DROP; goto L{}; }} DROP;", target),
            BranchIfNot(target) => {
                format!("    if (!TOS) {{ DROP; goto L{}; }} DROP;", target)
            }

            // Memory operations
            Load => "    TOS = *(cell_t*)TOS;".to_string(),
            Store => "    *(cell_t*)TOS = NOS; sp -= 2;".to_string(),
            Load8 => "    TOS = *(uint8_t*)TOS;".to_string(),
            Store8 => "    *(uint8_t*)TOS = (uint8_t)NOS; sp -= 2;".to_string(),

            // Return stack
            ToR => "    *rsp++ = TOS; DROP;".to_string(),
            FromR => "    PUSH(*--rsp);".to_string(),
            RFetch => "    PUSH(rsp[-1]);".to_string(),

            // Metadata
            Label(name) => format!("{}:", sanitize_name(name)),
            Comment(text) => format!("    /* {} */", text),
            Nop => "    /* nop */".to_string(),

            _ => "    /* unimplemented */".to_string(),
        }
    }
}

impl Default for CCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl CodegenBackend for CCodegen {
    fn generate(&mut self, ir: &ForthIR) -> Result<String> {
        let mut code = String::new();

        // Header
        code.push_str(
            r#"
#include <stdint.h>
#include <stdbool.h>

typedef int64_t cell_t;

// Stack macros
#define STACK_SIZE 256
static cell_t stack[STACK_SIZE];
static cell_t* sp = stack;
static cell_t rstack[STACK_SIZE];
static cell_t* rsp = rstack;

#define TOS (sp[-1])
#define NOS (sp[-2])
#define THIRD (sp[-3])
#define PUSH(x) (*sp++ = (x))
#define DROP (sp--)

"#,
        );

        // Generate word definitions
        for word in ir.words.values() {
            code.push_str(&self.generate_word(word)?);
            code.push('\n');
        }

        // Generate main
        code.push_str("void forth_main(void) {\n");
        for inst in &ir.main {
            code.push_str(&self.generate_instruction(inst));
            code.push('\n');
        }
        code.push_str("}\n");

        Ok(code)
    }

    fn generate_word(&mut self, word: &WordDef) -> Result<String> {
        let mut code = String::new();

        code.push_str(&format!(
            "void {}(void) /* {} */ {{\n",
            sanitize_name(&word.name),
            word.stack_effect
        ));

        for inst in &word.instructions {
            code.push_str(&self.generate_instruction(inst));
            code.push('\n');
        }

        code.push_str("}\n");

        Ok(code)
    }
}

/// Sanitize word name for C identifier
fn sanitize_name(name: &str) -> String {
    let mut result = name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");

    // C identifiers can't start with a digit, replace leading digits with underscore
    if result.chars().next().map_or(false, |c| c.is_numeric()) {
        result = format!("_{}", &result[1..]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_codegen_simple() {
        let mut codegen = CCodegen::new();
        let ir = ForthIR::parse("1 2 +").unwrap();

        let code = codegen.generate(&ir).unwrap();

        assert!(code.contains("PUSH(1)"));
        assert!(code.contains("PUSH(2)"));
        assert!(code.contains("+="));
    }

    #[test]
    fn test_c_codegen_word() {
        let mut codegen = CCodegen::new();
        let word = WordDef::new(
            "square".to_string(),
            vec![Instruction::Dup, Instruction::Mul],
        );

        let code = codegen.generate_word(&word).unwrap();

        assert!(code.contains("void square"));
        assert!(code.contains("TOS"));
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("foo+bar"), "foo_bar");
        assert_eq!(sanitize_name("2dup"), "_dup");
        assert_eq!(sanitize_name("valid_name"), "valid_name");
    }
}
