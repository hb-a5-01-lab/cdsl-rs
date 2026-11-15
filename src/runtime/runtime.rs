//! Interactive REPL and command execution runtime for CDSL.

use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::core::{self, diagnostics_error, Config};
use crate::custom_types::Instruction;
use crate::instructions::{
    process_move, process_print, process_printc, process_reset, process_set, process_wait,
};
use crate::logging;
use crate::ui;

/// Dispatches a parsed instruction to its corresponding handler based on the command keyword.
pub fn process_instruction(config: &Config, instruction: &Instruction) {
    if instruction.is_empty() {
        return;
    }

    match instruction[0].as_str() {
        cmd if cmd.eq_ignore_ascii_case("set") => process_set(instruction),
        cmd if cmd.eq_ignore_ascii_case("print") => process_print(instruction),
        cmd if cmd.eq_ignore_ascii_case("move") => process_move(config, instruction),
        cmd if cmd.eq_ignore_ascii_case("wait") => process_wait(instruction),
        cmd if cmd.eq_ignore_ascii_case("printc") => process_printc(config),
        cmd if cmd.eq_ignore_ascii_case("reset") => process_reset(config, instruction),
        _ => diagnostics_error(&format!("Invalid action: {}", instruction[0]), instruction),
    }
}

/// Runs the interactive CDSL REPL shell.
///
/// Loads user configuration, manages persistent command history in `~/.cdsl_history`,
/// and evaluates user commands until exit.
pub fn run_interactive_mode() {
    // Load hardware and component configuration.
    let config = match core::load_config() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    // Determine path for persistent command history.
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            eprintln!("failed to get home directory");
            return;
        }
    };

    let history_file: PathBuf = home.join(".cdsl_history");

    // Initialize session logging.
    if let Err(err) = logging::init() {
        eprintln!("{}", err);
        return;
    }

    // Print welcome banner.
    println!("{}CDSL Interactive Mode{}", ui::YELLOW, ui::RESET);
    println!("Type `exit` to quit.");
    println!("Type `clear` to clear screen.");
    println!();

    // Initialize the rustyline editor.
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    // Load command history if available.
    let _ = rl.load_history(&history_file);

    // Show prompt only when running in an interactive terminal.
    let is_tty = io::stdin().is_terminal();
    let prompt = if is_tty {
        format!("{}cdsl-rs> {}", ui::YELLOW, ui::RESET)
    } else {
        String::new()
    };

    // Main REPL loop.
    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                if trimmed == "exit" {
                    break;
                }

                if trimmed == "clear" {
                    // ANSI escape codes to clear screen and reset cursor.
                    print!("\x1b[2J\x1b[H");
                    io::stdout().flush().ok();
                    continue;
                }

                // Tokenize command line and execute.
                let instruction = core::tokenize(trimmed);
                if instruction.is_empty() {
                    continue;
                }

                process_instruction(&config, &instruction);
            }
            Err(ReadlineError::Interrupted) => {
                // Handle Ctrl-C.
                break;
            }
            Err(ReadlineError::Eof) => {
                // Handle Ctrl-D.
                break;
            }
            Err(err) => {
                eprintln!("{}", err);
                break;
            }
        }
    }

    // Save history and close log file before exiting.
    let _ = rl.save_history(&history_file);
    let _ = logging::close();
}
