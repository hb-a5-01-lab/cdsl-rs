//! Entry point for the CDSL (Custom Description Scripting Language) CLI.

use std::env;
use std::process;

/// Parses command-line arguments and launches the CDSL interactive shell.
fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for optional command-line flags.
    if args.len() > 1 {
        if args.len() == 2 && args[1] == "--help" {
            println!("Usage: cdsl [--help]");
            return;
        }

        eprintln!("Error: Unknown option.");
        eprintln!("Usage: cdsl [--help]");
        process::exit(1);
    }

    // Launch the interactive REPL.
    cdsl::runtime::run_interactive_mode();
}
