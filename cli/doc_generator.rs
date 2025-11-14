// doc_generator.rs - Documentation generator for Forth code
// Extracts stack effects, comments, and examples to generate HTML/Markdown docs

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};

// Inline CSS instead of include_str! for now
const DOC_CSS: &str = include_str!("doc_style.css");

/// Documentation format
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocFormat {
    Html,
    Markdown,
}

/// Word documentation
#[derive(Debug, Clone)]
pub struct WordDoc {
    pub name: String,
    pub stack_effect: String,
    pub description: String,
    pub examples: Vec<String>,
    pub implementation: String,
    pub category: String,
}

/// Documentation generator
pub struct DocGenerator {
    format: DocFormat,
}

impl DocGenerator {
    pub fn new(format: DocFormat) -> Self {
        DocGenerator { format }
    }

    /// Generate documentation for a source file
    pub fn generate(&self, input_path: &Path, output_dir: &Path) -> Result<Vec<PathBuf>> {
        let source = fs::read_to_string(input_path)
            .context("Failed to read source file")?;

        // Parse word definitions
        let words = self.parse_words(&source)?;

        // Create output directory
        fs::create_dir_all(output_dir)?;

        // Generate documentation files
        let mut output_files = Vec::new();

        for word in &words {
            let output_path = match self.format {
                DocFormat::Html => output_dir.join(format!("{}.html", word.name.to_lowercase())),
                DocFormat::Markdown => output_dir.join(format!("{}.md", word.name.to_lowercase())),
            };

            let content = match self.format {
                DocFormat::Html => self.generate_html(word)?,
                DocFormat::Markdown => self.generate_markdown(word)?,
            };

            fs::write(&output_path, content)?;
            output_files.push(output_path);
        }

        // Generate index
        let index_path = self.generate_index(&words, output_dir)?;
        output_files.push(index_path);

        Ok(output_files)
    }

    /// Parse word definitions from source
    fn parse_words(&self, source: &str) -> Result<Vec<WordDoc>> {
        let mut words = Vec::new();
        let mut current_comments = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();

            // Collect comments
            if line.starts_with('\\') {
                current_comments.push(line[1..].trim().to_string());
                continue;
            }

            // Parse word definition
            if line.starts_with(':') {
                let word = self.parse_word_definition(line, &current_comments, &lines, i)?;
                if let Some(w) = word {
                    words.push(w);
                }
                current_comments.clear();
            }
        }

