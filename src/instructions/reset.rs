//! Handler for the RESET command.
//!
//! Restores all configured hardware components to their default positions.

use std::thread::sleep;
use std::time::Duration;

use crate::core::{diagnostics_error, diagnostics_success, Config};
use crate::custom_types::Instruction;
use crate::instructions::move_cmd::execute_move;

// Holds component information required for executing a reset sequence.
struct ComponentInfo<'a> {
    name: &'a str,
    id: i64,
    default_value: i64,
}

/// Moves all configured components to their default values in ascending order of their IDs.
///
/// Inserts a 2-second pause between movements to allow mechanical components to settle.
pub fn process_reset(config: &Config, instruction: &Instruction) {
    if config.components.is_empty() {
        diagnostics_error("No valid component configuration found", instruction);
        return;
    }

    // Collect components and sort them by ID.
    let mut component_list: Vec<ComponentInfo<'_>> = config
        .components
        .iter()
        .map(|(name, comp)| ComponentInfo {
            name: name.as_str(),
            id: comp.id,
            default_value: comp.default_val,
        })
        .collect();

    component_list.sort_by_key(|c| c.id);

    // Reset components one by one with a delay in between.
    for (i, component) in component_list.iter().enumerate() {
        let value = component.default_value as f32;
        let value_text = component.default_value.to_string();

        if !execute_move(config, instruction, component.name, &value_text, value) {
            return;
        }

        // Delay between movements, skipping the final iteration.
        if i < component_list.len() - 1 {
            sleep(Duration::from_secs(2));
        }
    }

    diagnostics_success(
        "All components reset to their default positions",
        instruction,
    );
}
