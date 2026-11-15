//! Logging module for CDSL.
//!
//! Provides file-based logging for executed instructions, success/failure statuses,
//! and diagnostic messages.

#[allow(clippy::module_inception)]
pub mod logging;

pub use logging::*;
