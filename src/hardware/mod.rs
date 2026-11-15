//! Hardware interface module.
//!
//! Handles low-level serial communication and command execution for connected microcontrollers.

#[allow(clippy::module_inception)]
pub mod hardware;

pub use hardware::*;
