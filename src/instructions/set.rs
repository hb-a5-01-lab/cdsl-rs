//! Handler for the SET command.
//!
//! Assigns values to variables in the runtime variable table.

use crate::core::{
    diagnostics_error, diagnostics_success, evaluate_expression, get_variable, has_variable,
    is_valid_float_value, is_valid_variable_name, set_variable,
};
use crate::custom_types::Instruction;

/// Processes a SET instruction.
///
/// Syntax: `SET <variable> <value>`
/// Supports numeric literals, variable copies (`$var`), and arithmetic expressions (`#[...]`).
pub fn process_set(instruction: &Instruction) {
    if instruction.len() != 3 {
        diagnostics_error(
            "Invalid number of arguments. Example: `SET VARIABLE VALUE`",
            instruction,
        );
        return;
    }

    // Validate that the target variable name meets identifier rules.
    if !is_valid_variable_name(&instruction[1]) {
        diagnostics_error(
            &format!("Invalid variable name: {}", instruction[1]),
            instruction,
        );
        return;
    }

    let variable_key = &instruction[1];
    let value = &instruction[2];

    let variable_value: f32;

    // Case 1: Arithmetic expression enclosed in #[...]
    if value.starts_with("#[") {
        if value.len() < 3 || !value.ends_with(']') {
            diagnostics_error("Invalid expression", instruction);
            return;
        }

        let expression = &value[2..value.len() - 1];

        match evaluate_expression(expression) {
            Ok(result) => variable_value = result,
            Err(err) => {
                diagnostics_error(&err, instruction);
                return;
            }
        }
    // Case 2: Reference to another variable
    } else if let Some(variable_name) = value.strip_prefix('$') {
        if !has_variable(variable_name) {
            diagnostics_error(&format!("Unknown variable: {}", variable_name), instruction);
            return;
        }

        variable_value = get_variable(variable_name);
    // Case 3: Direct floating-point literal
    } else {
        match is_valid_float_value(value) {
            Ok(result) => variable_value = result,
            Err(err) => {
                diagnostics_error(&err, instruction);
                return;
            }
        }
    }

    // Save variable to the runtime store and acknowledge.
    set_variable(variable_key, variable_value);
    diagnostics_success(
        &format!("Variable {} set to {}", variable_key, variable_value),
        instruction,
    );
}
