//! Diagnostics and feedback reporting for CDSL commands.

use crate::custom_types::Instruction;
use crate::logging;
use crate::ui;

/// Prints the tokens of an instruction separated by spaces.
pub fn display_instruction(instruction: &Instruction) {
    for token in instruction {
        print!("{} ", token);
    }
    println!();
}

/// Formats and prints an error message to stderr and records the failure in the log file.
pub fn diagnostics_error(message: &str, instruction: &Instruction) {
    let instr_str = instruction.join(" ");
    eprint!("{}[{}]: {}{}\n\n", ui::RED, instr_str, message, ui::RESET);

    if let Err(err) = logging::log_it(false, message, instruction) {
        eprintln!("Logging error: {}", err);
    }
}

/// Formats and prints a success message to stdout and records the success in the log file.
pub fn diagnostics_success(message: &str, instruction: &Instruction) {
    let instr_str = instruction.join(" ");
    print!("{}[{}]: {}{}\n\n", ui::GREEN, instr_str, message, ui::RESET);

    if let Err(err) = logging::log_it(true, message, instruction) {
        eprintln!("Logging error: {}", err);
    }
}