        Ok(words)
    }

    /// Parse a single word definition
    fn parse_word_definition(
        &self,
        line: &str,
        comments: &[String],
        _lines: &[&str],
        _index: usize,
    ) -> Result<Option<WordDoc>> {
        // Parse: : WORD-NAME ( stack-effect ) implementation ;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return Ok(None);
        }

        let name = parts[1].to_uppercase();

        // Extract stack effect
        let mut stack_effect = String::new();
        let mut impl_start = 2;

        if parts.get(2) == Some(&"(") {
            // Find matching )
            for (i, &part) in parts.iter().enumerate().skip(3) {
                if part == ")" {
                    impl_start = i + 1;
                    break;
                }
                if !stack_effect.is_empty() {
                    stack_effect.push(' ');
                }
                stack_effect.push_str(part);
            }
        }

        // Extract implementation
        let implementation: Vec<&str> = parts[impl_start..]
            .iter()
            .take_while(|&&p| p != ";")
            .copied()
            .collect();

        // Extract description and examples from comments
        let mut description = String::new();
        let mut examples = Vec::new();

        for comment in comments {
            if comment.starts_with("Example:") {
                examples.push(comment[8..].trim().to_string());
            } else if !comment.is_empty() {
                if !description.is_empty() {
                    description.push('\n');
                }
                description.push_str(comment);
            }
        }

        if description.is_empty() {
            description = format!("The {} word", name);
        }

        Ok(Some(WordDoc {
            name,
            stack_effect,
            description,
            examples,
            implementation: implementation.join(" "),
            category: "General".to_string(),
        }))
    }

    /// Generate HTML documentation
    fn generate_html(&self, word: &WordDoc) -> Result<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n");
        html.push_str("<head>\n");
        html.push_str(&format!("  <title>{} - Fast Forth Documentation</title>\n", word.name));
        html.push_str("  <style>\n");
        html.push_str(DOC_CSS);
        html.push_str("  </style>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Word signature
        html.push_str(&format!("  <h1>{}</h1>\n", word.name));
        html.push_str(&format!("  <div class=\"stack-effect\">( {} )</div>\n", word.stack_effect));

        // Description
        html.push_str("  <h2>Description</h2>\n");
        html.push_str(&format!("  <p>{}</p>\n", word.description));

        // Examples
        if !word.examples.is_empty() {
            html.push_str("  <h2>Examples</h2>\n");
            html.push_str("  <div class=\"examples\">\n");
            for example in &word.examples {
                html.push_str(&format!("    <pre>{}</pre>\n", example));
            }
            html.push_str("  </div>\n");
        }

        // Implementation
        html.push_str("  <h2>Implementation</h2>\n");
        html.push_str(&format!("  <pre>{}</pre>\n", word.implementation));

        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }

    /// Generate Markdown documentation
    fn generate_markdown(&self, word: &WordDoc) -> Result<String> {
        let mut md = String::new();

        md.push_str(&format!("# {}\n\n", word.name));
        md.push_str(&format!("**Stack Effect:** `( {} )`\n\n", word.stack_effect));

        md.push_str("## Description\n\n");
        md.push_str(&format!("{}\n\n", word.description));

        if !word.examples.is_empty() {
            md.push_str("## Examples\n\n");
            for example in &word.examples {
                md.push_str(&format!("```forth\n{}\n```\n\n", example));
            }
        }

        md.push_str("## Implementation\n\n");
        md.push_str(&format!("```forth\n{}\n```\n", word.implementation));

        Ok(md)
    }

    /// Generate documentation index
    fn generate_index(&self, words: &[WordDoc], output_dir: &Path) -> Result<PathBuf> {
        let index_path = match self.format {
            DocFormat::Html => output_dir.join("index.html"),
            DocFormat::Markdown => output_dir.join("index.md"),
        };

        let content = match self.format {
            DocFormat::Html => self.generate_html_index(words)?,
            DocFormat::Markdown => self.generate_markdown_index(words)?,
        };

        fs::write(&index_path, content)?;
        Ok(index_path)
    }

    fn generate_html_index(&self, words: &[WordDoc]) -> Result<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("  <title>Fast Forth Documentation - Index</title>\n");
        html.push_str("  <style>\n");
        html.push_str(DOC_CSS);
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");
        html.push_str("  <h1>Fast Forth Documentation</h1>\n");
        html.push_str("  <h2>Word Index</h2>\n");
        html.push_str("  <ul class=\"word-list\">\n");

        for word in words {
            html.push_str(&format!(
                "    <li><a href=\"{}.html\">{}</a> <span class=\"stack-effect\">( {} )</span> - {}</li>\n",
                word.name.to_lowercase(),
                word.name,
                word.stack_effect,
                word.description.lines().next().unwrap_or("")
            ));
        }

        html.push_str("  </ul>\n");
        html.push_str("</body>\n</html>\n");

        Ok(html)
    }

    fn generate_markdown_index(&self, words: &[WordDoc]) -> Result<String> {
        let mut md = String::new();

        md.push_str("# Fast Forth Documentation\n\n");
        md.push_str("## Word Index\n\n");

        for word in words {
            md.push_str(&format!(
                "- [{}]({}.md) `( {} )` - {}\n",
                word.name,
                word.name.to_lowercase(),
                word.stack_effect,
                word.description.lines().next().unwrap_or("")
            ));
        }

        Ok(md)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_word() {
        let generator = DocGenerator::new(DocFormat::Markdown);
        let source = ": SQUARE ( n -- n^2 ) DUP * ;";

        let words = generator.parse_words(source).unwrap();
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].name, "SQUARE");
        assert_eq!(words[0].stack_effect, "n -- n^2");
    }
}
