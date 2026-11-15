//! Handler for the WAIT command.
//!
//! Pauses interpreter execution for a specified duration in milliseconds.

use std::thread::sleep;
use std::time::Duration;

use crate::core::{
    diagnostics_error, diagnostics_success, evaluate_expression, get_variable, has_variable,
    is_valid_int_value,
};
use crate::custom_types::Instruction;

// Pauses thread execution for the given duration in milliseconds.
fn wait_function(delay: i64) {
    sleep(Duration::from_millis(delay as u64));
}

/// Processes a WAIT instruction.
///
/// Syntax: `WAIT <duration_ms>`
/// Duration may be an integer literal, variable reference (`$var`), or arithmetic expression (`#[...]`).
pub fn process_wait(instruction: &Instruction) {
    if instruction.len() != 2 {
        diagnostics_error(
            "Invalid number of arguments. Example: `WAIT DURATION_MS`",
            instruction,
        );
        return;
    }

    let value = &instruction[1];

    let delay: i64;

    // Case 1: Arithmetic expression enclosed in #[...]
    if value.starts_with("#[") {
        if value.len() < 3 || !value.ends_with(']') {
            diagnostics_error("Invalid expression", instruction);
            return;
        }

        let expression = &value[2..value.len() - 1];

        match evaluate_expression(expression) {
            Ok(result) => delay = result as i64,
            Err(err) => {
                diagnostics_error(&err, instruction);
                return;
            }
        }
    // Case 2: Variable reference starting with '$'
    } else if let Some(variable_name) = value.strip_prefix('$') {
        if !has_variable(variable_name) {
            diagnostics_error(&format!("Unknown variable: {}", variable_name), instruction);
            return;
        }

        delay = get_variable(variable_name) as i64;
    // Case 3: Direct integer literal
    } else {
        match is_valid_int_value(value) {
            Ok(result) => delay = result,
            Err(err) => {
                diagnostics_error(&err, instruction);
                return;
            }
        }
    }

    // Reject negative wait times.
    if delay < 0 {
        diagnostics_error(&format!("Delay cannot be negative: {}", delay), instruction);
        return;
    }

    diagnostics_success(&format!("Waiting for {} milliseconds", delay), instruction);

    wait_function(delay);
}
