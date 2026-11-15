//! CDSL (Custom Description Scripting Language)
//!
//! A lightweight domain-specific language for robotic movement and hardware control.
//! Provides an interactive REPL, arithmetic expression evaluation, variable storage,
//! and serial communication with microcontrollers.

pub mod core;
pub mod custom_types;
pub mod hardware;
pub mod instructions;
pub mod logging;
pub mod runtime;
pub mod ui;
