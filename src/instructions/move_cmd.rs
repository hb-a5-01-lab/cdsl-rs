//! Handler for the MOVE command.
//!
//! Validates component positions against configured limits and sends movement instructions to hardware.

use crate::core::{
    component_exists, diagnostics_error, diagnostics_success, evaluate_expression, get_variable,
    has_variable, is_valid_float_value, value_within_limits, Config,
};
use crate::custom_types::Instruction;
use crate::hardware;

/// Validates limits for a component and dispatches the movement command to the serial driver.
pub fn execute_move(
    config: &Config,
    instruction: &Instruction,
    component_label: &str,
    value_text: &str,
    value: f32,
) -> bool {
    // Check if the requested position is within allowed hardware limits.
    let within_limits = match value_within_limits(config, component_label, value) {
        Ok(w) => w,
        Err(err) => {
            diagnostics_error(&err, instruction);
            return false;
        }
    };

    if !within_limits {
        diagnostics_error(
            &format!("Value out of range for {}: {:.6}", component_label, value),
            instruction,
        );
        return false;
    }

    // Transmit command to the connected hardware.
    if let Err(err) = hardware::send_command(config, component_label, value) {
        diagnostics_error(&err, instruction);
        return false;
    }

    diagnostics_success(
        &format!("Moved {} by {}", component_label, value_text),
        instruction,
    );

    true
}

/// Processes a MOVE instruction.
///
/// Syntax: `MOVE <component> <value>`
/// The value can be a numeric literal, variable reference (`$var`), or arithmetic expression (`#[...]`).
pub fn process_move(config: &Config, instruction: &Instruction) {
    if instruction.len() != 3 {
        diagnostics_error(
            "Invalid number of arguments. Example: `MOVE COMPONENT_NAME VALUE`",
            instruction,
        );
        return;
    }

    let component_label = &instruction[1];

    // Ensure the target component exists in the configuration.
    if !component_exists(config, component_label) {
        diagnostics_error(
            &format!("Undefined component: {}", component_label),
            instruction,
        );
        return;
    }

    let value = &instruction[2];

    // Case 1: Arithmetic expression enclosed in #[...]
    if value.starts_with("#[") {
        if value.len() < 3 || !value.ends_with(']') {
            diagnostics_error("Invalid expression", instruction);
            return;
        }

        let expression = &value[2..value.len() - 1];

        match evaluate_expression(expression) {
            Ok(result) => {
                execute_move(config, instruction, component_label, value, result);
            }
            Err(err) => {
                diagnostics_error(&err, instruction);
            }
        }
        return;
    }

    // Case 2: Variable reference starting with '$'
    if let Some(variable_name) = value.strip_prefix('$') {
        if !has_variable(variable_name) {
            diagnostics_error(&format!("Unknown variable: {}", variable_name), instruction);
            return;
        }

        let result = get_variable(variable_name);
        execute_move(config, instruction, component_label, value, result);
        return;
    }

    // Case 3: Direct floating-point literal
    match is_valid_float_value(value) {
        Ok(result) => {
            execute_move(config, instruction, component_label, value, result);
        }
        Err(err) => {
            diagnostics_error(&err, instruction);
        }
    }
}
