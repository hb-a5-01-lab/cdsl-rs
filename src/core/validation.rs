//! Validation routines for numeric inputs and variable identifiers.

use std::num::IntErrorKind;

/// Validates and parses a string as a finite 32-bit floating-point number.
pub fn is_valid_float_value(string_value: &str) -> Result<f32, String> {
    match string_value.parse::<f32>() {
        Ok(value) => {
            // Reject values that overflowed to infinity unless explicitly requested.
            if value.is_infinite()
                && !string_value
                    .as_bytes()
                    .windows(3)
                    .any(|w| w.eq_ignore_ascii_case(b"inf"))
            {
                Err("value out of range".to_string())
            } else {
                Ok(value)
            }
        }
        Err(_) => Err(format!("invalid value: {}", string_value)),
    }
}

/// Validates and parses a string as a 64-bit signed integer.
pub fn is_valid_int_value(string_value: &str) -> Result<i64, String> {
    string_value.parse::<i64>().map_err(|e| {
        if matches!(
            e.kind(),
            IntErrorKind::PosOverflow | IntErrorKind::NegOverflow
        ) {
            "value out of range".to_string()
        } else {
            format!("invalid value: {}", string_value)
        }
    })
}

/// Validates that a string is a legal CDSL variable identifier.
///
/// Must begin with a letter or underscore, followed by letters, digits, or underscores.
pub fn is_valid_variable_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_alphabetic() || first == '_' => {
            chars.all(|c| c.is_alphanumeric() || c == '_')
        }
        _ => false,
    }
}
