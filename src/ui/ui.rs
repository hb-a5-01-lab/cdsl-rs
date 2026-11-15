//! ANSI escape codes for formatted terminal output.

/// ANSI red escape code for error messages.
pub const RED: &str = "\x1b[31m";

/// ANSI green escape code for success messages.
pub const GREEN: &str = "\x1b[32m";

/// ANSI yellow escape code for prompts and highlights.
pub const YELLOW: &str = "\x1b[33m";

/// ANSI white escape code for regular text.
pub const WHITE: &str = "\x1b[37m";

/// ANSI reset code to restore terminal styling.
pub const RESET: &str = "\x1b[0m";
