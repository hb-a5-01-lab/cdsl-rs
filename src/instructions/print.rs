//! Handler for the PRINT command.
//!
//! Evaluates and displays variable values or arithmetic expressions.

use crate::core::{
    diagnostics_error, diagnostics_success, evaluate_expression, get_variable, has_variable,
};
use crate::custom_types::Instruction;

/// Processes a PRINT instruction.
///
/// Syntax: `PRINT <value>`
/// Supports variable references (`$var` or `var`) and arithmetic expressions (`#[...]`).
pub fn process_print(instruction: &Instruction) {
    if instruction.len() != 2 {
        diagnostics_error(
            "Invalid number of arguments. Example: `PRINT VALUE`",
            instruction,
        );
        return;
    }

    let argument = &instruction[1];

    // Case 1: Arithmetic expression enclosed in #[...]
    if argument.starts_with("#[") {
        if argument.len() < 3 || !argument.ends_with(']') {
            diagnostics_error("Invalid expression", instruction);
            return;
        }

        let expression = &argument[2..argument.len() - 1];

        match evaluate_expression(expression) {
            Ok(result) => {
                let message = format!("{}", result);
                diagnostics_success(&message, instruction);
                return;
            }
            Err(err) => {
                diagnostics_error(&err, instruction);
                return;
            }
        }
    }

    // Case 2: Variable name (with or without leading '$')
    let variable_key = argument.strip_prefix('$').unwrap_or(argument);

    if !has_variable(variable_key) {
        diagnostics_error(
            &format!("No variable with name {}", variable_key),
            instruction,
        );
        return;
    }

    let message = format!("{}", get_variable(variable_key));
    diagnostics_success(&message, instruction);
}
