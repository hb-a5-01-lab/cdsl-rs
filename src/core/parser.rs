//! Tokenizer for CDSL command lines.

use crate::custom_types::Instruction;

/// Splits an input line into individual whitespace-delimited tokens.
///
/// Any comment starting with `//` and all text following it are ignored.
pub fn tokenize(line: &str) -> Instruction {
    line.split_whitespace()
        .take_while(|token| !token.starts_with("//"))
        .map(String::from)
        .collect()
}
