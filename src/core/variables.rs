//! Thread-safe runtime variable storage for CDSL.

use std::collections::HashMap;
use std::sync::RwLock;

// Global variable table guarded by a read-write lock for thread-safe access.
static VARIABLES: RwLock<Option<HashMap<String, f32>>> = RwLock::new(None);

/// Assigns a floating-point value to a variable, creating it if it does not exist.
pub fn set_variable(variable_key: &str, variable_value: f32) {
    VARIABLES
        .write()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(variable_key.to_string(), variable_value);
}

/// Checks whether a variable has been defined in the runtime table.
pub fn has_variable(name: &str) -> bool {
    let lock = VARIABLES.read().unwrap();
    lock.as_ref().is_some_and(|m| m.contains_key(name))
}

/// Retrieves the value of a variable. Returns `0.0` if the variable is undefined.
pub fn get_variable(name: &str) -> f32 {
    let lock = VARIABLES.read().unwrap();
    lock.as_ref()
        .and_then(|m| m.get(name).copied())
        .unwrap_or(0.0)
}

/// Prints all currently defined variables and their values to standard output.
pub fn display_variables() {
    let lock = VARIABLES.read().unwrap();
    if let Some(map) = lock.as_ref() {
        for (key, value) in map {
            println!("{} : {:.6}", key, value);
        }
    }
}

/// Clears all variables from the runtime storage table (useful for test isolation).
pub fn clear_variables() {
    let mut lock = VARIABLES.write().unwrap();
    if let Some(map) = lock.as_mut() {
        map.clear();
    }
}
