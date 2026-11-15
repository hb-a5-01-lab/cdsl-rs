//! CDSL runtime engine.
//!
//! Provides the interactive REPL and command execution pipeline.

#[allow(clippy::module_inception)]
pub mod runtime;

pub use runtime::*;
