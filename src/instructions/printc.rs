//! Handler for the PRINTC command.
//!
//! Displays configured Arduino communication parameters and a formatted table of all hardware components.

use crate::core::{arduino_baud_rate, arduino_port, Config};

// Internal representation of component row for tabular display.
struct ComponentRow<'a> {
    id_str: String,
    id: i64,
    name: &'a str,
    range_str: String,
    default_str: String,
}

/// Displays the current hardware configuration in an aligned table.
pub fn process_printc(config: &Config) {
    // Print serial connection settings.
    match arduino_port(config) {
        Ok(port) => println!("port = {:?}", port),
        Err(err) => println!("port = error: {}", err),
    }

    match arduino_baud_rate(config) {
        Ok(baud_rate) => println!("baud_rate = {}", baud_rate),
        Err(err) => println!("baud_rate = error: {}", err),
    }

    println!();
    println!("Components:");

    if config.components.is_empty() {
        println!("No components configured");
        return;
    }

    let mut id_width = "c_id".len();
    let mut name_width = "c_name".len();
    let mut range_width = "c_range".len();
    let mut default_width = "default".len();

    // Collect components, precompute formatted values, and determine column widths in a single pass.
    let mut component_list: Vec<ComponentRow<'_>> = config
        .components
        .iter()
        .map(|(name, comp)| {
            let id_str = comp.id.to_string();
            let range_str = format!("{}-{}", comp.min, comp.max);
            let default_str = comp.default_val.to_string();

            id_width = id_width.max(id_str.len());
            name_width = name_width.max(name.len());
            range_width = range_width.max(range_str.len());
            default_width = default_width.max(default_str.len());

            ComponentRow {
                id_str,
                id: comp.id,
                name: name.as_str(),
                range_str,
                default_str,
            }
        })
        .collect();

    // Sort components by ID for ordered output.
    component_list.sort_by_key(|c| c.id);

    // Print table header.
    println!(
        "{:<id_w$} | {:<name_w$} | {:<range_w$} | {:<default_w$}",
        "c_id",
        "c_name",
        "c_range",
        "default",
        id_w = id_width,
        name_w = name_width,
        range_w = range_width,
        default_w = default_width,
    );

    let table_width = id_width + 3 + name_width + 3 + range_width + 3 + default_width;
    println!("{}", "-".repeat(table_width));

    // Print each component row.
    for component in &component_list {
        println!(
            "{:<id_w$} | {:<name_w$} | {:<range_w$} | {:<default_w$}",
            component.id_str,
            component.name,
            component.range_str,
            component.default_str,
            id_w = id_width,
            name_w = name_width,
            range_w = range_width,
            default_w = default_width,
        );
    }

    println!();
}
