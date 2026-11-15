//! XML configuration loader and component registry for CDSL.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Configuration parameters for an individual hardware component.
#[derive(Debug, Clone)]
pub struct ComponentConfig {
    /// Numeric hardware ID used in serial commands.
    pub id: i64,
    /// Minimum allowed movement limit (inclusive).
    pub min: i64,
    /// Maximum allowed movement limit (inclusive).
    pub max: i64,
    /// Default position for component reset.
    pub default_val: i64,
}

/// Serial communication settings for the connected Arduino.
#[derive(Debug, Clone)]
pub struct ArduinoConfig {
    /// Serial port device path (e.g., `/dev/ttyUSB0`).
    pub port: String,
    /// Baud rate for communication (e.g., `115200`).
    pub baud_rate: i64,
}

/// Root configuration holding hardware and component settings.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Arduino serial communication configuration.
    pub arduino: Option<ArduinoConfig>,
    /// Map of component names to their configurations.
    pub components: HashMap<String, ComponentConfig>,
}

// Helper to parse integer text inside a child element with the given tag name.
fn parse_child_i64(node: roxmltree::Node<'_, '_>, tag: &str) -> i64 {
    node.children()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse::<i64>().ok())
        .unwrap_or(0)
}

/// Loads and parses the CDSL configuration file from `~/.config/cdsl/config.xml`.
pub fn load_config() -> Result<Config, String> {
    let home = dirs::home_dir().ok_or_else(|| "failed to get home directory".to_string())?;

    let config_file: PathBuf = home.join(".config").join("cdsl").join("config.xml");

    let data =
        fs::read_to_string(&config_file).map_err(|e| format!("failed to read config: {}", e))?;

    let doc =
        roxmltree::Document::parse(&data).map_err(|e| format!("failed to parse config: {}", e))?;

    let root = doc.root_element();
    if !root.has_tag_name("config") {
        return Err("failed to parse config: root element must be <config>".to_string());
    }

    // Parse Arduino settings if present.
    let mut arduino = None;
    if let Some(ard_node) = root.children().find(|n| n.has_tag_name("arduino")) {
        let port = ard_node
            .children()
            .find(|n| n.has_tag_name("port"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
            .to_string();

        let baud_rate = parse_child_i64(ard_node, "baud_rate");

        arduino = Some(ArduinoConfig { port, baud_rate });
    }

    // Parse hardware components under the <component> section.
    let mut components = HashMap::new();
    if let Some(comp_node) = root.children().find(|n| n.has_tag_name("component")) {
        for child in comp_node.children().filter(|n| n.is_element()) {
            let name = child.tag_name().name().to_string();
            let id = parse_child_i64(child, "id");
            let min = parse_child_i64(child, "min");
            let max = parse_child_i64(child, "max");
            let default_val = parse_child_i64(child, "default");

            components.insert(
                name,
                ComponentConfig {
                    id,
                    min,
                    max,
                    default_val,
                },
            );
        }
    }

    Ok(Config {
        arduino,
        components,
    })
}

/// Checks whether a component with the specified name exists in the configuration.
pub fn component_exists(config: &Config, component_name: &str) -> bool {
    config.components.contains_key(component_name)
}

/// Returns the numeric hardware ID (0-255) for the specified component.
pub fn component_id(config: &Config, component_name: &str) -> Result<u8, String> {
    if config.components.is_empty() {
        return Err("no components configured".to_string());
    }

    let component = config
        .components
        .get(component_name)
        .ok_or_else(|| format!("component not configured: {}", component_name))?;

    if component.id < 0 || component.id > 255 {
        return Err("component ID must be between 0 and 255".to_string());
    }

    Ok(component.id as u8)
}

/// Checks whether a given value is within the configured min and max limits for the component.
pub fn value_within_limits(
    config: &Config,
    component_name: &str,
    value: f32,
) -> Result<bool, String> {
    Ok(config
        .components
        .get(component_name)
        .is_some_and(|c| value >= c.min as f32 && value <= c.max as f32))
}

/// Returns the configured default position value for the specified component.
pub fn component_default(config: &Config, component_name: &str) -> Result<f32, String> {
    if config.components.is_empty() {
        return Err("no components configured".to_string());
    }

    let component = config
        .components
        .get(component_name)
        .ok_or_else(|| format!("component not configured: {}", component_name))?;

    Ok(component.default_val as f32)
}

/// Returns the configured serial port path for the Arduino.
pub fn arduino_port(config: &Config) -> Result<String, String> {
    let arduino = config
        .arduino
        .as_ref()
        .ok_or_else(|| "arduino serial port is not configured".to_string())?;

    if arduino.port.is_empty() {
        return Err("arduino serial port is not configured".to_string());
    }

    Ok(arduino.port.clone())
}

/// Returns the configured baud rate for Arduino serial communication.
pub fn arduino_baud_rate(config: &Config) -> Result<i32, String> {
    let arduino = config
        .arduino
        .as_ref()
        .ok_or_else(|| "arduino baud rate is not configured".to_string())?;

    if arduino.baud_rate == 0 {
        return Err("arduino baud rate is not configured".to_string());
    }

    Ok(arduino.baud_rate as i32)
}
